use std::{collections::BTreeMap, net::SocketAddr};

use chain_utils::{AnyChain, AnyChainTryFromConfigError, ChainConfigType};
use serde::{Deserialize, Serialize};

use crate::{passes::tx_batch::TxBatch, queue::AnyQueueConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct Config {
    /// Map of chain name to it's respective config.
    pub chain: BTreeMap<String, ChainConfig>,
    pub voyager: VoyagerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyagerConfig {
    pub num_workers: u16,
    pub laddr: SocketAddr,
    pub queue: AnyQueueConfig,
    pub tx_batch: TxBatch,
    #[serde(default)]
    pub optimizer_delay_milliseconds: u64,
}

impl Config {
    pub async fn get_chain(&self, name: &str) -> Result<AnyChain, GetChainError> {
        match self.chain.get(name) {
            Some(config) => Ok(AnyChain::try_from_config(config.ty.clone()).await?),
            None => Err(GetChainError::ChainNotFound {
                name: name.to_string(),
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetChainError {
    #[error("chain `{name}` not found in config")]
    ChainNotFound { name: String },
    #[error("error initializing chain")]
    AnyChainTryFromConfig(#[from] AnyChainTryFromConfigError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub ty: ChainConfigType,
}
