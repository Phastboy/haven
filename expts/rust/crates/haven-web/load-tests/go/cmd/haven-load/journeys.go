package main

import (
	"context"
	"errors"
	"fmt"
	"html"
	"net/http"
	"regexp"
	"strings"
)

func steadyJourney(ctx context.Context, client *http.Client, baseURL string, actor session, index int, collector *stats) error {
	if len(actor.OfferIDs) == 0 {
		return errors.New("steady scenario requires sessions with offer_ids")
	}
	loadBucket := index % 100
	switch {
	case loadBucket < 50:
		return get(ctx, client, baseURL, "/offers", actor.Session, 200, collector)
	case loadBucket < 85:
		return get(ctx, client, baseURL, "/profile/offers/"+actor.OfferIDs[0], actor.Session, 200, collector)
	case loadBucket < 97:
		return get(ctx, client, baseURL, "/", "", 200, collector)
	default:
		return offerWriteJourney(ctx, client, baseURL, actor, index, collector)
	}
}

func offerWriteJourney(ctx context.Context, client *http.Client, baseURL string, actor session, index int, collector *stats) error {
	formBody, err := request(ctx, client, baseURL, "/profile/offers/new", actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	keyMatch := idempotencyKeyInput.FindStringSubmatch(formBody)
	if len(keyMatch) != 2 {
		return errors.New("new-offer form did not include an idempotency key")
	}
	title := scenarioTitle("Load", actor, index)
	createValues := map[string]string{
		"title": title, "description": "Go load test create edit delete journey", "price": "500",
		"currency": "NGN", "category": "other", "idempotency_key": keyMatch[1],
	}
	offerPath, err := requestWithLocation(ctx, client, baseURL, "/profile/offers/new", actor.Session, createValues, 303, collector)
	if err != nil {
		return err
	}
	if !strings.HasPrefix(offerPath, "/profile/offers/") || strings.Contains(offerPath, "..") {
		return fmt.Errorf("create returned unexpected offer location %q", offerPath)
	}
	createdBody, err := request(ctx, client, baseURL, offerPath, actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	if !strings.Contains(createdBody, title) {
		return errors.New("created offer detail did not contain its title")
	}
	if err := get(ctx, client, baseURL, offerPath+"/edit", actor.Session, 200, collector); err != nil {
		return err
	}
	editValues := map[string]string{"title": title + " Edited", "description": "Go load test edited offer", "price": "0", "currency": "NGN", "category": "other"}
	if err := post(ctx, client, baseURL, offerPath+"/edit", actor.Session, editValues, 303, collector); err != nil {
		return err
	}
	editedBody, err := request(ctx, client, baseURL, offerPath, actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	if !strings.Contains(editedBody, title+" Edited") || !strings.Contains(editedBody, "Free") {
		return errors.New("edited offer detail did not show the updated title and Free price")
	}
	if err := post(ctx, client, baseURL, offerPath+"/delete", actor.Session, nil, 303, collector); err != nil {
		return err
	}
	if err := get(ctx, client, baseURL, offerPath, actor.Session, 404, collector); err != nil {
		return err
	}
	return get(ctx, client, baseURL, "/offers", actor.Session, 200, collector)
}

func scenarioTitle(prefix string, actor session, index int) string {
	personTag := actor.UserID
	if len(personTag) > 8 {
		personTag = personTag[:8]
	}
	return fmt.Sprintf("%s %s-%d", prefix, personTag, index)
}

func v05Journey(ctx context.Context, client *http.Client, baseURL string, collector *stats) error {
	combinedPath := "/offers?q=Test+Offer&currency=NGN&min_price=1000&max_price=10000"
	combinedBody, err := request(ctx, client, baseURL, combinedPath, "", nil, 200, collector)
	if err != nil {
		return err
	}
	nextPage := paginationHref.FindStringSubmatch(combinedBody)
	if len(nextPage) != 2 {
		return errors.New("combined search and price results did not render a next-page link")
	}
	nextPath := html.UnescapeString(nextPage[1])
	for _, required := range []string{"q=", "currency=NGN", "min_price=", "max_price="} {
		if !strings.Contains(nextPath, required) {
			return fmt.Errorf("filtered pagination link lost %q: %s", required, nextPath)
		}
	}
	if err := get(ctx, client, baseURL, nextPath, "", 200, collector); err != nil {
		return err
	}
	freeBody, err := request(ctx, client, baseURL, "/offers?currency=NGN&max_price=0", "", nil, 200, collector)
	if err != nil {
		return err
	}
	if !strings.Contains(freeBody, "Free") {
		return errors.New("max_price=0 feed did not contain a Free offer")
	}
	emptyBody, err := request(ctx, client, baseURL, "/offers?q=haven-no-result-1f35b7", "", nil, 200, collector)
	if err != nil {
		return err
	}
	if !strings.Contains(emptyBody, "No offers match your filters") {
		return errors.New("no-result search did not render the filtered empty state")
	}
	return nil
}

func authorizationJourney(ctx context.Context, client *http.Client, baseURL string, sessions []session, index int, collector *stats) error {
	actor := sessions[index%len(sessions)]
	other := sessions[(index+1)%len(sessions)]
	if len(other.OfferIDs) == 0 {
		return errors.New("authorization target has no seeded offer")
	}
	path := "/profile/offers/" + other.OfferIDs[0]
	if err := get(ctx, client, baseURL, path, actor.Session, 404, collector); err != nil {
		return err
	}
	if err := get(ctx, client, baseURL, path+"/edit", actor.Session, 404, collector); err != nil {
		return err
	}
	if err := post(ctx, client, baseURL, path+"/edit", actor.Session, map[string]string{"title": "Unauthorized edit", "price": "0", "currency": "NGN"}, 404, collector); err != nil {
		return err
	}
	if err := post(ctx, client, baseURL, path+"/delete", actor.Session, nil, 404, collector); err != nil {
		return err
	}
	ownerBody, err := request(ctx, client, baseURL, path, other.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	if strings.Contains(ownerBody, "Unauthorized edit") {
		return errors.New("unauthorized write changed the owner's offer")
	}
	for _, route := range []string{path, path + "/edit", "/profile/offers/new"} {
		if err := get(ctx, client, baseURL, route, "", 303, collector); err != nil {
			return err
		}
	}
	for _, route := range []string{path + "/edit", path + "/delete"} {
		if err := post(ctx, client, baseURL, route, "", map[string]string{"title": "Anonymous edit"}, 303, collector); err != nil {
			return err
		}
	}
	formBody, err := request(ctx, client, baseURL, "/profile/offers/new", actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	keyMatch := idempotencyKeyInput.FindStringSubmatch(formBody)
	if len(keyMatch) != 2 {
		return errors.New("new-offer form did not include an idempotency key")
	}
	values := map[string]string{"title": scenarioTitle("Idem", actor, index), "description": "Go load test idempotency check", "price": "500", "currency": "NGN", "category": "other", "idempotency_key": keyMatch[1]}
	type createResult struct {
		location string
		err      error
	}
	results := make(chan createResult, 2)
	for range 2 {
		go func() {
			_, location, requestErr := requestResult(ctx, client, baseURL, "/profile/offers/new", actor.Session, values, 303, collector)
			results <- createResult{location: location, err: requestErr}
		}()
	}
	first, second := <-results, <-results
	if first.err != nil {
		return first.err
	}
	if second.err != nil {
		return second.err
	}
	firstLocation, secondLocation := first.location, second.location
	if firstLocation == "" || firstLocation != secondLocation {
		return fmt.Errorf("idempotent requests returned different locations %q and %q", firstLocation, secondLocation)
	}
	if !strings.HasPrefix(firstLocation, "/profile/offers/") {
		return fmt.Errorf("idempotent create returned unexpected location %q", firstLocation)
	}
	return post(ctx, client, baseURL, firstLocation+"/delete", actor.Session, nil, 303, collector)
}

var paginationHref = regexp.MustCompile(`href=["']([^"']*\?[^"']*after=[^"']+)["']`)
var idempotencyKeyInput = regexp.MustCompile(`<input\b[^>]*name=["']idempotency_key["'][^>]*value=["']([^"']+)["']`)

func v06Journey(ctx context.Context, client *http.Client, baseURL string, sessions []session, workerID int, collector *stats) error {
	groupStart := workerID * 22
	if len(sessions) < groupStart+22 || sessions[groupStart].Session == "" {
		return errors.New("v06 isolated fixture group is incomplete")
	}
	actor := sessions[groupStart]
	targets := sessions[groupStart+1 : groupStart+22]
	for _, target := range targets {
		if len(target.OfferSlugs) == 0 || target.Handle == "" {
			return errors.New("v06 targets require offer slugs and handles; regenerate sessions.json")
		}
	}
	for _, target := range targets {
		for range 2 {
			if err := post(ctx, client, baseURL, "/offers/"+target.OfferSlugs[0]+"/save", actor.Session, map[string]string{"action": "save"}, 303, collector); err != nil {
				return err
			}
			if err := post(ctx, client, baseURL, "/profiles/"+target.Handle+"/follow", actor.Session, map[string]string{"action": "follow"}, 303, collector); err != nil {
				return err
			}
		}
	}
	savedBody, err := request(ctx, client, baseURL, "/profile/saved", actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	followingBody, err := request(ctx, client, baseURL, "/profile/following", actor.Session, nil, 200, collector)
	if err != nil {
		return err
	}
	for _, page := range []struct {
		body string
		path string
	}{{savedBody, "/profile/saved"}, {followingBody, "/profile/following"}} {
		nextPage := paginationHref.FindStringSubmatch(page.body)
		if len(nextPage) != 2 {
			return fmt.Errorf("%s did not paginate with 21 fixture entries", page.path)
		}
		if err := get(ctx, client, baseURL, html.UnescapeString(nextPage[1]), actor.Session, 200, collector); err != nil {
			return err
		}
	}
	for _, target := range targets {
		for range 2 {
			if err := post(ctx, client, baseURL, "/offers/"+target.OfferSlugs[0]+"/save", actor.Session, map[string]string{"action": "unsave"}, 303, collector); err != nil {
				return err
			}
			if err := post(ctx, client, baseURL, "/profiles/"+target.Handle+"/follow", actor.Session, map[string]string{"action": "unfollow"}, 303, collector); err != nil {
				return err
			}
		}
	}
	return nil
}
