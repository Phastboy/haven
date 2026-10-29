package main

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"html"
	"io"
	"net/http"
	"regexp"
	"strings"
	"time"
)

var conversationLocation = regexp.MustCompile(`^/conversations/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$`)

// Each worker owns a responder, maker and outsider for the entire run. These
// actors must not be used by any other concurrently mutating scenario.
func validateMessagingSessions(sessions []session, workers int) error {
	if workers < 1 || len(sessions) < workers*3 {
		return errors.New("messaging requires three isolated people per worker")
	}
	people, tokens := map[string]bool{}, map[string]bool{}
	for index, actor := range sessions[:workers*3] {
		if actor.UserID == "" || actor.Session == "" || actor.Handle == "" || people[actor.UserID] || tokens[actor.Session] {
			return errors.New("messaging people and session credentials must be complete and distinct")
		}
		people[actor.UserID], tokens[actor.Session] = true, true
		if index%3 == 1 && (len(actor.OfferSlugs) == 0 || actor.OfferSlugs[0] == "") {
			return errors.New("messaging makers require a visible seeded offer slug")
		}
	}
	return nil
}

func messagingJourney(ctx context.Context, client *http.Client, baseURL string, sessions []session, workerID, index int, collector *stats) (result error) {
	defer func() {
		collector.mu.Lock()
		defer collector.mu.Unlock()
		if collector.semantic == nil {
			collector.semantic = make(map[string]semanticCount)
		}
		count := collector.semantic["messaging/round-trip"]
		count.Checked++
		if result == nil {
			count.Passed++
		}
		collector.semantic["messaging/round-trip"] = count
	}()
	start := workerID * 3
	if start < 0 || len(sessions) < start+3 {
		return errors.New("messaging fixture group is incomplete")
	}
	responder, maker, outsider := sessions[start], sessions[start+1], sessions[start+2]
	if len(maker.OfferSlugs) == 0 {
		return errors.New("messaging maker has no offer")
	}
	inquiry := scenarioTitle("Inquiry", responder, index)
	thread, err := requestWithLocation(ctx, client, baseURL, "/offers/"+maker.OfferSlugs[0]+"/message", responder.Session, map[string]string{"body": inquiry}, 303, collector)
	if err != nil {
		return err
	}
	if !conversationLocation.MatchString(thread) {
		return errors.New("inquiry returned an invalid conversation destination")
	}
	if err := checkUnread(ctx, client, baseURL, maker, true, collector); err != nil {
		return err
	}
	if err := checkThread(ctx, client, baseURL, thread, responder, []string{inquiry}, collector); err != nil {
		return err
	}
	if err := checkUnread(ctx, client, baseURL, responder, false, collector); err != nil {
		return err
	}
	inbox, err := request(ctx, client, baseURL, "/conversations", maker.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	if !strings.Contains(inbox, thread) || !strings.Contains(inbox, html.EscapeString(responder.Handle)) {
		return errors.New("maker inbox lost inquiry identity or conversation")
	}
	if err := checkUnread(ctx, client, baseURL, maker, true, collector); err != nil {
		return err
	}
	if err := checkThread(ctx, client, baseURL, thread, maker, []string{inquiry}, collector); err != nil {
		return err
	}
	if err := checkUnread(ctx, client, baseURL, maker, false, collector); err != nil {
		return err
	}
	reply := scenarioTitle("Reply", maker, index)
	if err := post(ctx, client, baseURL, thread+"/messages", maker.Session, map[string]string{"body": reply}, 303, collector); err != nil {
		return err
	}
	if err := checkUnread(ctx, client, baseURL, responder, true, collector); err != nil {
		return err
	}
	if err := checkThread(ctx, client, baseURL, thread, responder, []string{inquiry, reply}, collector); err != nil {
		return err
	}
	if err := checkUnread(ctx, client, baseURL, responder, false, collector); err != nil {
		return err
	}
	if err := get(ctx, client, baseURL, thread, outsider.Session, 404, collector); err != nil {
		return err
	}
	if err := post(ctx, client, baseURL, thread+"/messages", outsider.Session, map[string]string{"body": "Unauthorized outsider message"}, 404, collector); err != nil {
		return err
	}
	content, err := request(ctx, client, baseURL, thread, responder.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	if strings.Contains(content, "Unauthorized outsider message") {
		return errors.New("outsider mutated the thread")
	}
	if err := checkUnread(ctx, client, baseURL, responder, false, collector); err != nil {
		return err
	}
	return get(ctx, client, baseURL, "/unread-status", "", 401, collector)
}

func checkThread(ctx context.Context, client *http.Client, baseURL, path string, actor session, bodies []string, collector *stats) error {
	content, err := request(ctx, client, baseURL, path, actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	position := 0
	for _, body := range bodies {
		found := strings.Index(content[position:], html.EscapeString(body))
		if found < 0 {
			return errors.New("thread lost message body or chronological order")
		}
		position += found + len(html.EscapeString(body))
	}
	return nil
}

// Unread is a separate request with its own timing and semantic/private checks.
func checkUnread(ctx context.Context, client *http.Client, baseURL string, actor session, expected bool, collector *stats) (result error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, strings.TrimRight(baseURL, "/")+"/unread-status", nil)
	if err != nil {
		return err
	}
	req.Header.Set("Cookie", "sid="+actor.Session)
	started := time.Now()
	statusCode := 0
	defer func() {
		entry := sample{Route: "/unread-status", Duration: time.Since(started), Status: statusCode}
		if result != nil {
			entry.Err = result.Error()
		}
		collector.record(entry, result == nil)
	}()
	response, err := client.Do(req)
	if err != nil {
		return errors.New("unread request failed")
	}
	statusCode = response.StatusCode
	defer response.Body.Close()
	raw, err := io.ReadAll(io.LimitReader(response.Body, 4097))
	if err != nil || len(raw) > 4096 {
		return errors.New("unread response body invalid")
	}
	if response.StatusCode != 200 || response.Header.Get("Cache-Control") != "private, no-store" || !strings.Contains(strings.ToLower(response.Header.Get("Vary")), "cookie") {
		return errors.New("unread status or privacy headers invalid")
	}
	var status struct {
		PersonID  string `json:"person_id"`
		HasUnread *bool  `json:"has_unread"`
	}
	if json.Unmarshal(raw, &status) != nil || status.PersonID != actor.UserID || status.HasUnread == nil || *status.HasUnread != expected {
		return fmt.Errorf("unread semantic check failed (expected %t)", expected)
	}
	return nil
}
