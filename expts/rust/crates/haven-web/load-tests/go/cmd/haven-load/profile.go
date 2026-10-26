package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"os"
	"time"
)

type profileStage struct {
	rate     int
	duration time.Duration
}

type stageSummary struct {
	Rate    int        `json:"journeys_per_second"`
	Summary runSummary `json:"summary"`
	Breach  string     `json:"breach,omitempty"`
}

func runProfile(ctx context.Context, client *http.Client, baseURL, scenario string, sessions []session, workers int, collector *stats) runSummary {
	stages := profileStages(scenario)
	var scheduledTotal, startedTotal, completedTotal, droppedTotal, failedTotal uint64
	var profileFailed bool
	stageResults := make([]stageSummary, 0, len(stages))
	for _, stage := range stages {
		stageStarted := time.Now()
		stageStats := &stats{samples: make(map[string][]time.Duration)}
		scheduled, started, completed, dropped, failed := run(ctx, client, baseURL, "steady", sessions, stage.rate, stage.duration, workers, 0, stageStats)
		summary := makeSummary(scenario, stage.duration, stage.rate, scheduled, started, completed, dropped, failed, stageStats)
		summary.Duration = time.Since(stageStarted)
		if summary.Duration > 0 {
			summary.RequestsPerSecond = float64(summary.Requests) / summary.Duration.Seconds()
		}
		stageResult := stageSummary{Rate: stage.rate, Summary: summary}
		if scenario == "capacity" {
			stageResult.Breach = capacityBreach(summary)
		} else if scenario == "spike" {
			stageResult.Breach = spikeBreach(summary, len(stageResults))
		}
		stageResults = append(stageResults, stageResult)
		scheduledTotal += scheduled
		startedTotal += started
		completedTotal += completed
		droppedTotal += dropped
		failedTotal += failed
		if stageResult.Breach != "" {
			profileFailed = true
		}
		collector.merge(stageStats)
		printSummary(summary)
		if scenario == "capacity" && stageResult.Breach != "" {
			fmt.Printf("capacity stopped at %d journeys/s: %s\n", stage.rate, stageResult.Breach)
			break
		}
	}
	result := makeSummary(scenario, 0, stages[0].rate, scheduledTotal, startedTotal, completedTotal, droppedTotal, failedTotal, collector)
	result.Stages = stageResults
	result.ProfileFailed = profileFailed
	return result
}

func capacityBreach(summary runSummary) string {
	if summary.Dropped > 0 {
		return "load generator dropped scheduled journeys"
	}
	if summary.Requests == 0 {
		return "no requests completed"
	}
	if requestErrorRate(summary) > .01 {
		return "request error rate exceeded 1%"
	}
	for _, route := range summary.Routes {
		if route.P95 > 250*time.Millisecond {
			return fmt.Sprintf("%s p95 exceeded 250ms", route.Route)
		}
	}
	return ""
}

func spikeBreach(summary runSummary, stage int) string {
	if stage == 0 {
		return ""
	}
	if summary.Dropped > 0 {
		return "load generator dropped scheduled journeys"
	}
	if summary.Requests == 0 {
		return "no requests completed"
	}
	latencyLimit := 1000 * time.Millisecond
	errorLimit := .01
	if stage == 2 {
		latencyLimit = 100 * time.Millisecond
		errorLimit = .001
	}
	if requestErrorRate(summary) >= errorLimit {
		return fmt.Sprintf("request error rate reached %.2f%% limit", errorLimit*100)
	}
	for _, route := range summary.Routes {
		if route.P95 >= latencyLimit {
			return fmt.Sprintf("%s p95 reached %s limit", route.Route, latencyLimit)
		}
	}
	return ""
}

func requestErrorRate(summary runSummary) float64 {
	if summary.Checks > 0 {
		return float64(summary.Checks-summary.ChecksPassed) / float64(summary.Checks)
	}
	if summary.Requests == 0 {
		return 1
	}
	return float64(summary.Failed) / float64(summary.Requests)
}

func profileStages(scenario string) []profileStage {
	if scenario == "spike" {
		return []profileStage{{rate: 100, duration: time.Minute}, {rate: 2000, duration: time.Minute}, {rate: 100, duration: time.Minute}}
	}
	rates := []int{100, 250, 500, 750, 1000, 1500, 2000, 3000, 5000}
	stages := make([]profileStage, 0, len(rates))
	for _, rate := range rates {
		stages = append(stages, profileStage{rate: rate, duration: 2 * time.Minute})
	}
	return stages
}

func needsSessions(scenario string) bool {
	return scenario == "warmup" || scenario == "steady" || scenario == "authorization" || scenario == "v06" || scenario == "spike" || scenario == "capacity"
}

func validateSessions(scenario string, sessions []session) error {
	switch scenario {
	case "warmup":
		if len(sessions) == 0 || sessions[0].Session == "" {
			return errors.New("warmup requires seeded session tokens")
		}
	case "steady", "spike", "capacity":
		if len(sessions) == 0 || len(sessions[0].OfferIDs) == 0 || sessions[0].Handle == "" {
			return errors.New("steady workloads require session tokens, owner offer IDs, and handles; regenerate sessions.json")
		}
	case "authorization":
		if len(sessions) < 2 || len(sessions[0].OfferIDs) == 0 || len(sessions[1].OfferIDs) == 0 {
			return errors.New("authorization requires two seeded users with offers")
		}
	}
	return nil
}

func loadSessions(path string) ([]session, error) {
	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("open sessions fixture %q: %w", path, err)
	}
	defer file.Close()
	var sessions []session
	if err := json.NewDecoder(file).Decode(&sessions); err != nil {
		return nil, fmt.Errorf("decode sessions fixture: %w", err)
	}
	if len(sessions) == 0 {
		return nil, errors.New("sessions fixture is empty")
	}
	return sessions, nil
}
