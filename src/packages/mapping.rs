use crate::packages::PackageEntry;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

const BUILTIN_MAPPINGS: &str = include_str!("../../mappings/packages.json");

#[derive(Debug, serde::Serialize)]
pub struct MappingResult {
    pub app_name: String,
    pub source: String,
    pub target: Vec<TargetPackage>,
}

#[derive(Debug, serde::Serialize)]
pub struct TargetPackage {
    pub pm: String,
    pub name: String,
    pub priority: u8,
}

/// Map a single app_name from source distro to target distro
pub fn lookup(app_name: &str, from: &str, to: &str) -> Result<MappingResult> {
    let db: HashMap<String, Value> = serde_json::from_str(BUILTIN_MAPPINGS)
        .map_err(|e| anyhow::anyhow!("Failed to parse mapping DB: {}", e))?;

    let entry = db.get(app_name)
        .ok_or_else(|| anyhow::anyhow!("'{}' not found in mapping DB", app_name))?;

    let mut targets = Vec::new();

    // Direct distro match
    if let Some(direct) = entry.get(to) {
        if let Some(s) = direct.as_str() {
            if !s.is_empty() {
                targets.push(TargetPackage { pm: to.to_string(), name: s.to_string(), priority: 10 });
            }
        }
    }

    // Check id_like fallbacks
    let id_like_map: HashMap<&str, &[&str]> = [
        ("debian", &["ubuntu", "linuxmint", "pop"] as &[&str]),
        ("ubuntu", &["debian"]),
        ("fedora", &["rhel", "centos"]),
        ("arch", &["manjaro", "cachyos", "endeavouros"]),
    ].iter().cloned().collect();

    if let Some(alternatives) = id_like_map.get(to) {
        for alt in *alternatives {
            if targets.iter().any(|t| t.pm == *alt) { continue; }
            if let Some(val) = entry.get(*alt) {
                if let Some(s) = val.as_str() {
                    if !s.is_empty() {
                        targets.push(TargetPackage { pm: alt.to_string(), name: s.to_string(), priority: 8 });
                    }
                }
            }
        }
    }

    // Flatpak fallback
    if let Some(fp) = entry.get("flatpak") {
        if let Some(s) = fp.as_str() {
            if !s.is_empty() {
                targets.push(TargetPackage { pm: "flatpak".to_string(), name: s.to_string(), priority: 5 });
            }
        }
    }

    if targets.is_empty() {
        anyhow::bail!("No mapping found for '{}' to '{}'", app_name, to);
    }

    targets.sort_by(|a, b| b.priority.cmp(&a.priority));

    let source_name = entry.get(from)
        .and_then(|v| v.as_str())
        .unwrap_or(app_name);

    Ok(MappingResult {
        app_name: app_name.to_string(),
        source: format!("{}:{}", from, source_name),
        target: targets,
    })
}

/// Map all packages in a manifest from source distro to target distro
pub fn map_all(pkgs: &[PackageEntry], from: &str, to: &str) -> Result<Vec<MappingResult>> {
    let mut results = Vec::new();
    for pkg in pkgs {
        if pkg.pm == "flatpak" {
            // Flatpak entries are already the canonical install ID; pass through directly
            results.push(MappingResult {
                app_name: pkg.app_name.clone(),
                source: format!("flatpak:{}", pkg.native_name),
                target: vec![TargetPackage {
                    pm: "flatpak".to_string(),
                    name: pkg.native_name.clone(),
                    priority: 10,
                }],
            });
            continue;
        }
        match lookup(&pkg.app_name, from, to) {
            Ok(r) => results.push(r),
            Err(e) => eprintln!("  ⚠  {} — skipping", e),
        }
    }
    Ok(results)
}
