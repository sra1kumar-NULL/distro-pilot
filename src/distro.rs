use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub id: String,
    pub id_like: String,
    pub version_id: Option<String>,
    pub name: String,
    pub kernel: String,
}

pub fn detect() -> Result<DistroInfo> {
    let os_release = std::fs::read_to_string("/etc/os-release")
        .map_err(|e| anyhow::anyhow!("Cannot read /etc/os-release: {}. Are you on Linux?", e))?;

    let mut map = HashMap::new();
    for line in os_release.lines() {
        if let Some((key, val)) = line.split_once('=') {
            let val = val.trim_matches('"');
            map.insert(key.to_string(), val.to_string());
        }
    }

    let kernel = std::fs::read_to_string("/proc/version")
        .unwrap_or_default()
        .trim()
        .to_string();

    Ok(DistroInfo {
        id: map.get("ID").cloned().unwrap_or_default(),
        id_like: map.get("ID_LIKE").cloned().unwrap_or_default(),
        version_id: map.get("VERSION_ID").cloned(),
        name: map.get("NAME").cloned().unwrap_or_default(),
        kernel,
    })
}
