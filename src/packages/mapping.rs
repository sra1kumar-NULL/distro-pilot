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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_firefox_arch_to_debian() {
        let result = lookup("firefox", "arch", "debian").unwrap();
        assert_eq!(result.app_name, "firefox");
        assert!(result.target.iter().any(|t| t.name == "firefox-esr"));
        assert!(result.target.iter().any(|t| t.pm == "debian"));
    }

    #[test]
    fn test_lookup_arch_to_arch() {
        let result = lookup("firefox", "arch", "arch").unwrap();
        let top = &result.target[0];
        assert_eq!(top.name, "firefox");
        assert_eq!(top.pm, "arch");
    }

    #[test]
    fn test_lookup_fallback_to_flatpak() {
        let result = lookup("slack", "arch", "debian").unwrap();
        assert!(result.target.iter().any(|t| t.pm == "flatpak"));
    }

    #[test]
    fn test_lookup_unknown_package() {
        let result = lookup("this-package-definitely-does-not-exist-42", "arch", "debian");
        assert!(result.is_err());
    }

    #[test]
    fn test_pm_for_distro() {
        assert_eq!(pm_for_distro("arch"), Some("pacman"));
        assert_eq!(pm_for_distro("debian"), Some("apt"));
        assert_eq!(pm_for_distro("fedora"), Some("dnf"));
        assert_eq!(pm_for_distro("opensuse"), Some("zypper"));
        assert_eq!(pm_for_distro("unknown"), None);
    }

    #[test]
    fn test_same_pm_family() {
        assert!(same_pm_family("arch", "cachyos"));
        assert!(same_pm_family("fedora", "rhel"));
        assert!(same_pm_family("debian", "ubuntu"));
    }

    #[test]
    fn test_different_pm_family() {
        assert!(!same_pm_family("arch", "debian"));
        assert!(!same_pm_family("fedora", "ubuntu"));
    }

    #[test]
    fn test_map_all_same_family_fallback() {
        let pkgs = vec![
            PackageEntry {
                app_name: "firefox".to_string(),
                native_name: "firefox".to_string(),
                pm: "pacman".to_string(),
                category: "unknown".to_string(),
            },
            PackageEntry {
                app_name: "some-obscure-tool".to_string(),
                native_name: "some-obscure-tool".to_string(),
                pm: "pacman".to_string(),
                category: "unknown".to_string(),
            },
        ];
        // arch -> cachyos (same pacman family): obscure tool passes through
        let results = map_all(&pkgs, "arch", "cachyos").unwrap();
        assert_eq!(results.len(), 2);
        let obscure = results.iter().find(|r| r.app_name == "some-obscure-tool").unwrap();
        assert_eq!(obscure.target[0].name, "some-obscure-tool");
        assert_eq!(obscure.target[0].pm, "pacman");
        // firefox should have proper mapping
        let ff = results.iter().find(|r| r.app_name == "firefox").unwrap();
        assert!(!ff.target.is_empty());
    }
}

fn pm_for_distro(distro_id: &str) -> Option<&'static str> {
    match distro_id {
        "arch" | "manjaro" | "cachyos" | "endeavouros" => Some("pacman"),
        "debian" | "ubuntu" | "linuxmint" | "pop" => Some("apt"),
        "fedora" | "rhel" | "centos" | "rocky" | "alma" => Some("dnf"),
        "opensuse" | "suse" | "opensuse-tumbleweed" | "opensuse-leap" => Some("zypper"),
        _ => None,
    }
}

fn same_pm_family(from: &str, to: &str) -> bool {
    pm_for_distro(from) == pm_for_distro(to) && pm_for_distro(from).is_some()
}

/// Map all packages in a manifest from source distro to target distro
pub fn map_all(pkgs: &[PackageEntry], from: &str, to: &str) -> Result<Vec<MappingResult>> {
    let mut results = Vec::new();
    for pkg in pkgs {
        if pkg.pm == "flatpak" {
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
            Err(_) if same_pm_family(from, to) => {
                let pm = pm_for_distro(to).unwrap_or("apt");
                results.push(MappingResult {
                    app_name: pkg.app_name.clone(),
                    source: format!("{}:{}", from, pkg.native_name),
                    target: vec![TargetPackage {
                        pm: pm.to_string(),
                        name: pkg.native_name.clone(),
                        priority: 1,
                    }],
                });
            }
            Err(e) => eprintln!("  ⚠  {} — skipping", e),
        }
    }
    Ok(results)
}
