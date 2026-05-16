use crate::packages::PackageEntry;
use anyhow::Result;
use std::process::Command;

pub fn list_manual() -> Result<Vec<PackageEntry>> {
    let output = Command::new("apt-mark")
        .args(["showmanual"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run apt-mark: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("apt-mark showmanual failed: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pkgs: Vec<PackageEntry> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|name| PackageEntry {
            app_name: name.to_string(),
            native_name: name.to_string(),
            pm: "apt".to_string(),
            category: "unknown".to_string(),
        })
        .collect();

    Ok(pkgs)
}
