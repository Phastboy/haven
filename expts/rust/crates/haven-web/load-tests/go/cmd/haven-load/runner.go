package main

import (
	"context"
	"net/http"
	"sync"
	"sync/atomic"
	"time"
)

func run(ctx context.Context, client *http.Client, baseURL, scenario string, sessions []session, rate int, duration time.Duration, workers, maxIterations int, collector *stats) (uint64, uint64, uint64, uint64, uint64) {
	queue := make(chan job, workers)
	var scheduled, started, dropped, completed, failed atomic.Uint64
	var workerGroup sync.WaitGroup
	for workerID := range workers {
		workerGroup.Add(1)
		go func(workerID int) {
			defer workerGroup.Done()
			for work := range queue {
				started.Add(1)
				if err := execute(ctx, client, baseURL, scenario, sessions, work.index, workerID, collector); err != nil {
					failed.Add(1)
				}
				completed.Add(1)
			}
		}(workerID)
	}

	period := time.Second / time.Duration(rate)
	if period < time.Microsecond {
		period = time.Microsecond
	}
	deadline := time.Now().Add(duration)
	for index, next := 0, time.Now(); time.Now().Before(deadline); index, next = index+1, next.Add(period) {
		if maxIterations > 0 && index >= maxIterations {
			break
		}
		scheduled.Add(1)
		if delay := time.Until(next); delay > 0 {
			timer := time.NewTimer(delay)
			deadlineTimer := time.NewTimer(time.Until(deadline))
			select {
			case <-ctx.Done():
				timer.Stop()
				deadlineTimer.Stop()
				close(queue)
				workerGroup.Wait()
				return scheduled.Load(), started.Load(), completed.Load(), dropped.Load(), failed.Load()
			case <-deadlineTimer.C:
				timer.Stop()
				close(queue)
				workerGroup.Wait()
				return scheduled.Load(), started.Load(), completed.Load(), dropped.Load(), failed.Load()
			case <-timer.C:
				deadlineTimer.Stop()
			}
		}
		select {
		case queue <- job{index: index}:
		default:
			dropped.Add(1)
		}
	}
	close(queue)
	workerGroup.Wait()
	return scheduled.Load(), started.Load(), completed.Load(), dropped.Load(), failed.Load()
}

func execute(ctx context.Context, client *http.Client, baseURL, scenario string, sessions []session, index, workerID int, collector *stats) error {
	actor := session{}
	if len(sessions) > 0 {
		actor = sessions[index%len(sessions)]
	}
	var err error
	switch scenario {
	case "smoke":
		err = get(ctx, client, baseURL, "/health", "", 200, collector)
	case "warmup":
		err = get(ctx, client, baseURL, "/auth/sign-in", "", 200, collector)
		if err == nil {
			err = get(ctx, client, baseURL, "/offers", actor.Session, 200, collector)
		}
	case "steady":
		err = steadyJourney(ctx, client, baseURL, actor, index, collector)
	case "authorization":
		err = authorizationJourney(ctx, client, baseURL, sessions, index, collector)
	case "v05":
		err = v05Journey(ctx, client, baseURL, collector)
	case "v06":
		err = v06Journey(ctx, client, baseURL, sessions, workerID, collector)
	}
	return err
}
