use anyhow::Result;

pub fn detect() -> Result<String> {
    let distro = crate::distro::detect()?;

    let candidates: &[(&str, &[&str])] = &[
        ("arch", &["pacman"]),
        ("manjaro", &["pacman"]),
        ("cachyos", &["pacman"]),
        ("endeavouros", &["pacman"]),
        ("debian", &["apt"]),
        ("ubuntu", &["apt"]),
        ("linuxmint", &["apt"]),
        ("pop", &["apt"]),
        ("fedora", &["dnf"]),
        ("rhel", &["dnf"]),
        ("centos", &["dnf"]),
        ("opensuse", &["zypper"]),
        ("suse", &["zypper"]),
    ];

    for (distro_id, pms) in candidates {
        if distro.id == *distro_id || distro.id_like.contains(distro_id) {
            for pm in *pms {
                if which(pm).is_some() {
                    return Ok(pm.to_string());
                }
            }
        }
    }

    // Fallback: try each binary directly
    for pm in &["pacman", "apt", "dnf", "zypper"] {
        if which(pm).is_some() {
            return Ok(pm.to_string());
        }
    }

    anyhow::bail!("No supported package manager found (checked: pacman, apt, dnf, zypper)")
}

fn which(cmd: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|_| cmd.to_string())
}
