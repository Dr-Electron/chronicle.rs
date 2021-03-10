use permanode_api::ApiConfig;
use permanode_broker::BrokerConfig;
use permanode_storage::StorageConfig;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    borrow::Cow,
    path::Path,
};

pub const CONFIG_PATH: &str = "./config.ron";

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Config {
    pub storage_config: StorageConfig,
    pub api_config: ApiConfig,
    pub broker_config: BrokerConfig,
}

impl Config {
    pub fn load() -> Result<Config, Cow<'static, str>> {
        let path = std::env::var("CONFIG_PATH").unwrap_or(CONFIG_PATH.to_string());
        let path = Path::new(&path);
        match std::fs::File::open(path) {
            Ok(f) => ron::de::from_reader(f).map_err(|e| e.to_string().into()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    let config = Self::default();
                    config.save()?;
                    Ok(config)
                }
                _ => Err(e.to_string().into()),
            },
        }
    }

    pub fn save(&self) -> Result<(), Cow<'static, str>> {
        let path = std::env::var("CONFIG_PATH").unwrap_or(CONFIG_PATH.to_string());
        let path = Path::new(&path);
        let f = std::fs::File::create(path).map_err(|e| Cow::from(e.to_string()))?;
        ron::ser::to_writer_pretty(f, self, ron::ser::PrettyConfig::default()).map_err(|e| e.to_string().into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;
    use permanode_storage::{
        DatacenterConfig,
        KeyspaceConfig,
        ThreadCount,
    };

    #[test]
    pub fn example_config() {
        let config = Config {
            storage_config: StorageConfig {
                keyspaces: vec![KeyspaceConfig {
                    name: "my_mainnet".to_string(),
                    data_centers: hashmap! {
                        "USA".to_string() => DatacenterConfig {
                            replication_factor: 2,
                        },
                        "Canada".to_string() => DatacenterConfig {
                            replication_factor: 1,
                        },
                    },
                }],
                listen_address: "127.0.0.1:8080".to_owned(),
                thread_count: ThreadCount::CoreMultiple(1),
                reporter_count: 2,
                local_datacenter: "datacenter1".to_owned(),
            },
            api_config: ApiConfig {},
            broker_config: BrokerConfig {},
        };

        std::env::set_var("CONFIG_PATH", "../config.example.ron");

        let mut deserialized_config = Config::load().expect("Failed to deserialize example config!");

        if config != deserialized_config {
            config.save().expect("Failed to serialize example config!");
            deserialized_config = Config::load().expect("Failed to deserialize example config!");
        }

        assert_eq!(config, deserialized_config);
    }
}