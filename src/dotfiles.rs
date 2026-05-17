use anyhow::Result;
use std::path::Path;

fn should_exclude(relative_path: &Path, excludes: &[String]) -> bool {
    let path_str = relative_path.to_string_lossy();
    for pattern in excludes {
        if path_str.contains(pattern.as_str()) {
            return true;
        }
    }
    false
}

fn archive_dir(dir: &Path, archive_path: &Path, excludes: &[String]) -> Result<()> {
    let file = std::fs::File::create(archive_path)?;
    let enc = zstd::Encoder::new(file, 3)?;
    let mut archive = tar::Builder::new(enc);

    for entry in walkdir::WalkDir::new(dir).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() && !entry.file_type().is_symlink() {
            continue;
        }
        let relative = entry.path().strip_prefix(dir).unwrap_or(entry.path());
        if should_exclude(relative, excludes) {
            continue;
        }
        if let Ok(mut f) = std::fs::File::open(entry.path()) {
            let _ = archive.append_file(relative, &mut f);
        }
    }

    let _ = archive.finish()?;
    Ok(())
}

pub fn capture(output: &Path, include_ssh: bool, excludes: &[String]) -> Result<()> {
    let dest = output.join("dotfiles");
    std::fs::create_dir_all(&dest)?;
    let home = dirs::home_dir();

    // ~/.config (covers Chromium, Brave, Spotify, VS Code, alacritty, etc.)
    if let Some(config_dir) = dirs::config_dir() {
        if config_dir.exists() {
            archive_dir(&config_dir, &dest.join("config.tar.zst"), excludes)?;
        }
    }

    // Home dotfiles (.bashrc, .zshrc, .profile, .gitconfig, .tmux.conf, .xinitrc)
    if let Some(ref home) = home {
        if home.exists() {
            let file = std::fs::File::create(dest.join("home.tar.zst"))?;
            let enc = zstd::Encoder::new(file, 3)?;
            let mut archive = tar::Builder::new(enc);
            for dotfile in [".bashrc", ".zshrc", ".profile", ".gitconfig", ".tmux.conf", ".xinitrc"] {
                let path = home.join(dotfile);
                if path.exists() {
                    if let Ok(mut f) = std::fs::File::open(&path) {
                        if !should_exclude(Path::new(dotfile), excludes) {
                            let _ = archive.append_file(dotfile, &mut f);
                        }
                    }
                }
            }
            let _ = archive.finish()?;
        }
    }

    // Browser profiles: Firefox (~/.mozilla/firefox/)
    if let Some(ref home) = home {
        let mozilla = home.join(".mozilla");
        if mozilla.exists() {
            archive_dir(&mozilla, &dest.join("mozilla.tar.zst"), excludes)?;
        }
    }

    // SSH keys (opt-in, security warning)
    if include_ssh {
        if let Some(ref home) = home {
            let ssh = home.join(".ssh");
            if ssh.exists() {
                eprintln!("  ⚠  WARNING: Including SSH private keys in the bundle ({})", ssh.display());
                eprintln!("  ⚠  This bundle must be stored securely. Anyone with access can use these keys.");
                archive_dir(&ssh, &dest.join("ssh.tar.zst"), excludes)?;
            }
        }
    }

    Ok(())
}

pub fn apply(bundle_path: &Path, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("dotfiles");
    if !src.exists() {
        println!("  No dotfiles in bundle, skipping");
        return Ok(());
    }

    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    for archive_name in ["config.tar.zst", "home.tar.zst", "mozilla.tar.zst", "ssh.tar.zst"] {
        let archive_path = src.join(archive_name);
        if archive_path.exists() {
            if dry_run {
                println!("  [dry-run] Would extract: {}", archive_name);
            } else {
                let file = std::fs::File::open(&archive_path)?;
                let dec = zstd::Decoder::new(file)?;
                let mut archive = tar::Archive::new(dec);
                archive.unpack(&home)?;
                println!("  ✓ Restored: {}", archive_name);
            }
        }
    }

    Ok(())
}
