# Polymarket 15-minute trading bot (dump-and-hedge)

A small **Rust** bot that watches Polymarket **15-minute Up/Down** markets (for example BTC, ETH, SOL, XRP), runs a **dump-and-hedge** strategy, and can place orders on the Polymarket CLOB when you enable production mode.

- **Repository:** [github.com/Poly-Tutor/Polymarket-15min-arbitrage-bot](https://github.com/Poly-Tutor/Polymarket-15min-arbitrage-bot) — clone with  
  `git clone https://github.com/Poly-Tutor/Polymarket-15min-arbitrage-bot.git`
- **Telegram:** [@AlterEgo_Eth](https://t.me/AlterEgo_Eth)

If you are new to Rust or Polymarket, read [Quick start for beginners](#quick-start-for-beginners) first, then [Trading strategy explained](#trading-strategy-explained).

---

## What you need before you start

1. **[Rust](https://www.rust-lang.org/tools/install)** (stable toolchain).
2. A **Polymarket** account and wallet setup the way Polymarket expects (often a proxy wallet on Polygon).
3. **USDC on Polygon** if you will trade for real (production mode).
4. **API credentials** from Polymarket for the CLOB (API key, secret, passphrase) where required.
5. A **private key** that can sign orders for your trading account (stored only in your local `config.json`—never commit real keys).

**Important:** Trading prediction markets is risky. This software does not guarantee profit. Start in **simulation** mode until you understand behavior and costs.

---

## How to run bot


https://github.com/user-attachments/assets/1d008d5b-b182-4b14-bf5c-76f530b613da


---

## Quick start for beginners

### 1. Clone and build

```bash
git clone https://github.com/Poly-Tutor/Polymarket-15min-arbitrage-bot.git
cd Polymarket-15min-arbitrage-bot
cargo build --release
```

The binary is at `target/release/polymarket-arbitrage-bot` (or `polymarket-arbitrage-bot.exe` on Windows).

### 2. Configuration file

On first run, if `config.json` is missing, the program **creates a default** `config.json` in the current directory. Edit it with your settings.

- Keep **`config.json` out of git** if it contains secrets (add it to `.gitignore` if needed).
- Use **`"markets": ["btc"]`** (or `eth`, `sol`, `xrp`) to choose which 15m assets to watch.

### 3. Run in simulation (recommended first)

Set **`"simulation": true`** under `trading` in `config.json` (this is the default when the bot creates a new file). In simulation mode the bot still **reads live prices** from Polymarket, but it **does not place orders** or **send on-chain redemptions**.

```bash
cargo run --release
```

To **force** paper mode for one run (for example if `simulation` is `false` in config), pass:

```bash
cargo run --release -- --simulation
```

### 4. Run in production (real trades)

Only after you understand the strategy and have funded your account appropriately. Either set **`"simulation": false`** in `config.json` or pass **`--production`** (CLI wins over config when used):

```bash
cargo run --release -- --production
```

Production mode uses your `private_key` and Polymarket API settings to **authenticate, place real orders**, and **redeem** winning positions on-chain when applicable.

### 5. Logs

- Console output goes to **stderr**.
- A log/history file **`history.toml`** is appended in the working directory (despite the name, it is used as a rolling log file).

Optional: set log level with the `RUST_LOG` environment variable, for example `RUST_LOG=debug`.

---

## Command-line options

| Option | Meaning |
|--------|---------|
| `--simulation` | Force paper mode for this run (overrides `trading.simulation` in config). Cannot be combined with `--production`. |
| `--production` | Force live trading for this run (overrides `trading.simulation`). Cannot be combined with `--simulation`. |
| `-c`, `--config <path>` | Path to `config.json` (default: `config.json`). |

If neither flag is passed, the mode comes from **`trading.simulation`** in `config.json` (default **`true`**).

Examples:

```bash
# Custom config path
cargo run --release -- --config my-config.json

# Production with explicit config
cargo run --release -- --production --config config.json
```

---

## `config.json` overview

### `polymarket`

| Field | Purpose |
|-------|---------|
| `gamma_api_url` | Gamma API base URL (default: `https://gamma-api.polymarket.com`). |
| `clob_api_url` | CLOB API base URL (default: `https://clob.polymarket.com`). |
| `api_key`, `api_secret`, `api_passphrase` | Polymarket CLOB API credentials. |
| `private_key` | Hex private key used to sign orders (required for real trading). |
| `proxy_wallet_address` | If you use Polymarket’s proxy wallet, set your funder/proxy address here. |
| `signature_type` | `0` = EOA, `1` = Proxy, `2` = Gnosis Safe—must match how your Polymarket account is set up. |

### `trading`

| Field | Typical meaning |
|-------|-----------------|
| `simulation` | **`true`** (default): paper trading only—no orders or redemptions. **`false`**: live trading when not overridden by `--simulation` / `--production`. |
| `check_interval_ms` | How often prices are polled when using API mode (milliseconds). |
| `market_closure_check_interval_seconds` | How often the bot checks closed markets for settlement / redemption logic. |
| `data_source` | `"api"` (HTTP polling) or `"websocket"` (CLOB market WebSocket, with fallback). |
| `markets` | List of assets, e.g. `["btc", "eth"]`. Supported slug prefixes: `btc`, `eth`, `sol`, `xrp`. |
| `dump_hedge_shares` | Share size per leg (see strategy below). |
| `dump_hedge_sum_target` | Max combined price for both legs to lock a hedge (e.g. `0.95`). |
| `dump_hedge_move_threshold` | Minimum drop in a short window to count as a “dump” (e.g. `0.15` = 15%). |
| `dump_hedge_window_minutes` | Only the **first N minutes** of each 15m period are used to detect dumps. |
| `dump_hedge_stop_loss_max_wait_minutes` | If the hedge condition is not met after this many minutes from leg 1, a **stop-loss hedge** (buy opposite side) runs. |
| `dump_hedge_stop_loss_percentage` | Present in config for future use; **time-based** stop loss is what the code uses today. |

---

## Trading strategy explained

The bot implements a **dump-and-hedge** flow on **binary 15-minute Up/Down** markets:

1. **Each 15-minute round** has an Up token and a Down token. Prices move as traders buy and sell.
2. **Watch window:** For the first `dump_hedge_window_minutes` of the round, the bot watches prices.
3. **Dump detection:** If either side’s price **falls quickly** by at least `dump_hedge_move_threshold` (for example 15%) within a short time window, the bot treats that as a **dump** and **buys that side** (leg 1) for `dump_hedge_shares` shares (in production: market-style buy via the CLOB).
4. **Hedge (leg 2):** The bot waits until **Up ask + Down ask** (using the opposite side’s ask after leg 1) is **≤ `dump_hedge_sum_target`**. Then it buys the **opposite** side for the same share size so that the **combined cost per paired share** is below $1.00 in principle—similar in spirit to classic “pair cost under one dollar per share pair” ideas on binary markets (fees and execution can change outcomes).
5. **Stop-loss hedge:** If that condition is **not** met before `dump_hedge_stop_loss_max_wait_minutes`, the bot still buys the opposite side to **close exposure** (you may still lose money depending on prices and fees).

After the market **closes**, the bot can use resolution data and, in production, attempt **on-chain redemption** of winning outcome tokens via the configured Polygon setup.

**Simulation mode** runs the same logic but **does not** send orders or chain transactions.

---

## Mental model (simple diagram)

```text
New 15m period
    │
    ▼
First N minutes: watch for sharp price drop on Up or Down
    │
    ├─► Dump on one side? ──► Buy that side (leg 1)
    │                              │
    │                              ▼
    │                    Wait: leg1_price + opposite_ask <= sum_target?
    │                              │
    │                    Yes ──────► Buy opposite (leg 2), cycle complete
    │                              │
    │                    No and too long? ──► Stop-loss: buy opposite anyway
    │
    └─► No dump in window ──► No trade this cycle (for that leg logic)
```

---

## Risks and limitations

- **Market risk:** Prices move fast; slippage and fees can make “theoretical” edges negative.
- **Execution risk:** Orders may fail or partially fill; WebSocket mode can fall back to polling.
- **Key security:** Anyone with your `private_key` can move funds. Protect `config.json`.
- **Regulatory / tax:** You are responsible for compliance in your jurisdiction.

---

## Troubleshooting (short)

- **“No markets configured”** — Add at least one entry to `trading.markets` in `config.json`.
- **“Unsupported asset”** — Use only `btc`, `eth`, `sol`, or `xrp` (case-insensitive in practice).
- **Authentication or order errors** — Check API credentials, `private_key`, proxy wallet address, and `signature_type` against your Polymarket account type.
- **No trades in simulation** — Dumps are rare; thresholds may be too strict, or the watch window may have passed for the current period.

---

## License, disclaimer, and contact

This project is provided **as-is** for education and experimentation. It is **not** financial advice. Use at your own risk.

**Questions or feedback:** reach out on Telegram at [@AlterEgo_Eth](https://t.me/AlterEgo_Eth).  
**Source code:** [Poly-Tutor/Polymarket-15min-arbitrage-bot](https://github.com/Poly-Tutor/Polymarket-15min-arbitrage-bot) (`https://github.com/Poly-Tutor/Polymarket-15min-arbitrage-bot.git`).
