package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"
)

const (
	lsValidateURL = "https://api.lemonsqueezy.com/v1/licenses/validate"
	lsActivateURL = "https://api.lemonsqueezy.com/v1/licenses/activate"
	keyFile       = "~/.crypto-intel/.license"
	freeDailyLimit = 10
)

type LicenseState struct {
	Valid       bool
	Plan        string // "free" | "pro"
	QueriesLeft int
	LastReset   time.Time
}

var licenseState *LicenseState

// CheckAuth validates the license key at startup and enforces free tier limits.
// Returns an error if the free quota is exhausted.
func CheckAuth() error {
	key := os.Getenv("CRYPTO_INTEL_LICENSE")
	if key == "" {
		key = readStoredKey()
	}

	if key != "" {
		valid, plan, err := validateLicenseKey(key)
		if err == nil && valid {
			licenseState = &LicenseState{Valid: true, Plan: plan}
			return nil
		}
	}

	// Free tier: track daily query count
	count, resetTime := getDailyCount()
	if count >= freeDailyLimit {
		nextReset := resetTime.Add(24 * time.Hour)
		return fmt.Errorf(
			"free tier limit reached (%d queries/day).\n"+
				"  Limit resets at %s\n"+
				"  Upgrade at: https://cryptointel.lemonsqueezy.com — $49/month",
			freeDailyLimit,
			nextReset.Format("15:04 MST"),
		)
	}

	licenseState = &LicenseState{
		Valid:       true,
		Plan:        "free",
		QueriesLeft: freeDailyLimit - count,
	}
	return nil
}

// RecordQuery increments the daily query counter for free tier users.
func RecordQuery() {
	if licenseState != nil && licenseState.Plan != "free" {
		return
	}
	incrementDailyCount()
}

// ShowPlanInfo prints current license status.
func ShowPlanInfo() {
	if licenseState == nil {
		fmt.Println("License: not checked")
		return
	}
	if licenseState.Plan == "free" {
		fmt.Printf("Plan: Free (%d queries remaining today)\n", licenseState.QueriesLeft)
		fmt.Println("Upgrade: https://cryptointel.lemonsqueezy.com")
	} else {
		fmt.Printf("Plan: Pro — unlimited queries\n")
	}
}

// ActivateLicenseKey stores and validates a LemonSqueezy license key.
func ActivateLicenseKey(key string) error {
	resp, err := lsPost(lsActivateURL, map[string]string{
		"license_key":    key,
		"instance_name": hostname(),
	})
	if err != nil {
		return fmt.Errorf("activation failed: %w", err)
	}

	if !resp.Activated {
		return fmt.Errorf("key not valid: %s", resp.Error)
	}

	if err := storeKey(key); err != nil {
		fmt.Printf("Warning: could not save key (%v) — set CRYPTO_INTEL_LICENSE=%s\n", err, key)
	}

	licenseState = &LicenseState{Valid: true, Plan: "pro"}
	fmt.Println("License activated — unlimited queries enabled")
	return nil
}

// ============================================================
// Internal helpers
// ============================================================

type lsResponse struct {
	Activated bool   `json:"activated"`
	Valid     bool   `json:"valid"`
	Error     string `json:"error"`
	License   struct {
		Status string `json:"status"`
	} `json:"license_key"`
}

func validateLicenseKey(key string) (bool, string, error) {
	resp, err := lsPost(lsValidateURL, map[string]string{
		"license_key":    key,
		"instance_name": hostname(),
	})
	if err != nil {
		return false, "", err
	}
	if !resp.Valid {
		return false, "", nil
	}
	return true, "pro", nil
}

func lsPost(url string, payload map[string]string) (*lsResponse, error) {
	body := make([]string, 0, len(payload))
	for k, v := range payload {
		body = append(body, k+"="+v)
	}

	req, err := http.NewRequest("POST", url, strings.NewReader(strings.Join(body, "&")))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	req.Header.Set("Accept", "application/json")

	r, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer r.Body.Close()

	data, _ := io.ReadAll(r.Body)
	var resp lsResponse
	return &resp, json.Unmarshal(data, &resp)
}

func readStoredKey() string {
	path := expandPath(keyFile)
	data, err := os.ReadFile(path)
	if err != nil {
		return ""
	}
	return strings.TrimSpace(string(data))
}

func storeKey(key string) error {
	path := expandPath(keyFile)
	if err := os.MkdirAll(strings.TrimSuffix(path, "/.license"), 0700); err != nil {
		return err
	}
	return os.WriteFile(path, []byte(key), 0600)
}

func getDailyCount() (int, time.Time) {
	path := expandPath("~/.crypto-intel/.usage")
	data, err := os.ReadFile(path)
	if err != nil {
		return 0, time.Now()
	}

	var usage struct {
		Date  string `json:"date"`
		Count int    `json:"count"`
	}
	if err := json.Unmarshal(data, &usage); err != nil {
		return 0, time.Now()
	}

	today := time.Now().Format("2006-01-02")
	if usage.Date != today {
		return 0, time.Now()
	}

	resetTime, _ := time.Parse("2006-01-02", usage.Date)
	return usage.Count, resetTime
}

func incrementDailyCount() {
	path := expandPath("~/.crypto-intel/.usage")
	count, _ := getDailyCount()

	usage := map[string]interface{}{
		"date":  time.Now().Format("2006-01-02"),
		"count": count + 1,
	}
	data, _ := json.Marshal(usage)
	_ = os.WriteFile(path, data, 0600)

	if licenseState != nil {
		licenseState.QueriesLeft = freeDailyLimit - (count + 1)
	}
}

func hostname() string {
	h, _ := os.Hostname()
	return h
}

func expandPath(p string) string {
	if strings.HasPrefix(p, "~/") {
		home, _ := os.UserHomeDir()
		return home + p[1:]
	}
	return p
}
