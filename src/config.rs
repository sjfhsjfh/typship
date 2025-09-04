use std::sync::Arc;

use log::error;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::utils::load_config;

pub static CONFIG: Lazy<Arc<Mutex<Config>>> = Lazy::new(|| {
    Mutex::new(
        load_config()
            .map_err(|e| {
                error!("Failed to load/init the config:\n{e:?}");
                e
            })
            .unwrap(),
    )
    .into()
});

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub tokens: RegistryTokens,
}

// TODO: use a enum or sth to manage
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegistryTokens {
    pub universe: Option<String>,
}
