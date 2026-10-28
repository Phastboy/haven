package main

import (
	"net/http"
	"strings"
	"testing"
)

func TestNarrowingMixAndHistoryWeights(t *testing.T) {
	config := &narrowingConfig{class: "mixed", groups: make(map[string][]narrowingCase)}
	for _, class := range []string{"public", "saved", "inquired"} {
		histories := []string{"dense", "sparse"}
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
	for i := 0; i < 200; i++ {
		entry, _ := config.choose(i)
		counts[entry.Class]++
		if entry.Class != "public" {
			counts[entry.Class+"/"+entry.History]++
		}
	}
	for key, want := range map[string]int{"public": 140, "saved": 30, "inquired": 30, "saved/dense": 15, "saved/sparse": 15, "inquired/dense": 15, "inquired/sparse": 15} {
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
