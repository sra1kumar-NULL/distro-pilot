use serde::{Deserialize, Serialize};
use crate::distro::DistroInfo;
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub distro: DistroInfo,
    pub created_at: String,
    pub version: String,
}

impl Bundle {
    pub fn new(distro: &DistroInfo) -> Self {
        Self {
            distro: distro.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn write_manifest(&self, path: &Path) -> Result<()> {
        let toml_str = toml::to_string_pretty(self)?;
        std::fs::write(path.join("manifest.toml"), toml_str)?;
        Ok(())
    }

    pub fn read(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path.join("manifest.toml"))?;
        Ok(toml::from_str(&content)?)
    }
}
