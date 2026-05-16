pub mod pm_detect;
pub mod pacman;
pub mod apt;
pub mod mapping;

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageManifest {
    pub distro: String,
    pub packages: Vec<PackageEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    pub app_name: String,
    pub native_name: String,
    pub pm: String,
    pub category: String,
}

/// Scan all detected package managers for user-installed packages
pub fn scan_all() -> Result<Vec<PackageEntry>> {
    let pm = pm_detect::detect()?;

    let pkgs = match pm.as_str() {
        "pacman" => pacman::list_explicit()?,
        "apt" => apt::list_manual()?,
        "dnf" => Vec::new(),
        "zypper" => Vec::new(),
        _ => anyhow::bail!("Unsupported package manager: {}", pm),
    };

    Ok(pkgs)
}

pub fn write_manifest(path: &Path, pkgs: &[PackageEntry]) -> Result<()> {
    let pkg_dir = path.join("packages");
    std::fs::create_dir_all(&pkg_dir)?;
    let manifest = PackageManifest {
        distro: crate::distro::detect()?.id,
        packages: pkgs.to_vec(),
    };
    std::fs::write(pkg_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;
    Ok(())
}

pub fn read_manifest(path: &Path) -> Result<Vec<PackageEntry>> {
    let content = std::fs::read_to_string(path.join("packages").join("manifest.json"))?;
    let manifest: PackageManifest = serde_json::from_str(&content)?;
    Ok(manifest.packages)
}

pub fn install_all(mapped: &[mapping::MappingResult], dry_run: bool) -> Result<()> {
    let mut success = 0;
    let mut skipped = 0;

    for m in mapped {
        if m.target.is_empty() {
            println!("  ⚠  No mapping for '{}', skipping", m.app_name);
            skipped += 1;
            continue;
        }

        // Take the highest-priority mapping
        let target = &m.target[0];
        if target.name.is_empty() {
            println!("  ⚠  No install target for '{}', skipping", m.app_name);
            skipped += 1;
            continue;
        }

        if dry_run {
            println!("  [dry-run] Would install: {} via {} ({})", target.name, target.pm, m.app_name);
            success += 1;
            continue;
        }

        let result = match target.pm.as_str() {
            "apt" => crate::util::sudo_cmd(&format!("apt install -y {}", target.name)),
            "pacman" => crate::util::sudo_cmd(&format!("pacman -S --noconfirm {}", target.name)),
            "dnf" => crate::util::sudo_cmd(&format!("dnf install -y {}", target.name)),
            "zypper" => crate::util::sudo_cmd(&format!("zypper install -y {}", target.name)),
            "flatpak" => crate::util::run_cmd(&format!("flatpak install -y {}", target.name)),
            _ => {
                println!("  ⚠  Unknown package manager '{}', skipping {}", target.pm, target.name);
                skipped += 1;
                continue;
            }
        };

        match result {
            Ok(_) => {
                println!("  ✓ Installed: {}", target.name);
                success += 1;
            }
            Err(e) => {
                eprintln!("  ✗ Failed to install {}: {}", target.name, e);
                skipped += 1;
            }
        }
    }

    println!("  Packages: {} installed, {} skipped", success, skipped);
    Ok(())
}
