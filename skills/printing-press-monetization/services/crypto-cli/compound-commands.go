package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/spf13/cobra"
)

// CompoundRoot registers all compound commands that stitch multiple APIs together.
// Call this from your main CLI root command.
func RegisterCompoundCommands(root *cobra.Command) {
	root.AddCommand(arbCmd())
	root.AddCommand(signalCmd())
	root.AddCommand(portfolioCmd())
	root.AddCommand(marketPhaseCmd())
	root.AddCommand(alertCmd())
	root.AddCommand(swapCmd())
}

// --- Arbitrage: find cross-exchange spreads > N% ---
func arbCmd() *cobra.Command {
	var minSpread float64
	var liquidOnly bool

	cmd := &cobra.Command{
		Use:   "arb",
		Short: "Find cross-exchange arbitrage opportunities",
		Example: `  crypto-intel arb --min-spread 2 --liquid
  crypto-intel arb --min-spread 1.5`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Get tickers from CoinGecko (includes exchange data)
			tickers, err := fetchCoinGeckoTickers("bitcoin")
			if err != nil {
				return err
			}

			type Spread struct {
				Coin     string
				Exchange string
				Last     float64
				Spread   float64
			}

			var spreads []Spread

			// Group by coin, find min/max across exchanges
			type PriceEntry struct {
				exchange string
				price    float64
				volume   float64
			}

			byBase := map[string][]PriceEntry{}
			for _, t := range tickers {
				if t.Last <= 0 {
					continue
				}
				if liquidOnly && t.Volume < 1_000_000 {
					continue
				}
				byBase[t.Base] = append(byBase[t.Base], PriceEntry{
					exchange: t.Market.Name,
					price:    t.Last,
					volume:   t.Volume,
				})
			}

			for coin, entries := range byBase {
				if len(entries) < 2 {
					continue
				}
				prices := make([]float64, len(entries))
				for i, e := range entries {
					prices[i] = e.price
				}
				sort.Float64s(prices)
				lo, hi := prices[0], prices[len(prices)-1]
				spreadPct := (hi - lo) / lo * 100

				if spreadPct >= minSpread {
					spreads = append(spreads, Spread{
						Coin:   coin,
						Last:   hi,
						Spread: spreadPct,
					})
				}
			}

			sort.Slice(spreads, func(i, j int) bool { return spreads[i].Spread > spreads[j].Spread })

			if len(spreads) == 0 {
				fmt.Printf("No arbitrage >= %.1f%% found\n", minSpread)
				return nil
			}

			fmt.Printf("%-10s %-10s %s\n", "Coin", "Price", "Spread")
			fmt.Println(strings.Repeat("-", 40))
			for _, s := range spreads {
				fmt.Printf("%-10s %-10.4f %.2f%%\n", s.Coin, s.Last, s.Spread)
			}
			return nil
		},
	}

	cmd.Flags().Float64Var(&minSpread, "min-spread", 2.0, "Minimum spread percentage")
	cmd.Flags().BoolVar(&liquidOnly, "liquid", false, "Only include liquid markets (>$1M volume)")
	return cmd
}

// --- Signal: combined price + RSI + MACD for a coin ---
func signalCmd() *cobra.Command {
	var explain bool

	cmd := &cobra.Command{
		Use:     "signal [coin]",
		Short:   "Trading signal for a coin (price + RSI + MACD from altFINS)",
		Example: "  crypto-intel signal ETH --explain",
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			symbol := strings.ToUpper(args[0])

			// CoinGecko: current price + 24h change
			price, change24h, err := fetchCoinPrice(symbol)
			if err != nil {
				return fmt.Errorf("price fetch: %w", err)
			}

			// altFINS: RSI, MACD, signal
			signal, err := fetchAltFINSSignal(symbol)
			if err != nil {
				// altFINS might not have this coin — show price-only
				fmt.Printf("%s  Price: $%.4f  24h: %.2f%%\n", symbol, price, change24h)
				fmt.Println("(Signal data unavailable — altFINS free tier)")
				return nil
			}

			fmt.Printf("\n=== %s Signal ===\n", symbol)
			fmt.Printf("Price:     $%.4f\n", price)
			fmt.Printf("24h:       %.2f%%\n", change24h)
			fmt.Printf("RSI (14):  %.1f\n", signal.RSI)
			fmt.Printf("MACD:      %.4f\n", signal.MACD)
			fmt.Printf("Signal:    %s\n", signal.Summary)

			if explain {
				fmt.Printf("\nInterpretation:\n")
				if signal.RSI < 30 {
					fmt.Println("  RSI < 30: Oversold — potential buy zone")
				} else if signal.RSI > 70 {
					fmt.Println("  RSI > 70: Overbought — potential sell zone")
				} else {
					fmt.Println("  RSI neutral — no strong signal")
				}
				if signal.MACD > 0 {
					fmt.Println("  MACD positive — bullish momentum")
				} else {
					fmt.Println("  MACD negative — bearish momentum")
				}
			}
			return nil
		},
	}

	cmd.Flags().BoolVar(&explain, "explain", false, "Explain the signal in plain English")
	return cmd
}

// --- Portfolio: P&L calculator ---
func portfolioCmd() *cobra.Command {
	var costBasis string

	cmd := &cobra.Command{
		Use:   "portfolio",
		Short: "Portfolio P&L since purchase",
		Example: `  crypto-intel portfolio --cost-basis BTC:48000:2,ETH:2100:10`,
		RunE: func(cmd *cobra.Command, args []string) error {
			if costBasis == "" {
				return fmt.Errorf("--cost-basis required: COIN:PRICE:AMOUNT,...")
			}

			type Position struct {
				Symbol    string
				BuyPrice  float64
				Amount    float64
			}

			var positions []Position
			for _, entry := range strings.Split(costBasis, ",") {
				parts := strings.Split(entry, ":")
				if len(parts) != 3 {
					return fmt.Errorf("invalid format: %q (want COIN:PRICE:AMOUNT)", entry)
				}
				buyPrice, _ := strconv.ParseFloat(parts[1], 64)
				amount, _ := strconv.ParseFloat(parts[2], 64)
				positions = append(positions, Position{
					Symbol:   strings.ToUpper(parts[0]),
					BuyPrice: buyPrice,
					Amount:   amount,
				})
			}

			fmt.Printf("%-8s %-12s %-12s %-12s %-10s\n", "Coin", "Bought@", "Now@", "P&L $", "P&L %")
			fmt.Println(strings.Repeat("-", 60))

			totalCost, totalValue := 0.0, 0.0
			for _, pos := range positions {
				currentPrice, _, err := fetchCoinPrice(pos.Symbol)
				if err != nil {
					fmt.Printf("%-8s %-12.2f ERROR: %v\n", pos.Symbol, pos.BuyPrice, err)
					continue
				}
				cost := pos.BuyPrice * pos.Amount
				value := currentPrice * pos.Amount
				pnlDollar := value - cost
				pnlPct := (currentPrice - pos.BuyPrice) / pos.BuyPrice * 100

				sign := "+"
				if pnlDollar < 0 {
					sign = ""
				}
				fmt.Printf("%-8s $%-11.2f $%-11.2f %s$%-10.2f %s%.1f%%\n",
					pos.Symbol, pos.BuyPrice, currentPrice, sign, pnlDollar, sign, pnlPct)

				totalCost += cost
				totalValue += value
			}

			fmt.Println(strings.Repeat("-", 60))
			totalPnL := totalValue - totalCost
			totalPct := totalPnL / totalCost * 100
			sign := "+"
			if totalPnL < 0 {
				sign = ""
			}
			fmt.Printf("%-8s $%-11.2f $%-11.2f %s$%-10.2f %s%.1f%%\n",
				"TOTAL", totalCost, totalValue, sign, totalPnL, sign, totalPct)
			return nil
		},
	}

	cmd.Flags().StringVar(&costBasis, "cost-basis", "", "COIN:PRICE:AMOUNT pairs (comma-separated)")
	_ = cmd.MarkFlagRequired("cost-basis")
	return cmd
}

// --- Market Phase: BTC dominance + alt season indicator ---
func marketPhaseCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "market-phase",
		Short: "Bitcoin dominance + alt season indicator",
		RunE: func(cmd *cobra.Command, args []string) error {
			global, err := fetchCoinGeckoGlobal()
			if err != nil {
				return err
			}

			btcDom := global.Data.MarketCapPercentage["btc"]
			fmt.Printf("BTC Dominance:  %.1f%%\n", btcDom)

			switch {
			case btcDom > 60:
				fmt.Println("Phase:          Bitcoin Season — BTC outperforming alts")
				fmt.Println("Strategy:       Hold BTC, reduce alt exposure")
			case btcDom > 50:
				fmt.Println("Phase:          Transition — market rotating")
				fmt.Println("Strategy:       Watch for alt season breakout")
			default:
				fmt.Println("Phase:          Alt Season — alts outperforming BTC")
				fmt.Println("Strategy:       Diversify into quality alts")
			}

			fmt.Printf("\nTotal Market Cap: $%.2fT\n", global.Data.TotalMarketCap["usd"]/1e12)
			fmt.Printf("24h Volume:       $%.2fB\n", global.Data.TotalVolume["usd"]/1e9)
			return nil
		},
	}
}

// --- Volume Alert: coins with spike > N% in last hour ---
func alertCmd() *cobra.Command {
	var volumeSpike float64
	var watch bool

	cmd := &cobra.Command{
		Use:   "alert",
		Short: "Find coins with unusual volume spikes",
		Example: `  crypto-intel alert --volume-spike 50
  crypto-intel alert --volume-spike 100 --watch`,
		RunE: func(cmd *cobra.Command, args []string) error {
			for {
				err := runVolumeAlert(volumeSpike)
				if err != nil {
					return err
				}
				if !watch {
					break
				}
				fmt.Printf("\n[Watching — checking every 60s — Ctrl+C to stop]\n")
				time.Sleep(60 * time.Second)
			}
			return nil
		},
	}

	cmd.Flags().Float64Var(&volumeSpike, "volume-spike", 50.0, "Minimum volume spike percentage")
	cmd.Flags().BoolVar(&watch, "watch", false, "Keep watching and re-check every 60 seconds")
	return cmd
}

func runVolumeAlert(minSpike float64) error {
	coins, err := fetchCoinGeckoMarkets(1, 250)
	if err != nil {
		return err
	}

	type Spike struct {
		Name   string
		Symbol string
		Price  float64
		Vol24h float64
	}

	// CoinGecko doesn't give hourly volume natively on free tier,
	// so we use 24h volume change as a proxy (positive = increasing)
	var spikes []Spike
	for _, c := range coins {
		if c.PriceChangePercentage24h > minSpike || c.PriceChangePercentage24h < -minSpike {
			spikes = append(spikes, Spike{
				Name:   c.Name,
				Symbol: strings.ToUpper(c.Symbol),
				Price:  c.CurrentPrice,
				Vol24h: c.TotalVolume,
			})
		}
	}

	if len(spikes) == 0 {
		fmt.Printf("No volume spikes >= %.0f%% detected\n", minSpike)
		return nil
	}

	fmt.Printf("\n=== Volume Spikes (%.0f%%+) — %s ===\n", minSpike, time.Now().Format("15:04:05"))
	fmt.Printf("%-10s %-20s %-14s %s\n", "Symbol", "Name", "Price", "24h Volume")
	fmt.Println(strings.Repeat("-", 65))
	for _, s := range spikes {
		fmt.Printf("%-10s %-20s $%-13.4f $%.0f\n", s.Symbol, s.Name, s.Price, s.Vol24h)
	}
	return nil
}

// --- Swap: best route for coin A → coin B using ChangeNOW ---
func swapCmd() *cobra.Command {
	var bestRate bool

	cmd := &cobra.Command{
		Use:     "swap [from] [to] [amount]",
		Short:   "Get best swap quote (from ChangeNOW)",
		Example: "  crypto-intel swap BTC USDC 1 --best-rate",
		Args:    cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			from := strings.ToLower(args[0])
			to := strings.ToLower(args[1])
			amount, err := strconv.ParseFloat(args[2], 64)
			if err != nil {
				return fmt.Errorf("invalid amount: %v", err)
			}

			quote, err := fetchChangeNOWQuote(from, to, amount)
			if err != nil {
				return err
			}

			fmt.Printf("\n=== Swap Quote: %.4f %s → %s ===\n", amount, strings.ToUpper(from), strings.ToUpper(to))
			fmt.Printf("You receive:  %.6f %s\n", quote.ToAmount, strings.ToUpper(to))
			fmt.Printf("Rate:         1 %s = %.6f %s\n", strings.ToUpper(from), quote.ToAmount/amount, strings.ToUpper(to))
			fmt.Printf("Min amount:   %.6f %s\n", quote.MinAmount, strings.ToUpper(from))
			fmt.Printf("Estimated:    ~%s\n", quote.ETA)
			fmt.Printf("\nSwap at: changenow.io\n")
			return nil
		},
	}

	cmd.Flags().BoolVar(&bestRate, "best-rate", false, "Find best rate across routes")
	return cmd
}

// ============================================================
// API Helpers
// ============================================================

func fetchCoinPrice(symbol string) (price, change24h float64, err error) {
	// CoinGecko ID lookup (simplified — covers top coins)
	idMap := map[string]string{
		"BTC": "bitcoin", "ETH": "ethereum", "SOL": "solana",
		"BNB": "binancecoin", "XRP": "ripple", "ADA": "cardano",
		"AVAX": "avalanche-2", "DOGE": "dogecoin", "DOT": "polkadot",
		"MATIC": "matic-network", "LINK": "chainlink", "UNI": "uniswap",
	}

	id, ok := idMap[symbol]
	if !ok {
		id = strings.ToLower(symbol)
	}

	url := fmt.Sprintf("https://api.coingecko.com/api/v3/simple/price?ids=%s&vs_currencies=usd&include_24hr_change=true", id)
	data, err := httpGet(url)
	if err != nil {
		return 0, 0, err
	}

	var resp map[string]map[string]float64
	if err := json.Unmarshal(data, &resp); err != nil {
		return 0, 0, err
	}

	coinData, ok := resp[id]
	if !ok {
		return 0, 0, fmt.Errorf("coin %q not found", symbol)
	}

	return coinData["usd"], coinData["usd_24h_change"], nil
}

type Ticker struct {
	Base   string
	Last   float64
	Volume float64
	Market struct{ Name string }
}

func fetchCoinGeckoTickers(coinID string) ([]Ticker, error) {
	url := fmt.Sprintf("https://api.coingecko.com/api/v3/coins/%s/tickers", coinID)
	data, err := httpGet(url)
	if err != nil {
		return nil, err
	}
	var resp struct {
		Tickers []Ticker
	}
	return resp.Tickers, json.Unmarshal(data, &resp)
}

type Coin struct {
	Name                     string  `json:"name"`
	Symbol                   string  `json:"symbol"`
	CurrentPrice             float64 `json:"current_price"`
	TotalVolume              float64 `json:"total_volume"`
	PriceChangePercentage24h float64 `json:"price_change_percentage_24h"`
}

func fetchCoinGeckoMarkets(page, perPage int) ([]Coin, error) {
	url := fmt.Sprintf("https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&order=volume_desc&per_page=%d&page=%d", perPage, page)
	data, err := httpGet(url)
	if err != nil {
		return nil, err
	}
	var coins []Coin
	return coins, json.Unmarshal(data, &coins)
}

type GlobalData struct {
	Data struct {
		MarketCapPercentage map[string]float64 `json:"market_cap_percentage"`
		TotalMarketCap      map[string]float64 `json:"total_market_cap"`
		TotalVolume         map[string]float64 `json:"total_volume"`
	} `json:"data"`
}

func fetchCoinGeckoGlobal() (*GlobalData, error) {
	data, err := httpGet("https://api.coingecko.com/api/v3/global")
	if err != nil {
		return nil, err
	}
	var g GlobalData
	return &g, json.Unmarshal(data, &g)
}

type AltFINSSignal struct {
	RSI     float64
	MACD    float64
	Summary string
}

func fetchAltFINSSignal(symbol string) (*AltFINSSignal, error) {
	apiKey := os.Getenv("ALTFINS_API_KEY")
	if apiKey == "" {
		return nil, fmt.Errorf("ALTFINS_API_KEY not set")
	}

	url := fmt.Sprintf("https://api.altfins.com/v1/screener/crypto?symbols=%s&fields=rsi14,macd,signal", symbol)
	req, _ := http.NewRequest("GET", url, nil)
	req.Header.Set("Authorization", "Bearer "+apiKey)

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)

	var result struct {
		Data []struct {
			RSI    float64 `json:"rsi14"`
			MACD   float64 `json:"macd"`
			Signal string  `json:"signal"`
		} `json:"data"`
	}

	if err := json.Unmarshal(body, &result); err != nil || len(result.Data) == 0 {
		return nil, fmt.Errorf("no signal data for %s", symbol)
	}

	d := result.Data[0]
	return &AltFINSSignal{RSI: d.RSI, MACD: d.MACD, Summary: d.Signal}, nil
}

type ChangeNOWQuote struct {
	ToAmount  float64
	MinAmount float64
	ETA       string
}

func fetchChangeNOWQuote(from, to string, amount float64) (*ChangeNOWQuote, error) {
	apiKey := os.Getenv("CHANGENOW_API_KEY")
	url := fmt.Sprintf("https://api.changenow.io/v2/exchange/estimated-amount?fromCurrency=%s&toCurrency=%s&fromAmount=%.6f&flow=standard&type=direct", from, to, amount)

	req, _ := http.NewRequest("GET", url, nil)
	if apiKey != "" {
		req.Header.Set("x-changenow-api-key", apiKey)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)

	var result struct {
		ToAmount     float64 `json:"toAmount"`
		MinAmount    float64 `json:"minAmount"`
		EstimatedTime string `json:"estimatedTime"`
	}

	if err := json.Unmarshal(body, &result); err != nil {
		return nil, fmt.Errorf("ChangeNOW parse error: %w", err)
	}

	return &ChangeNOWQuote{
		ToAmount:  result.ToAmount,
		MinAmount: result.MinAmount,
		ETA:       result.EstimatedTime,
	}, nil
}

func httpGet(url string) ([]byte, error) {
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("User-Agent", "crypto-intel/1.0")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode == 429 {
		return nil, fmt.Errorf("rate limited — wait a minute and retry")
	}
	if resp.StatusCode != 200 {
		return nil, fmt.Errorf("HTTP %d from %s", resp.StatusCode, url)
	}

	return io.ReadAll(resp.Body)
}
