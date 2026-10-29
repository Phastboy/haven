package main

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"os"
	"strings"
	"testing"
	"time"
)

func TestNarrowingMixAndHistoryWeights(t *testing.T) {
	config := &narrowingConfig{class: "mixed", groups: make(map[string][]narrowingCase)}
	for _, class := range []string{"public", "public-authenticated", "saved", "inquired"} {
		histories := []string{"dense", "sparse", "zero"}
		if class == "public" {
			histories = []string{"none"}
		}
		for _, history := range histories {
			key := class
			if class != "public" {
				key += "/" + history
			}
			config.groups[key] = []narrowingCase{{Class: class, History: history}}
		}
	}
	counts := map[string]int{}
	for i := 0; i < 600; i++ {
		entry, _ := config.choose(i)
		counts[entry.Class]++
		if entry.Class != "public" {
			counts[entry.Class+"/"+entry.History]++
		}
	}
	for key, want := range map[string]int{"public": 210, "public-authenticated": 210, "saved": 90, "inquired": 90, "public-authenticated/dense": 70, "public-authenticated/sparse": 70, "public-authenticated/zero": 70, "saved/dense": 30, "saved/sparse": 30, "saved/zero": 30, "inquired/dense": 30, "inquired/sparse": 30, "inquired/zero": 30} {
		if counts[key] != want {
			t.Fatalf("%s=%d want %d", key, counts[key], want)
		}
	}
}

func TestNarrowingSamplingIsReproduciblePerCase(t *testing.T) {
	config := &narrowingConfig{percent: 10, seed: 42}
	for _, id := range []string{"public-dominant-first", "saved-dense-later", "inquired-sparse-first"} {
		total := 0
		for i := uint64(0); i < 100; i++ {
			if config.check(narrowingCase{ID: id}, i) {
				total++
			}
		}
		if total != 10 {
			t.Fatalf("%s checks=%d want 10", id, total)
		}
	}
}

func TestNarrowingChecksExactOffersOrderAndPrivacy(t *testing.T) {
	entry := narrowingCase{ID: "test", Class: "saved", ExpectedSlugs: []string{"rice-a", "rice-b"}}
	body := `<li id="offer-rice-a" class="listing-card"></li><li id="offer-rice-b" class="listing-card"></li>`
	headers := http.Header{"Cache-Control": []string{"private, no-store"}, "X-Robots-Tag": []string{"noindex, follow"}}
	if err := checkNarrowing(entry, body, headers); err != nil {
		t.Fatal(err)
	}
	if err := checkNarrowing(entry, strings.Replace(body, "rice-b", "foreign-offer", 1), headers); err == nil {
		t.Fatal("foreign offer accepted")
	}
	if err := checkNarrowing(entry, body, http.Header{}); err == nil {
		t.Fatal("cacheable personal response accepted")
	}
}

func expandedFixture() narrowingFixture {
	fixture := narrowingFixture{People: 10000, Offers: 100000}
	read := false
	for _, class := range []string{"public", "public-authenticated", "saved", "inquired"} {
		histories := []string{"dense", "sparse", "zero"}
		if class == "public" {
			histories = []string{"none"}
		}
		for _, history := range histories {
			entry := narrowingCase{ID: class + "-" + history, Label: class + "/" + history, Class: class, History: history, Path: "/offers", ExpectedSlugs: []string{}}
			if class != "public" {
				entry.Token = "private-token"
				entry.PersonID = "person"
				entry.ExpectedUnread = &read
			}
			if class == "saved" || class == "inquired" {
				entry.Path += "?scope=" + class
			}
			fixture.Cases = append(fixture.Cases, entry)
		}
	}
	return fixture
}

func TestNarrowingRequiresAuthenticatedAndRealZeroStrata(t *testing.T) {
	fixture := expandedFixture()
	write := func(fixture narrowingFixture) string {
		t.Helper()
		raw, err := json.Marshal(fixture)
		if err != nil {
			t.Fatal(err)
		}
		path := t.TempDir() + "/private-fixture.json"
		if err := os.WriteFile(path, raw, 0600); err != nil {
			t.Fatal(err)
		}
		return path
	}
	if _, err := loadNarrowing(write(fixture), "public-authenticated", 100, 42); err != nil {
		t.Fatal(err)
	}
	for _, mutate := range []func(*narrowingFixture){
		func(f *narrowingFixture) { f.Cases = f.Cases[:len(f.Cases)-1] },
		func(f *narrowingFixture) { f.Cases[1].PersonID = "" },
		func(f *narrowingFixture) { f.Cases[1].ExpectedUnread = nil },
		func(f *narrowingFixture) { f.Cases[len(f.Cases)-1].ExpectedSlugs = []string{"unexpected-member"} },
	} {
		invalid := expandedFixture()
		mutate(&invalid)
		if _, err := loadNarrowing(write(invalid), "mixed", 100, 42); err == nil {
			t.Fatal("incomplete or false zero-history truth accepted")
		}
	}
}

func TestAuthenticatedNarrowingIssuesSeparateUnreadAndReportsJourneyLatency(t *testing.T) {
	read := false
	entry := narrowingCase{ID: "auth-zero", Label: "public-authenticated/zero/all/first", Class: "public-authenticated", History: "zero", Path: "/offers", Token: "private-token", PersonID: "person", ExpectedUnread: &read, ExpectedSlugs: []string{}}
	calls := 0
	client := &http.Client{Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
		calls++
		cookie, err := req.Cookie("sid")
		if err != nil || cookie.Value != "private-token" {
			t.Fatal("authenticated case lost session")
		}
		headers := http.Header{"Cache-Control": []string{"private, no-store"}, "Vary": []string{"Cookie"}}
		content := "<main>No offers</main>"
		if req.URL.Path == "/unread-status" {
			content = `{"person_id":"person","has_unread":false}`
		}
		return &http.Response{StatusCode: 200, Header: headers, Body: io.NopCloser(strings.NewReader(content))}, nil
	})}
	collector := &stats{samples: make(map[string][]time.Duration)}
	if err := narrowingCaseJourney(context.Background(), client, "https://haven.test", entry, true, collector); err != nil {
		t.Fatal(err)
	}
	if calls != 2 || collector.requests.Load() != 2 {
		t.Fatalf("expected page+status, got %d", calls)
	}
	summary := makeSummary("v07", time.Second, 1, 1, 1, 1, 0, 0, collector)
	if summary.Journeys[entry.Label].Count != 1 {
		t.Fatal("journey latency missing")
	}
	if len(summary.Routes) != 2 {
		t.Fatal("page and unread timings were not separated")
	}
	if summary.Semantic[entry.ID].Passed != 1 {
		t.Fatal("page semantic check missing")
	}
}
