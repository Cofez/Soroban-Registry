use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Testnet,
    Futurenet,
    Auto, // Issue #78: Added Auto routing variant
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Testnet => write!(f, "testnet"),
            Network::Futurenet => write!(f, "futurenet"),
            Network::Auto => write!(f, "auto"), // Issue #78
        }
    }
}

impl FromStr for Network {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            "futurenet" => Ok(Network::Futurenet),
            "auto" => Ok(Network::Auto), // Issue #78: Allow "auto" string
            _ => anyhow::bail!("Invalid network: {}. Allowed values: mainnet, testnet, futurenet, auto", s),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    network: Option<String>,
}

pub fn resolve_network(cli_flag: Option<String>) -> Result<Network> {
    // 1. CLI Flag
    if let Some(net_str) = cli_flag {
        return net_str.parse::<Network>();
    }

    // 2. Config File
    if let Some(config_path) = config_file_path() {
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file at {:?}", config_path))?;
            
            let config: ConfigFile = toml::from_str(&content)
                .with_context(|| "Failed to parse config file")?;

            if let Some(net_str) = config.network {
                return net_str.parse::<Network>();
            }
        }
    }

    // 3. Default
    Ok(Network::Testnet)
}

fn config_file_path() -> Option<PathBuf> {
    dirs::home_dir().map(|mut p| {
        p.push(".soroban-registry.toml");
        p
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_network_parsing() {
        assert_eq!("mainnet".parse::<Network>().unwrap(), Network::Mainnet);
        assert_eq!("testnet".parse::<Network>().unwrap(), Network::Testnet);
        assert_eq!("futurenet".parse::<Network>().unwrap(), Network::Futurenet);
        assert_eq!("auto".parse::<Network>().unwrap(), Network::Auto); // Issue #78
        assert_eq!("Mainnet".parse::<Network>().unwrap(), Network::Mainnet); // Case insensitive
        assert!("invalid".parse::<Network>().is_err());
    }

        // Note: Integration tests involving file system would require mocking or temporary files.
    // Given the constraints and the environment, we focus on unit tests for parsing here.
    // `resolve_network` with file interaction is harder to test in isolation without dependency injection or mocking `dirs` / `fs`.
}
