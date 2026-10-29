package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"testing"
	"time"
)

func messagingSessions() []session {
	var actors []session
	for i := 0; i < 6; i++ {
		actors = append(actors, session{UserID: fmt.Sprintf("person-%d", i), Handle: fmt.Sprintf("handle-%d", i), Session: fmt.Sprintf("token-%d", i), OfferSlugs: []string{fmt.Sprintf("offer-%d", i)}})
	}
	return actors
}

func TestMessagingFixtureRejectsSharedOrIncompletePeople(t *testing.T) {
	for _, change := range []func([]session) []session{
		func(s []session) []session { return s[:5] },
		func(s []session) []session { s[3].UserID = s[0].UserID; return s },
		func(s []session) []session { s[3].Session = s[0].Session; return s },
		func(s []session) []session { s[1].OfferSlugs = nil; return s },
		func(s []session) []session { s[0].Session = ""; return s },
	} {
		if err := validateMessagingSessions(change(messagingSessions()), 2); err == nil {
			t.Fatal("invalid shared/incomplete fixture accepted")
		}
	}
	if err := validateMessagingSessions(messagingSessions(), 2); err != nil {
		t.Fatal(err)
	}
	if err := validate("https://haven.test", "messaging", 1, time.Second, 2); err != nil {
		t.Fatal(err)
	}
}

// Stateful transport verifies the runner's sequence and detects false-200
// semantics; it is not evidence of real application or sustained-load behavior.
func messagingClient(t *testing.T, actors []session, worker int, fault string) *http.Client {
	t.Helper()
	responder, maker, outsider := actors[worker*3], actors[worker*3+1], actors[worker*3+2]
	thread := "/conversations/00000000-0000-0000-0000-000000000001"
	unread := map[string]bool{}
	inquiry, reply := "", ""
	return &http.Client{CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }, Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
		cookie, _ := req.Cookie("sid")
		actor := session{}
		if cookie != nil {
			for _, candidate := range actors {
				if candidate.Session == cookie.Value {
					actor = candidate
				}
			}
		}
		if actor.Session != "" && actor.UserID != responder.UserID && actor.UserID != maker.UserID && actor.UserID != outsider.UserID {
			t.Error("worker touched another worker's actor")
		}
		status, content := 200, ""
		headers := http.Header{}
		switch {
		case req.URL.Path == "/unread-status":
			if actor.Session == "" {
				status = 401
				break
			}
			headers.Set("Cache-Control", "private, no-store")
			headers.Set("Vary", "Cookie")
			person := actor.UserID
			if fault == "wrong-person" {
				person = "another-person"
			}
			if fault == "public-status" {
				headers.Set("Cache-Control", "public")
			}
			raw, _ := json.Marshal(map[string]any{"person_id": person, "has_unread": unread[actor.UserID]})
			content = string(raw)
		case req.URL.Path == "/offers/"+maker.OfferSlugs[0]+"/message":
			if req.Method != "POST" || actor.UserID != responder.UserID {
				t.Fatal("incorrect inquiry actor/method")
			}
			if err := req.ParseForm(); err != nil {
				t.Fatal(err)
			}
			inquiry = req.Form.Get("body")
			unread[maker.UserID] = true
			status = 303
			headers.Set("Location", thread)
		case req.URL.Path == thread+"/messages":
			if actor.UserID == outsider.UserID {
				status = 404
				break
			}
			if actor.UserID != maker.UserID || req.Method != "POST" {
				t.Fatal("incorrect reply actor/method")
			}
			if err := req.ParseForm(); err != nil {
				t.Fatal(err)
			}
			reply = req.Form.Get("body")
			unread[responder.UserID] = true
			status = 303
			headers.Set("Location", thread)
		case req.URL.Path == thread:
			if actor.UserID == outsider.UserID {
				status = 404
				break
			}
			unread[actor.UserID] = false
			content = inquiry + " " + reply
			if fault == "missing-body" {
				content = "thread is empty"
			}
		case req.URL.Path == "/conversations":
			content = thread + " " + responder.Handle
		default:
			t.Fatalf("unexpected request %s", req.URL.Path)
		}
		return &http.Response{StatusCode: status, Header: headers, Body: io.NopCloser(strings.NewReader(content))}, nil
	})}
}

func TestMessagingJourneyChecksTransitionsAndIsolatedWorker(t *testing.T) {
	actors := messagingSessions()
	for worker := 0; worker < 2; worker++ {
		collector := &stats{samples: make(map[string][]time.Duration)}
		err := messagingJourney(context.Background(), messagingClient(t, actors, worker, ""), "https://haven.test", actors, worker, 1, collector)
		if err != nil {
			t.Fatal(err)
		}
		if collector.requests.Load() != 17 {
			t.Fatalf("got %d requests, want 17", collector.requests.Load())
		}
		if got := collector.semantic["messaging/round-trip"]; got.Checked != 1 || got.Passed != 1 {
			t.Fatalf("semantic counts = %#v", got)
		}
		summary := makeSummary("messaging", time.Second, 1, 1, 1, 1, 0, 0, collector)
		if summary.Semantic["messaging/round-trip"].Passed != 1 {
			t.Fatal("messaging semantics missing from JSON summary")
		}
		for route := range collector.samples {
			if strings.Contains(route, "00000000") {
				t.Fatal("conversation ID leaked into endpoint labels")
			}
		}
	}
}

func TestMessagingJourneyRejectsIncorrectSuccessfulResponses(t *testing.T) {
	for _, fault := range []string{"wrong-person", "public-status", "missing-body"} {
		t.Run(fault, func(t *testing.T) {
			actors := messagingSessions()
			collector := &stats{samples: make(map[string][]time.Duration)}
			err := messagingJourney(context.Background(), messagingClient(t, actors, 0, fault), "https://haven.test", actors, 0, 1, collector)
			if err == nil {
				t.Fatal("false success accepted")
			}
			if got := collector.semantic["messaging/round-trip"]; got.Checked != 1 || got.Passed != 0 {
				t.Fatalf("semantic counts = %#v", got)
			}
		})
	}
}

func TestOfferWriteSuppliesRequiredCategory(t *testing.T) {
	actor := session{UserID: "person", Session: "token"}
	title := ""
	client := &http.Client{CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse }, Transport: roundTripFunc(func(req *http.Request) (*http.Response, error) {
		status, content := 200, ""
		headers := http.Header{}
		if req.Method == "POST" && (req.URL.Path == "/profile/offers/new" || strings.HasSuffix(req.URL.Path, "/edit")) {
			if err := req.ParseForm(); err != nil {
				t.Fatal(err)
			}
			if req.Form.Get("category") != "other" {
				t.Fatal("required category omitted")
			}
			title = req.Form.Get("title")
			status = 303
			headers.Set("Location", "/profile/offers/created")
		} else if req.URL.Path == "/profile/offers/new" {
			content = `<input name="idempotency_key" value="key">`
		} else if strings.HasSuffix(req.URL.Path, "/delete") {
			status = 303
			title = ""
		} else if req.URL.Path == "/profile/offers/created" {
			if title == "" {
				status = 404
			} else {
				content = title + " Free"
			}
		}
		return &http.Response{StatusCode: status, Header: headers, Body: io.NopCloser(strings.NewReader(content))}, nil
	})}
	collector := &stats{samples: make(map[string][]time.Duration)}
	if err := offerWriteJourney(context.Background(), client, "https://haven.test", actor, 1, collector); err != nil {
		t.Fatal(err)
	}
}
