# Crypto Intelligence CLI — Landing Page Copy

**URL:** cryptointel.dev (or crypto-intel.lemonsqueezy.com)

---

## Hero Section

**Headline:**
Crypto intelligence in your terminal.

**Subheadline:**
Cross-exchange arbitrage. Real-time signals. Portfolio P&L. All from one CLI powered by CoinGecko, altFINS, and ChangeNOW.

**CTA:** Start free — $49/month for unlimited

---

## What It Does

```bash
# Find arbitrage opportunities right now
$ crypto-intel arb --min-spread 2 --liquid
Coin       Price      Spread
AVAX       $38.2102   3.4%
LINK       $14.8803   2.8%
SOL        $141.9901  2.1%

# Should I buy ETH?
$ crypto-intel signal ETH --explain
=== ETH Signal ===
Price:     $3,241.80
24h:       +4.2%
RSI (14):  38.2
MACD:      -12.40
Signal:    Potential buy

Interpretation:
  RSI < 40: Approaching oversold — watch for reversal
  MACD negative — still bearish, wait for crossover

# What's my portfolio worth since I bought?
$ crypto-intel portfolio --cost-basis BTC:48000:2,ETH:2100:10
Coin     Bought@      Now@         P&L $        P&L %
BTC      $48,000.00   $62,100.00   +$28,200.00  +29.4%
ETH      $2,100.00    $3,241.80    +$11,418.00  +54.4%
──────────────────────────────────────────────────────
TOTAL    $117,000.00  $156,618.00  +$39,618.00  +33.9%

# Bitcoin season or alt season?
$ crypto-intel market-phase
BTC Dominance:  54.3%
Phase:          Transition — market rotating
Strategy:       Watch for alt season breakout

Total Market Cap: $2.41T
24h Volume:       $98.20B

# Best swap rate: 1 BTC → USDC
$ crypto-intel swap BTC USDC 1 --best-rate
=== Swap Quote: 1.0000 BTC → USDC ===
You receive:  62,034.500000 USDC
Rate:         1 BTC = 62,034.500000 USDC
Estimated:    ~10 minutes
```

---

## Why This CLI Wins

**These queries are impossible with any single API.**

- CoinGecko alone gives you price data — no signals, no arbitrage detection
- altFINS alone gives you signals — no cross-exchange price comparison
- ChangeNOW alone gives you swap quotes — no portfolio tracking

The CLI stitches all three together into compound commands. One query, multiple data sources, one answer.

**And it's all free data.** You're paying for the compiled intelligence and the UX.

---

## Pricing

**Free**
- 10 queries/day
- All commands
- No credit card

**Pro — $49/month**
- Unlimited queries
- Priority support
- Early access to new commands

**Annual — $399/year** (2 months free)

---

## FAQ

**Q: Do I need API keys?**
A: CoinGecko works without a key. altFINS and ChangeNOW offer free keys (1,000 credits/mo and no limits respectively). The CLI tells you exactly what to set.

**Q: Does this give financial advice?**
A: No. Signals are technical indicators — RSI, MACD, momentum. Use them as data points, not decisions. Always do your own research.

**Q: What platforms?**
A: Linux, macOS, Windows — amd64 and arm64.

**Q: How do I install it?**
A: `brew install cryptointel/tap/crypto-intel` or download from GitHub Releases.

**Q: What's the refund policy?**
A: 7-day full refund, no questions.

---

## Installation

```bash
# macOS
brew install cryptointel/tap/crypto-intel

# Linux
curl -sSL https://cryptointel.dev/install.sh | bash

# Windows (PowerShell)
iwr https://cryptointel.dev/install.ps1 | iex

# Verify
crypto-intel --version
```

**First run:**
```bash
# Optional: set your free altFINS API key for trading signals
export ALTFINS_API_KEY=your-key-here

# Try it
crypto-intel signal BTC --explain
```

---

## Distribution Checklist

- [ ] Register `cryptointel.dev` (or `crypto-intel.dev`)
- [ ] Set up Homebrew tap: `github.com/[username]/homebrew-tap`
- [ ] Create LemonSqueezy product (run `scripts/setup-lemonsqueezy.sh`)
- [ ] Post to:
  - r/algotrading: "I built a crypto intelligence CLI that stitches CoinGecko + altFINS + ChangeNOW"
  - r/CryptoTechnology: "Open-source crypto CLI: arbitrage, signals, swaps from the terminal"
  - Hacker News: "Show HN: Crypto Intelligence CLI — arbitrage and signals in one command"
  - Product Hunt: "crypto-intel — cross-exchange arbitrage and trading signals from your terminal"
  - Twitter/X: Tag CoinGecko, altFINS for retweets
