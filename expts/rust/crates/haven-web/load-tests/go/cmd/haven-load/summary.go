package main

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"sort"
	"time"
)

func (collector *stats) record(entry sample, passed bool) {
	collector.mu.Lock()
	collector.samples[entry.Route] = append(collector.samples[entry.Route], entry.Duration)
	collector.mu.Unlock()
	collector.requests.Add(1)
	collector.checks.Add(1)
	if passed {
		collector.passed.Add(1)
	} else {
		collector.failures.Add(1)
	}
}

type routeSummary struct {
	Route string        `json:"route"`
	Count int           `json:"count"`
	P50   time.Duration `json:"p50_ns"`
	P95   time.Duration `json:"p95_ns"`
	P99   time.Duration `json:"p99_ns"`
}

type runSummary struct {
	Scenario                string         `json:"scenario"`
	Duration                time.Duration  `json:"duration_ns"`
	TargetJourneysPerSecond int            `json:"target_journeys_per_second"`
	Scheduled               uint64         `json:"scheduled"`
	Started                 uint64         `json:"started"`
	Completed               uint64         `json:"completed"`
	Dropped                 uint64         `json:"dropped"`
	Failed                  uint64         `json:"failed"`
	Requests                uint64         `json:"requests"`
	RequestsPerSecond       float64        `json:"requests_per_second"`
	ChecksPassed            uint64         `json:"checks_passed"`
	Checks                  uint64         `json:"checks"`
	Routes                  []routeSummary `json:"routes"`
	Stages                  []stageSummary `json:"stages,omitempty"`
	ProfileFailed           bool           `json:"profile_failed,omitempty"`
}

func (collector *stats) merge(other *stats) {
	other.mu.Lock()
	defer other.mu.Unlock()
	collector.mu.Lock()
	defer collector.mu.Unlock()
	for route, values := range other.samples {
		collector.samples[route] = append(collector.samples[route], values...)
	}
	collector.requests.Add(other.requests.Load())
	collector.failures.Add(other.failures.Load())
	collector.checks.Add(other.checks.Load())
	collector.passed.Add(other.passed.Load())
}

func makeSummary(scenario string, duration time.Duration, rate int, scheduled, started, completed, dropped, failed uint64, collector *stats) runSummary {
	summary := runSummary{
		Scenario:                scenario,
		Duration:                duration,
		TargetJourneysPerSecond: rate,
		Scheduled:               scheduled,
		Started:                 started,
		Completed:               completed,
		Dropped:                 dropped,
		Failed:                  failed,
		Requests:                collector.requests.Load(),
		ChecksPassed:            collector.passed.Load(),
		Checks:                  collector.checks.Load(),
	}
	if seconds := duration.Seconds(); seconds > 0 {
		summary.RequestsPerSecond = float64(summary.Requests) / seconds
	}
	collector.mu.Lock()
	defer collector.mu.Unlock()
	routes := make([]string, 0, len(collector.samples))
	for route := range collector.samples {
		routes = append(routes, route)
	}
	sort.Strings(routes)
	for _, route := range routes {
		values := collector.samples[route]
		sort.Slice(values, func(i, j int) bool { return values[i] < values[j] })
		summary.Routes = append(summary.Routes, routeSummary{
			Route: route,
			Count: len(values),
			P50:   percentile(values, .50),
			P95:   percentile(values, .95),
			P99:   percentile(values, .99),
		})
	}
	return summary
}

func printSummary(summary runSummary) {
	fmt.Printf("scenario=%s elapsed=%s target_journeys_per_second=%d scheduled=%d started=%d completed=%d dropped=%d failed=%d requests=%d actual_requests_per_second=%.2f checks_passed=%d/%d\n",
		summary.Scenario, summary.Duration.Round(time.Millisecond), summary.TargetJourneysPerSecond,
		summary.Scheduled, summary.Started, summary.Completed, summary.Dropped, summary.Failed, summary.Requests,
		summary.RequestsPerSecond, summary.ChecksPassed, summary.Checks)
	for _, route := range summary.Routes {
		fmt.Printf("route=%s count=%d p50=%s p95=%s p99=%s\n", route.Route, route.Count,
			route.P50, route.P95, route.P99)
	}
}

func writeSummary(path string, summary runSummary) error {
	file, err := os.Create(path)
	if err != nil {
		return fmt.Errorf("create summary file: %w", err)
	}
	defer file.Close()
	encoder := json.NewEncoder(file)
	encoder.SetIndent("", "  ")
	return encoder.Encode(summary)
}

func percentile(sorted []time.Duration, percentile float64) time.Duration {
	if len(sorted) == 0 {
		return 0
	}
	index := int(math.Ceil(float64(len(sorted))*percentile)) - 1
	index = max(index, 0)
	return sorted[index]
}

func envOr(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}
