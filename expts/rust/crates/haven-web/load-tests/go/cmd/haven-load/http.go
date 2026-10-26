package main

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

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
	duration := time.Since(started)
	entry := sample{Route: path, Duration: duration}
	if requestErr != nil {
		entry.Err = requestErr.Error()
		collector.record(entry, false)
		return "", "", requestErr
	}
	defer response.Body.Close()
	entry.Status = response.StatusCode
	location := response.Header.Get("Location")
	var content strings.Builder
	_, readErr := io.Copy(&content, response.Body)
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
