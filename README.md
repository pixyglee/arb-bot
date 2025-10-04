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

