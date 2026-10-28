package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"hash/fnv"
	"io"
	"net/http"
	"os"
	"reflect"
	"regexp"
	"strings"
	"time"
)

type narrowingCase struct {
	ID            string   `json:"id"`
	Label         string   `json:"label"`
	Class         string   `json:"class"`
	History       string   `json:"history"`
	Path          string   `json:"path"`
	Token         string   `json:"session_token,omitempty"`
	ExpectedSlugs []string `json:"expected_slugs"`
	ExpectedNext  bool     `json:"expected_next"`
	Canonical     bool     `json:"canonical"`
}
type narrowingFixture struct {
	People         int             `json:"people"`
	Offers         int             `json:"offers"`
	CategoryCounts map[string]int  `json:"category_counts"`
	Cases          []narrowingCase `json:"cases"`
}
type narrowingConfig struct {
	fixture narrowingFixture
	sha     string
	class   string
	percent int
	seed    uint64
	groups  map[string][]narrowingCase
}
type semanticCount struct {
	Checked uint64 `json:"checked"`
	Passed  uint64 `json:"passed"`
}

var cardIDs = regexp.MustCompile(`<li id="offer-([^"]+)" class="listing-card">`)

func loadNarrowing(path, class string, percent int, seed uint64) (*narrowingConfig, error) {
	if percent < 1 || percent > 100 {
		return nil, errors.New("semantic-percent must be 1..100")
	}
	if class != "public" && class != "saved" && class != "inquired" && class != "mixed" {
		return nil, errors.New("query-class must be public, saved, inquired or mixed")
	}
	raw, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	var fixture narrowingFixture
	if err = json.Unmarshal(raw, &fixture); err != nil {
		return nil, err
	}
	if fixture.People < 10000 || fixture.Offers < 100000 {
		return nil, errors.New("v07 requires at least 10k people and 100k offers")
	}
	sum := sha256.Sum256(raw)
	config := &narrowingConfig{fixture: fixture, sha: hex.EncodeToString(sum[:]), class: class, percent: percent, seed: seed, groups: make(map[string][]narrowingCase)}
	seen := make(map[string]bool)
	for _, entry := range fixture.Cases {
		if entry.ID == "" || seen[entry.ID] {
			return nil, errors.New("fixture case IDs must be unique and nonempty")
		}
		seen[entry.ID] = true
		if entry.Class != "public" && entry.Class != "saved" && entry.Class != "inquired" {
			return nil, errors.New("invalid fixture class")
		}
		if entry.Path != "/offers" && !strings.HasPrefix(entry.Path, "/offers?") {
			return nil, errors.New("fixture paths must be feed-relative")
		}
		if (entry.Class == "public") != (entry.Token == "") {
			return nil, errors.New("public cases must be guests and personal cases must have a token")
		}
		if len(entry.ExpectedSlugs) > 20 {
			return nil, errors.New("expected page exceeds feed limit")
		}
		key := entry.Class
		if entry.Class != "public" {
			if entry.History != "dense" && entry.History != "sparse" {
				return nil, errors.New("personal history must be dense or sparse")
			}
			key += "/" + entry.History
		}
		config.groups[key] = append(config.groups[key], entry)
	}
	for _, key := range []string{"public", "saved/dense", "saved/sparse", "inquired/dense", "inquired/sparse"} {
		if len(config.groups[key]) == 0 {
			return nil, fmt.Errorf("missing fixture stratum %s", key)
		}
	}
	return config, nil
}

func (config *narrowingConfig) choose(index int) (narrowingCase, uint64) {
	class := config.class
	ordinal := index
	if class == "mixed" {
		cycle, bucket := index/100, index%100
		switch {
		case bucket < 70:
			class = "public"
			ordinal = cycle*70 + bucket
		case bucket < 85:
			class = "saved"
			ordinal = cycle*15 + bucket - 70
		default:
			class = "inquired"
			ordinal = cycle*15 + bucket - 85
		}
	}
	key := class
	if class != "public" {
		if ordinal%2 == 0 {
			key += "/dense"
		} else {
			key += "/sparse"
		}
		ordinal /= 2
	}
	group := config.groups[key]
	return group[ordinal%len(group)], uint64(ordinal / len(group))
}

func (config *narrowingConfig) check(entry narrowingCase, occurrence uint64) bool {
	hash := fnv.New64a()
	_, _ = hash.Write([]byte(entry.ID))
	// 73 is coprime to 100: every case is checked exactly percent times per 100 visits.
	return (occurrence*73+hash.Sum64()%100+config.seed%100)%100 < uint64(config.percent)
}

func narrowingJourney(ctx context.Context, client *http.Client, baseURL string, index int, collector *stats) error {
	config := collector.narrowing
	if config == nil {
		return errors.New("v07 needs --narrowing-fixture")
	}
	entry, occurrence := config.choose(index)
	check := config.check(entry, occurrence)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, strings.TrimRight(baseURL, "/")+entry.Path, nil)
	if err != nil {
		return err
	}
	if entry.Token != "" {
		req.Header.Set("Cookie", "sid="+entry.Token)
	}
	started := time.Now()
	response, err := client.Do(req)
	if err != nil {
		collector.record(sample{Route: entry.Label, Duration: time.Since(started), Err: err.Error()}, false)
		return err
	}
	defer response.Body.Close()
	var body string
	if check {
		raw, readErr := io.ReadAll(response.Body)
		body = string(raw)
		err = readErr
	} else {
		_, err = io.Copy(io.Discard, response.Body)
	}
	elapsed := time.Since(started)
	if err == nil && response.StatusCode != http.StatusOK {
		err = fmt.Errorf("%s returned HTTP %d", entry.ID, response.StatusCode)
	}
	if check {
		if err == nil {
			err = checkNarrowing(entry, body, response.Header)
		}
		collector.mu.Lock()
		if collector.semantic == nil {
			collector.semantic = make(map[string]semanticCount)
		}
		count := collector.semantic[entry.ID]
		count.Checked++
		if err == nil {
			count.Passed++
		}
		collector.semantic[entry.ID] = count
		collector.mu.Unlock()
	}
	message := ""
	if err != nil {
		message = err.Error()
	}
	collector.record(sample{Route: entry.Label, Duration: elapsed, Status: response.StatusCode, Err: message}, err == nil)
	return err
}

func checkNarrowing(entry narrowingCase, body string, headers http.Header) error {
	matches := cardIDs.FindAllStringSubmatch(body, -1)
	actual := make([]string, 0, len(matches))
	for _, match := range matches {
		actual = append(actual, match[1])
	}
	if !reflect.DeepEqual(actual, entry.ExpectedSlugs) {
		return fmt.Errorf("%s returned wrong offers or order", entry.ID)
	}
	if paginationHref.MatchString(body) != entry.ExpectedNext {
		return fmt.Errorf("%s returned wrong next-page state", entry.ID)
	}
	if strings.Contains(body, `rel="canonical"`) != entry.Canonical {
		return fmt.Errorf("%s returned wrong canonical state", entry.ID)
	}
	if entry.Class != "public" {
		if !strings.Contains(headers.Get("Cache-Control"), "private, no-store") || !strings.Contains(headers.Get("X-Robots-Tag"), "noindex") {
			return fmt.Errorf("%s exposed cacheable/indexable personal HTML", entry.ID)
		}
	}
	return nil
}
