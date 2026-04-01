use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Force simulation (paper) mode: no real orders or on-chain txs (overrides config)
    #[arg(long, default_value_t = false, conflicts_with = "production")]
    pub simulation: bool,

    /// Production mode: real trades and redemptions (overrides config)
    #[arg(long, default_value_t = false, conflicts_with = "simulation")]
    pub production: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "config.json")]
    pub config: PathBuf,
}

impl Args {
    /// Resolve mode: CLI wins over `config.trading.simulation` when a flag is set.
    pub fn is_simulation(&self, config_simulation: bool) -> bool {
        if self.production {
            false
        } else if self.simulation {
            true
        } else {
            config_simulation
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub polymarket: PolymarketConfig,
    pub trading: TradingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolymarketConfig {
    pub gamma_api_url: String,
    pub clob_api_url: String,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>,
    pub private_key: Option<String>,
    pub proxy_wallet_address: Option<String>,
    pub signature_type: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// When true (default), the bot does not place orders or send redemption txs.
    #[serde(default = "default_simulation")]
    pub simulation: bool,
    pub check_interval_ms: u64,
    #[serde(default = "default_market_closure_check_interval")]
    pub market_closure_check_interval_seconds: u64,
    #[serde(default = "default_data_source")]
    pub data_source: String,
    #[serde(default = "default_markets")]
    pub markets: Vec<String>,
    pub dump_hedge_shares: Option<f64>,
    pub dump_hedge_sum_target: Option<f64>,
    pub dump_hedge_move_threshold: Option<f64>,
    pub dump_hedge_window_minutes: Option<u64>,
    pub dump_hedge_stop_loss_max_wait_minutes: Option<u64>,
    pub dump_hedge_stop_loss_percentage: Option<f64>,
}

fn default_simulation() -> bool {
    true
}

fn default_market_closure_check_interval() -> u64 {
    20
}

fn default_data_source() -> String {
    "api".to_string()
}

fn default_markets() -> Vec<String> {
    vec!["btc".to_string()]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            polymarket: PolymarketConfig {
                gamma_api_url: "https://gamma-api.polymarket.com".to_string(),
                clob_api_url: "https://clob.polymarket.com".to_string(),
                api_key: None,
                api_secret: None,
                api_passphrase: None,
                private_key: None,
                proxy_wallet_address: None,
                signature_type: None,
            },
            trading: TradingConfig {
                simulation: true,
                check_interval_ms: 1000,
                market_closure_check_interval_seconds: 20,
                data_source: "api".to_string(),
                markets: vec!["btc".to_string()],
                dump_hedge_shares: Some(10.0),
                dump_hedge_sum_target: Some(0.95),
                dump_hedge_move_threshold: Some(0.15),
                dump_hedge_window_minutes: Some(2),
                dump_hedge_stop_loss_max_wait_minutes: Some(5),
                dump_hedge_stop_loss_percentage: Some(0.20),
            },
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Config::default();
            let content = serde_json::to_string_pretty(&config)?;
            std::fs::write(path, content)?;
            Ok(config)
        }
    }
}

