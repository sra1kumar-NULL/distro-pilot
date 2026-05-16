use crate::packages::PackageEntry;
use anyhow::Result;
use std::process::Command;

pub fn list_apps() -> Result<Vec<PackageEntry>> {
    let output = Command::new("flatpak")
        .args(["list", "--app", "--columns=application"])
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to run flatpak: {}", e))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pkgs: Vec<PackageEntry> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|name| PackageEntry {
            app_name: name.to_string(),
            native_name: name.to_string(),
            pm: "flatpak".to_string(),
            category: "flatpak".to_string(),
        })
        .collect();

    Ok(pkgs)
}
