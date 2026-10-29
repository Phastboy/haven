package main

import (
	"context"
	"crypto/tls"
	"errors"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

type session struct {
	UserID     string   `json:"user_id"`
	Handle     string   `json:"handle"`
	Session    string   `json:"session_token"`
	OfferIDs   []string `json:"offer_ids"`
	OfferSlugs []string `json:"offer_slugs"`
}

type sample struct {
	Route    string        `json:"route"`
	Duration time.Duration `json:"duration_ns"`
	Status   int           `json:"status"`
	Err      string        `json:"error,omitempty"`
}

type job struct {
	index int
}

type stats struct {
	narrowing      *narrowingConfig
	semantic       map[string]semanticCount
	journeySamples map[string][]time.Duration
	mu             sync.Mutex
	samples        map[string][]time.Duration
	requests       atomic.Uint64
	failures       atomic.Uint64
	checks         atomic.Uint64
	passed         atomic.Uint64
}

func main() {
	baseURL := flag.String("base-url", envOr("BASE_URL", "http://127.0.0.1:3000"), "Haven base URL")
	scenario := flag.String("scenario", "smoke", "smoke, warmup, steady, authorization, v05, v06, v07, messaging, spike, or capacity")
	sessionsPath := flag.String("sessions", "../data/sessions.json", "load-test-support session JSON")
	rate := flag.Int("rate", 1, "open arrival rate in journeys per second")
	duration := flag.Duration("duration", time.Second, "test duration")
	workers := flag.Int("workers", 32, "maximum concurrent journey workers")
	maxIterations := flag.Int("iterations", 0, "stop after this many scheduled journeys (0 uses duration)")
	summaryPath := flag.String("summary", "", "write machine-readable JSON summary")
	skipTLSVerify := flag.Bool("insecure-skip-tls-verify", false, "skip TLS verification for isolated self-signed QEMU benchmarks")
	narrowingPath := flag.String("narrowing-fixture", "", "private v0.7 expected-result fixture")
	historyClass := flag.String("history", "all", "all, dense, sparse or zero for an authenticated narrowing class")
	fixtureSweep := flag.Bool("fixture-sweep", false, "visit every selected v07 fixture case once with 100% semantic checks; not a load gate")
	queryClass := flag.String("query-class", "public", "public, public-authenticated, saved, inquired or mixed")
	semanticPercent := flag.Int("semantic-percent", 10, "stratified result checks per 100 case visits")
	sampleSeed := flag.Uint64("sample-seed", 42, "reproducible semantic sampling seed")
	profileTarget := flag.String("profile-target", "steady", "steady or v07 workload for spike/capacity")
	flag.Parse()

	if err := validate(*baseURL, *scenario, *rate, *duration, *workers); err != nil {
		log.Fatal(err)
	}
	fixture, err := loadSessions(*sessionsPath)
	if err != nil && needsSessions(*scenario) && *profileTarget != "v07" {
		log.Fatal(err)
	}
	if *profileTarget != "v07" {
		if err := validateSessions(*scenario, fixture); err != nil {
			log.Fatal(err)
		}
	}
	if *profileTarget != "steady" && *profileTarget != "v07" {
		log.Fatal("profile-target must be steady or v07")
	}
	if *scenario == "v06" && len(fixture) < *workers*22 {
		log.Fatalf("v06 needs %d seeded users for isolated pagination fixtures; found %d", *workers*22, len(fixture))
	}

	if *scenario == "messaging" {
		if err := validateMessagingSessions(fixture, *workers); err != nil {
			log.Fatal(err)
		}
	}

	client := &http.Client{
		Timeout: 15 * time.Second,
		Transport: &http.Transport{
			MaxIdleConns:        *workers * 2,
			MaxIdleConnsPerHost: *workers,
			MaxConnsPerHost:     *workers,
			IdleConnTimeout:     90 * time.Second,
			TLSClientConfig:     &tls.Config{InsecureSkipVerify: *skipTLSVerify}, // only for isolated QEMU's self-signed endpoint
		},
		CheckRedirect: func(_ *http.Request, _ []*http.Request) error { return http.ErrUseLastResponse },
	}
	collector := &stats{samples: make(map[string][]time.Duration)}
	if *scenario == "v07" || *profileTarget == "v07" {
		collector.narrowing, err = loadNarrowing(*narrowingPath, *queryClass, *semanticPercent, *sampleSeed)
		if err != nil {
			log.Fatal(err)
		}
		if *historyClass != "all" {
			if (*historyClass != "dense" && *historyClass != "sparse" && *historyClass != "zero") || *queryClass == "public" || *queryClass == "mixed" {
				log.Fatal("history filter requires dense/sparse/zero and a single authenticated class")
			}
		}
		collector.narrowing.history = *historyClass
	}
	if *fixtureSweep && *scenario != "v07" {
		log.Fatal("fixture-sweep requires scenario v07")
	}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	started := time.Now()
	if *scenario == "spike" || *scenario == "capacity" {
		summary := runProfile(ctx, client, *baseURL, *scenario, *profileTarget, fixture, *workers, collector)
		summary.Duration = time.Since(started)
		if summary.Duration > 0 {
			summary.RequestsPerSecond = float64(summary.Requests) / summary.Duration.Seconds()
		}
		printSummary(summary)
		if *summaryPath != "" {
			if err := writeSummary(*summaryPath, summary); err != nil {
				log.Fatal(err)
			}
		}
		if profileExitCode(summary) != 0 {
			os.Exit(1)
		}
		return
	}
	var scheduled, startedCount, completed, dropped, failed uint64
	if *fixtureSweep {
		for _, entry := range collector.narrowing.fixture.Cases {
			if (*queryClass != "mixed" && entry.Class != *queryClass) || (*historyClass != "all" && entry.History != *historyClass) {
				continue
			}
			scheduled++
			startedCount++
			if err := narrowingCaseJourney(ctx, client, *baseURL, entry, true, collector); err != nil {
				failed++
			}
			completed++
		}
	} else {
		scheduled, startedCount, completed, dropped, failed = run(ctx, client, *baseURL, *scenario, fixture, *rate, *duration, *workers, *maxIterations, collector)
	}
	summary := makeSummary(*scenario, time.Since(started), *rate, scheduled, startedCount, completed, dropped, failed, collector)
	if *fixtureSweep {
		summary.Scenario = "v07-correctness-sweep"
		summary.TargetJourneysPerSecond = 0
		summary.SemanticPercent = 100
	}
	printSummary(summary)
	if *summaryPath != "" {
		if err := writeSummary(*summaryPath, summary); err != nil {
			log.Fatal(err)
		}
	}
	if failed != 0 || dropped != 0 {
		os.Exit(1)
	}
}

func profileExitCode(summary runSummary) int {
	if summary.ProfileFailed || summary.Failed > 0 || summary.Dropped > 0 {
		return 1
	}
	return 0
}

func validate(baseURL, scenario string, rate int, duration time.Duration, workers int) error {
	if !strings.HasPrefix(baseURL, "http://") && !strings.HasPrefix(baseURL, "https://") {
		return errors.New("base-url must use http or https")
	}
	if scenario != "smoke" && scenario != "warmup" && scenario != "steady" && scenario != "authorization" && scenario != "v05" && scenario != "v06" && scenario != "v07" && scenario != "messaging" && scenario != "spike" && scenario != "capacity" {
		return fmt.Errorf("unknown scenario %q", scenario)
	}
	if rate < 1 || workers < 1 || duration <= 0 {
		return errors.New("rate, workers, and duration must be positive")
	}
	return nil
}
