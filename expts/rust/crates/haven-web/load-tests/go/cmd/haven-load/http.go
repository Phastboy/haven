package main

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"sort"
	"strings"
	"time"
)

func requestResult(ctx context.Context, client *http.Client, baseURL, path, token string, values map[string]string, expected int, collector *stats) (string, string, error) {
	var bodyReader io.Reader
	if values != nil && len(values) > 0 {
		form := url.Values{}
		for key, value := range values {
			form.Set(key, value)
		}
		bodyReader = strings.NewReader(form.Encode())
	}
	method := http.MethodGet
	if values != nil {
		method = http.MethodPost
	}
	req, err := http.NewRequestWithContext(ctx, method, strings.TrimRight(baseURL, "/")+path, bodyReader)
	if err != nil {
		return "", "", err
	}
	if token != "" {
		req.Header.Set("Cookie", "sid="+token)
	}
	if method == http.MethodPost {
		req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
		req.Header.Set("Origin", strings.TrimRight(baseURL, "/"))
		req.Header.Set("Sec-Fetch-Site", "same-origin")
	}
	started := time.Now()
	response, requestErr := client.Do(req)
	if requestErr != nil {
		entry := sample{Route: routeLabel(path), Duration: time.Since(started), Err: requestErr.Error()}
		collector.record(entry, false)
		return "", "", requestErr
	}
	defer response.Body.Close()

	entry := sample{Route: routeLabel(path), Status: response.StatusCode}
	location := response.Header.Get("Location")
	var content strings.Builder
	_, readErr := io.Copy(&content, response.Body)
	entry.Duration = time.Since(started)
	if response.StatusCode != expected {
		requestErr = fmt.Errorf("%s %s: got %d, expected %d", method, path, response.StatusCode, expected)
	} else if readErr != nil {
		requestErr = readErr
	}
	if requestErr != nil {
		entry.Err = requestErr.Error()
	}
	collector.record(entry, requestErr == nil)
	return content.String(), location, requestErr
}

func routeLabel(raw string) string {
	parsed, err := url.Parse(raw)
	if err != nil {
		return raw
	}
	path := parsed.Path
	if path == "" {
		path = "/"
	}
	if len(path) > 1 {
		path = strings.TrimRight(path, "/")
	}

	switch {
	case path == "/profile/offers/new":
	case strings.HasPrefix(path, "/profile/offers/"):
		rest := strings.TrimPrefix(path, "/profile/offers/")
		if i := strings.IndexByte(rest, '/'); i >= 0 {
			path = "/profile/offers/{id}" + rest[i:]
		} else {
			path = "/profile/offers/{id}"
		}
	case strings.HasPrefix(path, "/offers/manage/"):
		path = "/offers/manage/{id}"
	case strings.HasPrefix(path, "/offers/"):
		rest := strings.TrimPrefix(path, "/offers/")
		if i := strings.IndexByte(rest, '/'); i >= 0 {
			path = "/offers/{slug}" + rest[i:]
		} else {
			path = "/offers/{slug}"
		}
	case strings.HasPrefix(path, "/profiles/"):
		rest := strings.TrimPrefix(path, "/profiles/")
		if i := strings.IndexByte(rest, '/'); i >= 0 {
			path = "/profiles/{handle}" + rest[i:]
		} else {
			path = "/profiles/{handle}"
		}
	case strings.HasPrefix(path, "/conversations/"):
		rest := strings.TrimPrefix(path, "/conversations/")
		if i := strings.IndexByte(rest, '/'); i >= 0 {
			path = "/conversations/{id}" + rest[i:]
		} else {
			path = "/conversations/{id}"
		}
	}

	keys := make([]string, 0)
	for key := range parsed.Query() {
		switch key {
		case "after", "before", "cursor", "page", "limit":
			continue
		default:
			keys = append(keys, key)
		}
	}
	sort.Strings(keys)
	if len(keys) > 0 {
		path += "?" + strings.Join(keys, "&")
	}
	return path
}

func get(ctx context.Context, client *http.Client, baseURL, path, token string, status int, collector *stats) error {
	_, err := request(ctx, client, baseURL, path, token, nil, status, collector)
	return err
}

func post(ctx context.Context, client *http.Client, baseURL, path, token string, values map[string]string, status int, collector *stats) error {
	if values == nil {
		values = make(map[string]string)
	}
	_, err := request(ctx, client, baseURL, path, token, values, status, collector)
	return err
}

func request(ctx context.Context, client *http.Client, baseURL, path, token string, values map[string]string, expected int, collector *stats) (string, error) {
	body, _, err := requestResult(ctx, client, baseURL, path, token, values, expected, collector)
	return body, err
}

func requestWithLocation(ctx context.Context, client *http.Client, baseURL, path, token string, values map[string]string, expected int, collector *stats) (string, error) {
	_, location, err := requestResult(ctx, client, baseURL, path, token, values, expected, collector)
	return location, err
}
