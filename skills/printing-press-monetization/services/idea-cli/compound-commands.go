package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

// RegisterIdeaCommands registers all compound IdeaBrowser CLI commands.
func RegisterIdeaCommands(root *cobra.Command) {
	root.AddCommand(searchCmd())
	root.AddCommand(trendingCmd())
	root.AddCommand(validateCmd())
	root.AddCommand(mapCmd())
	root.AddCommand(pitchCmd())
	root.AddCommand(compareCmd())
}

func apiBase() string {
	base := os.Getenv("IDEABROWSER_API_BASE")
	if base == "" {
		base = "https://ideabrowser.com"
	}
	return base
}

func authHeader() string {
	return os.Getenv("IDEABROWSER_SESSION_TOKEN")
}

// --- Search ideas by niche + search volume ---
func searchCmd() *cobra.Command {
	var niche string
	var minSearchVolume int
	var limit int

	cmd := &cobra.Command{
		Use:   "search",
		Short: "Find startup ideas in a niche with minimum search volume",
		Example: `  idea search --niche "developer tools" --min-search-volume 1000
  idea search --niche fintech --limit 20`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if niche == "" {
				return fmt.Errorf("--niche required")
			}

			ideas, err := fetchIdeas(map[string]string{
				"q":                 niche,
				"min_search_volume": fmt.Sprintf("%d", minSearchVolume),
				"limit":             fmt.Sprintf("%d", limit),
			})
			if err != nil {
				return err
			}

			if len(ideas) == 0 {
				fmt.Printf("No ideas found for niche %q with min search volume %d\n", niche, minSearchVolume)
				return nil
			}

			fmt.Printf("=== Ideas for \"%s\" (min %d searches/mo) ===\n\n", niche, minSearchVolume)
			for i, idea := range ideas {
				fmt.Printf("[%d] %s\n", i+1, idea.Title)
				fmt.Printf("    Category:      %s\n", idea.Category)
				fmt.Printf("    Search Volume: %s/mo\n", formatVolume(idea.SearchVolume))
				fmt.Printf("    Reddit Signal: %s\n", idea.RedditSignal)
				fmt.Printf("    Complexity:    %s\n", idea.Complexity)
				fmt.Println()
			}
			return nil
		},
	}

	cmd.Flags().StringVar(&niche, "niche", "", "Niche or keyword to search")
	cmd.Flags().IntVar(&minSearchVolume, "min-search-volume", 0, "Minimum monthly search volume")
	cmd.Flags().IntVar(&limit, "limit", 10, "Max results")
	return cmd
}

// --- Trending: what's blowing up today ---
func trendingCmd() *cobra.Command {
	var today bool
	var limit int

	cmd := &cobra.Command{
		Use:   "trending",
		Short: "Top trending startup ideas right now",
		Example: `  idea trending --today
  idea trending --limit 20`,
		RunE: func(cmd *cobra.Command, args []string) error {
			params := map[string]string{
				"sort":  "trending",
				"limit": fmt.Sprintf("%d", limit),
			}
			if today {
				params["period"] = "today"
			}

			ideas, err := fetchIdeas(params)
			if err != nil {
				return err
			}

			period := "this week"
			if today {
				period = "today"
			}

			fmt.Printf("=== Trending Ideas — %s ===\n\n", period)
			for i, idea := range ideas {
				fmt.Printf("[%d] %s\n", i+1, idea.Title)
				fmt.Printf("    Trend:    %s\n", idea.TrendScore)
				fmt.Printf("    Reddit:   %s\n", idea.RedditSignal)
				fmt.Printf("    Category: %s\n\n", idea.Category)
			}
			return nil
		},
	}

	cmd.Flags().BoolVar(&today, "today", false, "Show today's trends only")
	cmd.Flags().IntVar(&limit, "limit", 10, "Max results")
	return cmd
}

// --- Validate: score an idea against market signals ---
func validateCmd() *cobra.Command {
	var depth string

	cmd := &cobra.Command{
		Use:   "validate [idea]",
		Short: "Validate an idea against real market signals",
		Example: `  idea validate "AI code review SaaS" --depth full
  idea validate "crypto tax automation"`,
		Args: cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			query := strings.Join(args, " ")

			idea, err := fetchValidation(query, depth)
			if err != nil {
				return err
			}

			fmt.Printf("=== Validation: %q ===\n\n", query)
			fmt.Printf("Market Size:     %s\n", idea.MarketSize)
			fmt.Printf("Competition:     %s\n", idea.Competition)
			fmt.Printf("Search Volume:   %s/mo\n", formatVolume(idea.SearchVolume))
			fmt.Printf("Reddit Activity: %s\n", idea.RedditSignal)
			fmt.Printf("Trend:           %s\n", idea.TrendScore)
			fmt.Printf("Verdict:         %s\n\n", idea.Verdict)

			if depth == "full" && len(idea.Gaps) > 0 {
				fmt.Println("Gaps in current solutions:")
				for _, g := range idea.Gaps {
					fmt.Printf("  - %s\n", g)
				}
			}
			return nil
		},
	}

	cmd.Flags().StringVar(&depth, "depth", "standard", "Validation depth: standard, full")
	return cmd
}

// --- Map: market map for a space ---
func mapCmd() *cobra.Command {
	var showCompetitors bool
	var showGaps bool

	cmd := &cobra.Command{
		Use:   "map [space]",
		Short: "Build a quick market map for a space",
		Example: `  idea map "fintech" --competitors --gaps
  idea map "developer tools"`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			space := args[0]

			result, err := fetchMarketMap(space)
			if err != nil {
				return err
			}

			fmt.Printf("=== Market Map: %s ===\n\n", space)
			fmt.Printf("Total Ideas:    %d\n", result.TotalIdeas)
			fmt.Printf("Avg Volume:     %s/mo\n\n", formatVolume(result.AvgSearchVolume))

			fmt.Println("Sub-categories:")
			for _, cat := range result.Categories {
				fmt.Printf("  %-25s %d ideas\n", cat.Name, cat.Count)
			}

			if showCompetitors && len(result.Players) > 0 {
				fmt.Println("\nKnown Players:")
				for _, p := range result.Players {
					fmt.Printf("  - %s (%s)\n", p.Name, p.Stage)
				}
			}

			if showGaps && len(result.Gaps) > 0 {
				fmt.Println("\nOpportunity Gaps:")
				for _, g := range result.Gaps {
					fmt.Printf("  - %s\n", g)
				}
			}

			return nil
		},
	}

	cmd.Flags().BoolVar(&showCompetitors, "competitors", false, "Show known players in the space")
	cmd.Flags().BoolVar(&showGaps, "gaps", false, "Show identified opportunity gaps")
	return cmd
}

// --- Pitch: generate investor pitch for top ideas ---
func pitchCmd() *cobra.Command {
	var top int
	var format string

	cmd := &cobra.Command{
		Use:   "pitch [space]",
		Short: "Generate pitch for top ideas in a space",
		Example: `  idea pitch fintech --top 3 --format investor
  idea pitch "dev tools" --format one-pager`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			space := args[0]

			ideas, err := fetchIdeas(map[string]string{
				"q":     space,
				"sort":  "trending",
				"limit": fmt.Sprintf("%d", top),
			})
			if err != nil {
				return err
			}

			for i, idea := range ideas {
				fmt.Printf("=== Pitch %d: %s ===\n\n", i+1, idea.Title)

				switch format {
				case "investor":
					fmt.Printf("Problem:     %s\n", idea.Problem)
					fmt.Printf("Solution:    %s\n", idea.Solution)
					fmt.Printf("Market:      %s\n", idea.MarketSize)
					fmt.Printf("Traction:    %s searches/mo, Reddit: %s\n", formatVolume(idea.SearchVolume), idea.RedditSignal)
					fmt.Printf("Competitors: %s\n", idea.Competition)
					fmt.Printf("Ask:         [Your funding ask here]\n")
				default:
					fmt.Printf("What:        %s\n", idea.Title)
					fmt.Printf("Problem:     %s\n", idea.Problem)
					fmt.Printf("Insight:     %s searches/mo with %s competition\n",
						formatVolume(idea.SearchVolume), strings.ToLower(idea.Competition))
				}
				fmt.Println()
			}
			return nil
		},
	}

	cmd.Flags().IntVar(&top, "top", 3, "Number of top ideas to pitch")
	cmd.Flags().StringVar(&format, "format", "one-pager", "Pitch format: one-pager, investor")
	return cmd
}

// --- Compare: side-by-side two ideas ---
func compareCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "compare [idea1] [idea2]",
		Short: "Compare two ideas on all dimensions",
		Example: `  idea compare "CLI tools SaaS" "MCP marketplace"`,
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			idea1, err := fetchValidation(args[0], "standard")
			if err != nil {
				return fmt.Errorf("idea 1: %w", err)
			}
			idea2, err := fetchValidation(args[1], "standard")
			if err != nil {
				return fmt.Errorf("idea 2: %w", err)
			}

			fmt.Printf("%-25s %-20s %-20s\n", "", args[0][:minLen(args[0], 18)], args[1][:minLen(args[1], 18)])
			fmt.Println(strings.Repeat("-", 65))
			fmt.Printf("%-25s %-20s %-20s\n", "Search Volume", formatVolume(idea1.SearchVolume), formatVolume(idea2.SearchVolume))
			fmt.Printf("%-25s %-20s %-20s\n", "Competition", idea1.Competition, idea2.Competition)
			fmt.Printf("%-25s %-20s %-20s\n", "Market Size", idea1.MarketSize, idea2.MarketSize)
			fmt.Printf("%-25s %-20s %-20s\n", "Trend", idea1.TrendScore, idea2.TrendScore)
			fmt.Printf("%-25s %-20s %-20s\n", "Reddit Signal", idea1.RedditSignal, idea2.RedditSignal)
			fmt.Printf("%-25s %-20s %-20s\n", "Verdict", idea1.Verdict, idea2.Verdict)
			return nil
		},
	}
	return cmd
}

// ============================================================
// API types and fetch helpers
// ============================================================

type Idea struct {
	Title        string
	Category     string
	Problem      string
	Solution     string
	MarketSize   string
	Competition  string
	SearchVolume int
	RedditSignal string
	TrendScore   string
	Complexity   string
	Verdict      string
	Gaps         []string
}

type MarketMap struct {
	TotalIdeas      int
	AvgSearchVolume int
	Categories      []struct {
		Name  string
		Count int
	}
	Players []struct {
		Name  string
		Stage string
	}
	Gaps []string
}

func fetchIdeas(params map[string]string) ([]Idea, error) {
	q := url.Values{}
	for k, v := range params {
		q.Set(k, v)
	}

	reqURL := apiBase() + "/api/ideas?" + q.Encode()
	data, err := ideaGet(reqURL)
	if err != nil {
		return nil, err
	}

	var resp struct {
		Ideas []Idea `json:"ideas"`
	}
	if err := json.Unmarshal(data, &resp); err != nil {
		return nil, fmt.Errorf("parse error: %w (raw: %s)", err, string(data[:minLen(string(data), 200)]))
	}
	return resp.Ideas, nil
}

func fetchValidation(query, depth string) (*Idea, error) {
	q := url.Values{"q": {query}, "depth": {depth}}
	data, err := ideaGet(apiBase() + "/api/validate?" + q.Encode())
	if err != nil {
		return nil, err
	}

	var idea Idea
	if err := json.Unmarshal(data, &idea); err != nil {
		return nil, err
	}
	return &idea, nil
}

func fetchMarketMap(space string) (*MarketMap, error) {
	q := url.Values{"space": {space}}
	data, err := ideaGet(apiBase() + "/api/market-map?" + q.Encode())
	if err != nil {
		return nil, err
	}

	var m MarketMap
	if err := json.Unmarshal(data, &m); err != nil {
		return nil, err
	}
	return &m, nil
}

func ideaGet(reqURL string) ([]byte, error) {
	req, err := http.NewRequest("GET", reqURL, nil)
	if err != nil {
		return nil, err
	}

	token := authHeader()
	if token != "" {
		req.Header.Set("Cookie", "session="+token)
		req.Header.Set("Authorization", "Bearer "+token)
	}
	req.Header.Set("User-Agent", "idea-cli/1.0")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode == 401 {
		return nil, fmt.Errorf("unauthorized — set IDEABROWSER_SESSION_TOKEN or run 'idea auth --key YOUR_LICENSE_KEY'")
	}
	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("HTTP %d from IdeaBrowser API", resp.StatusCode)
	}

	return io.ReadAll(resp.Body)
}

func formatVolume(v int) string {
	if v >= 1_000_000 {
		return fmt.Sprintf("%.1fM", float64(v)/1_000_000)
	}
	if v >= 1_000 {
		return fmt.Sprintf("%.1fK", float64(v)/1_000)
	}
	return fmt.Sprintf("%d", v)
}

func minLen(s string, n int) int {
	if len(s) < n {
		return len(s)
	}
	return n
}

// ActivateIdeaLicenseKey validates and stores a LemonSqueezy license key for the idea CLI.
func ActivateIdeaLicenseKey(key string) error {
	resp, err := http.Post(
		"https://api.lemonsqueezy.com/v1/licenses/activate",
		"application/x-www-form-urlencoded",
		strings.NewReader("license_key="+key+"&instance_name="+ideaHostname()),
	)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var result struct {
		Activated bool   `json:"activated"`
		Error     string `json:"error"`
	}
	json.Unmarshal(body, &result)

	if !result.Activated {
		return fmt.Errorf("activation failed: %s", result.Error)
	}

	keyPath := expandIdeaPath("~/.idea-cli/.license")
	os.MkdirAll(strings.TrimSuffix(keyPath, "/.license"), 0700)
	os.WriteFile(keyPath, []byte(key), 0600)

	fmt.Println("License activated — unlimited IdeaBrowser queries enabled")
	return nil
}

func ideaHostname() string {
	h, _ := os.Hostname()
	return h
}

func expandIdeaPath(p string) string {
	if strings.HasPrefix(p, "~/") {
		home, _ := os.UserHomeDir()
		return home + p[1:]
	}
	return p
}
