use std::sync::{Arc, LazyLock};

use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::utils::load_config;

pub static CONFIG: LazyLock<Arc<Mutex<Config>>> = LazyLock::new(|| {
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
