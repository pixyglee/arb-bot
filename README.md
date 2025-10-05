# Solana ↔ CEX Arbitrage Bot

Detects arbitrage opportunities between a Solana DEX (Raydium) and a centralized exchange (Backpack Exchange).

## Features

* Streams live DEX swap data.
* Fetches CEX order book.
* Logs profitable spreads.

## Usage

```bash
cargo run
```

**Note:** Logs opportunities only; does not execute trades. Profits are before fees.

---

## How it works

1. **Listen to the DEX:** Monitor a Solana pool to see how much SOL → USDC you would get.
2. **Poll the CEX:** Check the latest buy and sell prices from Backpack Exchange.
3. **Compare prices:** Determine if DEX output is higher than the CEX ask.
4. **Log opportunity:** If profitable, print “Arb found!” with details.
5. **Repeat continuously:** Loop forever to catch every opportunity.

---
