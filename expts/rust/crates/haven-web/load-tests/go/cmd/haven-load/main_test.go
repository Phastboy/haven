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

func TestRouteLabelNormalization(t *testing.T) {
	tests := []struct {
		input string
		want  string
	}{
		{"/offers", "/offers"},
		{"/offers/rustic-table-123", "/offers/{slug}"},
		{"/offers/rustic-table-123/save", "/offers/{slug}/save"},
		{"/offers/rustic-table-123/message", "/offers/{slug}/message"},
		{"/offers?q=Test+Offer&currency=NGN&min_price=1000&max_price=10000", "/offers?currency&max_price&min_price&q"},
		{"/offers?q=Test+Offer&after=abc", "/offers?q"},
		{"/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301", "/profile/offers/{id}"},
		{"/profile/offers/3f2504e0-4f89-11d3-9a0c-0305e82c3301/edit", "/profile/offers/{id}/edit"},
		{"/profile/offers/new", "/profile/offers/new"},
		{"/profiles/alex-wood", "/profiles/{handle}"},
		{"/profiles/alex-wood/follow", "/profiles/{handle}/follow"},
		{"/profile/saved?after=abc", "/profile/saved"},
		{"/conversations/3f2504e0-4f89-11d3-9a0c-0305e82c3301", "/conversations/{id}"},
		{"/conversations/3f2504e0-4f89-11d3-9a0c-0305e82c3301/messages", "/conversations/{id}/messages"},
		{"/health", "/health"},
		{"/", "/"},
	}
	for _, tt := range tests {
		if got := routeLabel(tt.input); got != tt.want {
			t.Errorf("routeLabel(%q) = %q, want %q", tt.input, got, tt.want)
		}
	}
}

func TestProfileExitCode(t *testing.T) {
	if got := profileExitCode(runSummary{Failed: 1}); got != 1 {
		t.Fatalf("expected 1 for failed journeys in profile context, got %d", got)
	}
	if got := profileExitCode(runSummary{ProfileFailed: true}); got != 1 {
		t.Fatalf("expected 1 when profile failed, got %d", got)
	}
	if got := profileExitCode(runSummary{ProfileFailed: false, Failed: 0, Dropped: 0}); got != 0 {
		t.Fatalf("expected 0 when profile not failed, got %d", got)
	}
}

func TestBodyInclusiveLatency(t *testing.T) {
	collector := &stats{samples: make(map[string][]time.Duration)}
	client := &http.Client{Transport: roundTripFunc(func(*http.Request) (*http.Response, error) {
		time.Sleep(80 * time.Millisecond)
		return &http.Response{StatusCode: http.StatusOK, Header: make(http.Header), Body: io.NopCloser(strings.NewReader("ok")), Request: nil}, nil
	})}
	if err := get(context.Background(), client, "http://example.test", "/health", "", http.StatusOK, collector); err != nil {
		t.Fatal(err)
	}
	if len(collector.samples["/health"]) != 1 {
		t.Fatalf("expected 1 sample, got %d", len(collector.samples))
	}
	if collector.samples["/health"][0] < 50*time.Millisecond {
		t.Fatalf("expected body-inclusive latency >= 50ms, got %s", collector.samples["/health"][0])
	}
}

func TestOverallPercentiles(t *testing.T) {
	collector := &stats{samples: make(map[string][]time.Duration)}
	collector.record(sample{Route: "/a", Duration: 10 * time.Millisecond}, true)
	collector.record(sample{Route: "/a", Duration: 20 * time.Millisecond}, true)
	collector.record(sample{Route: "/b", Duration: 30 * time.Millisecond}, true)
	summary := makeSummary("test", time.Second, 1, 3, 3, 3, 0, 0, collector)
	if summary.Overall.Count != 3 {
		t.Fatalf("expected overall count 3, got %d", summary.Overall.Count)
	}
	if summary.Overall.P50 != 20*time.Millisecond {
		t.Fatalf("expected overall p50 20ms, got %s", summary.Overall.P50)
	}
}

func TestScheduledInvariant(t *testing.T) {
	ctx := context.Background()
	client := &http.Client{Transport: roundTripFunc(func(*http.Request) (*http.Response, error) {
		return &http.Response{StatusCode: http.StatusOK, Header: make(http.Header), Body: io.NopCloser(strings.NewReader("ok")), Request: nil}, nil
	})}
	sessions := []session{{Session: "t", OfferIDs: []string{"1"}, Handle: "h"}}
	scheduled, started, completed, dropped, failed := run(ctx, client, "http://example.test", "steady", sessions, 1, 1200*time.Millisecond, 1, 0, &stats{samples: make(map[string][]time.Duration)})
	if scheduled != started+dropped {
		t.Fatalf("scheduled != started+dropped: %d != %d+%d", scheduled, started, dropped)
	}
	if failed != 0 {
		t.Fatalf("failed = %d", failed)
	}
	_ = completed
}
