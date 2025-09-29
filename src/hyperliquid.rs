use anyhow::Result;
use ethers::{signers::LocalWallet, types::H160};
use std::env;

pub struct HyperLiquidConfig {
    pub account_address: H160,
    secret_key: String,
}

impl HyperLiquidConfig {
    pub fn new() -> Self {
        Self {
            account_address: Self::get_account_address(),
            secret_key: Self::get_secret_key(),
        }
    }

    fn get_account_address() -> H160 {
        let address =
            env::var("HYPERLIQUID_ACCOUNT_ADDRESS").expect("HYPERLIQUID_ACCOUNT_ADDRESS not set");
        address.parse::<H160>().expect("Invalid account address")
    }

    fn get_secret_key() -> String {
        env::var("HYPERLIQUID_SECRET_KEY").expect("HYPERLIQUID_SECRET_KEY not set")
    }

    pub fn wallet(&self) -> Result<LocalWallet> {
        self.secret_key
            .parse()
            .map_err(|e| anyhow::anyhow!("Error parsing wallet secret key: {}", e))
    }
}
