package main

import (
	"context"
	"io"
	"net/http"
	"os"
	"strings"
	"testing"
	"time"
)

func TestValidateRejectsUnsafeOrInvalidConfiguration(t *testing.T) {
	tests := []struct {
		name     string
		baseURL  string
		scenario string
		rate     int
		duration time.Duration
		workers  int
	}{
		{name: "unsupported scheme", baseURL: "file:///etc/passwd", scenario: "smoke", rate: 1, duration: time.Second, workers: 1},
		{name: "unknown scenario", baseURL: "http://localhost", scenario: "unknown", rate: 1, duration: time.Second, workers: 1},
		{name: "zero rate", baseURL: "http://localhost", scenario: "smoke", rate: 0, duration: time.Second, workers: 1},
		{name: "zero duration", baseURL: "http://localhost", scenario: "smoke", rate: 1, duration: 0, workers: 1},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if err := validate(test.baseURL, test.scenario, test.rate, test.duration, test.workers); err == nil {
				t.Fatal("expected invalid config error")
			}
		})
	}
}

func TestProfileStagesMatchPerformanceContract(t *testing.T) {
	spike := profileStages("spike")
	if len(spike) != 3 || spike[0].rate != 100 || spike[1].rate != 2000 || spike[2].rate != 100 {
		t.Fatalf("spike stages = %#v", spike)
	}
	for _, stage := range spike {
		if stage.duration != time.Minute {
			t.Fatalf("spike stage duration = %s, want 1m", stage.duration)
		}
	}
	capacity := profileStages("capacity")
	wantRates := []int{100, 250, 500, 750, 1000, 1500, 2000, 3000, 5000}
	if len(capacity) != len(wantRates) {
		t.Fatalf("capacity stage count = %d, want %d", len(capacity), len(wantRates))
	}
	for index, rate := range wantRates {
		if capacity[index].rate != rate || capacity[index].duration != 2*time.Minute {
			t.Fatalf("capacity stage %d = %#v, want %d journeys/s for 2m", index, capacity[index], rate)
		}
	}
}

func TestCapacityBreachStopsAtContractThresholds(t *testing.T) {
	if got := capacityBreach(runSummary{Dropped: 1, Requests: 100}); got == "" {
		t.Fatal("expected dropped-journey breach")
	}
	if got := capacityBreach(runSummary{Requests: 100, Routes: []routeSummary{{Route: "/offers", P95: 251 * time.Millisecond}}}); got == "" {
		t.Fatal("expected p95 breach")
	}
	if got := capacityBreach(runSummary{Requests: 100, Failed: 2}); got == "" {
		t.Fatal("expected error-rate breach")
	}
	if got := capacityBreach(runSummary{Requests: 100, Routes: []routeSummary{{Route: "/offers", P95: 250 * time.Millisecond}}}); got != "" {
		t.Fatalf("threshold equality should pass, got %q", got)
	}
}

func TestSpikeBreachUsesSpikeAndRecoveryThresholds(t *testing.T) {
	if got := spikeBreach(runSummary{Requests: 100, Routes: []routeSummary{{Route: "/offers", P95: time.Second}}}, 1); got == "" {
		t.Fatal("expected spike latency breach at 1s")
	}
	if got := spikeBreach(runSummary{Requests: 100, Failed: 1}, 1); got == "" {
		t.Fatal("expected spike error breach at 1%")
	}
	if got := spikeBreach(runSummary{Requests: 100, Routes: []routeSummary{{Route: "/offers", P95: 100 * time.Millisecond}}}, 2); got == "" {
		t.Fatal("expected recovery latency breach at 100ms")
	}
	if got := spikeBreach(runSummary{Requests: 100, Routes: []routeSummary{{Route: "/offers", P95: 99 * time.Millisecond}}}, 2); got != "" {
		t.Fatalf("recovery under threshold rejected: %q", got)
	}
}

func TestPercentileUsesNearestLowerSample(t *testing.T) {
	samples := []time.Duration{time.Millisecond, 2 * time.Millisecond, 3 * time.Millisecond, 4 * time.Millisecond}
	if got := percentile(samples, .95); got != 4*time.Millisecond {
		t.Fatalf("p95 = %s, want 4ms", got)
	}
	if got := percentile(nil, .95); got != 0 {
		t.Fatalf("empty p95 = %s, want 0", got)
	}
}

func TestLoadSessionsRejectsEmptyFixture(t *testing.T) {
	file := t.TempDir() + "/sessions.json"
	if err := os.WriteFile(file, []byte("[]"), 0o600); err != nil {
		t.Fatal(err)
	}
	if _, err := loadSessions(file); err == nil {
		t.Fatal("expected empty fixture error")
	}
}

func TestValidateSessionsRequiresMigratedFixtureFields(t *testing.T) {
	legacy := []session{{Session: "token", OfferIDs: []string{"id"}}}
	if err := validateSessions("steady", legacy); err == nil {
		t.Fatal("expected legacy fixture to be rejected")
	}
	migrated := []session{{Session: "token", Handle: "person", OfferIDs: []string{"id"}}}
	if err := validateSessions("steady", migrated); err != nil {
		t.Fatalf("migrated fixture rejected: %v", err)
	}
}

func TestScenarioTitleStaysWithinOfferLimit(t *testing.T) {
	title := scenarioTitle("Idempotency", session{UserID: "12345678-1234-1234-1234-123456789012"}, 1234567890)
	if len(title) > 50 {
		t.Fatalf("scenario title has %d bytes, want at most 50: %q", len(title), title)
	}
}

func TestOpenArrivalSchedulerCompletesSmokeJourneys(t *testing.T) {
	collector := &stats{samples: make(map[string][]time.Duration)}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	client := &http.Client{Transport: roundTripFunc(func(request *http.Request) (*http.Response, error) {
		if request.URL.Path != "/health" {
			t.Errorf("path = %s, want /health", request.URL.Path)
		}
		return &http.Response{
			StatusCode: http.StatusOK,
			Header:     make(http.Header),
			Body:       io.NopCloser(strings.NewReader("ok")),
			Request:    request,
		}, nil
	})}
	scheduled, started, completed, dropped, failed := run(ctx, client, "http://example.test", "smoke", nil, 1000, time.Second, 3, 10, collector)
	if scheduled != 10 || started != 10 || completed != 10 || dropped != 0 || failed != 0 {
		t.Fatalf("scheduler counts = %d/%d/%d/%d/%d", scheduled, started, completed, dropped, failed)
	}
}

func TestPostHelperPreservesEmptyPostForms(t *testing.T) {
	collector := &stats{samples: make(map[string][]time.Duration)}
	client := &http.Client{Transport: roundTripFunc(func(request *http.Request) (*http.Response, error) {
		if request.Method != http.MethodPost {
			t.Errorf("method = %s, want POST", request.Method)
		}
		return &http.Response{StatusCode: http.StatusSeeOther, Header: make(http.Header), Body: io.NopCloser(strings.NewReader("")), Request: request}, nil
	})}
	if err := post(context.Background(), client, "http://example.test", "/delete", "token", nil, http.StatusSeeOther, collector); err != nil {
		t.Fatal(err)
	}
}

type roundTripFunc func(*http.Request) (*http.Response, error)

func (roundTrip roundTripFunc) RoundTrip(request *http.Request) (*http.Response, error) {
	return roundTrip(request)
}
