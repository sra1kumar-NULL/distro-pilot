use crate::distro::DistroInfo;
use anyhow::Result;
use std::path::Path;

const CONFIG_DIRS: &[&str] = &[
    "/etc/modprobe.d",
    "/etc/modules-load.d",
    "/etc/udev/rules.d",
    "/etc/sysctl.d",
];

pub fn source_dirs() -> &'static [&'static str] {
    CONFIG_DIRS
}

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("drivers");
    std::fs::create_dir_all(&dest)?;

    for dir in CONFIG_DIRS {
        let src = Path::new(dir);
        if src.exists() {
            let rel = src.strip_prefix("/etc/").unwrap_or(src);
            let target = dest.join(rel);
            std::fs::create_dir_all(&target)?;
            for entry in walkdir::WalkDir::new(src) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let relative = entry.path().strip_prefix(src)?;
                    if let Err(e) = std::fs::copy(entry.path(), target.join(relative)) {
                        eprintln!("  ⚠  Could not copy {}: {}", entry.path().display(), e);
                    }
                }
            }
        }
    }

    // Capture /etc/default/grub
    let grub_src = Path::new("/etc/default/grub");
    if grub_src.exists() {
        let grub_dest = dest.join("grub");
        std::fs::create_dir_all(&grub_dest)?;
        std::fs::copy(grub_src, grub_dest.join("grub"))?;
    }

    // Capture /etc/environment
    let env_src = Path::new("/etc/environment");
    if env_src.exists() {
        std::fs::copy(env_src, dest.join("environment"))?;
    }

    Ok(())
}

pub fn apply(bundle_path: &Path, _target: &DistroInfo, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("drivers");
    if !src.exists() {
        println!("  No driver configs in bundle, skipping");
        return Ok(());
    }

    for dir in CONFIG_DIRS {
        let rel = dir.strip_prefix("/etc/").unwrap_or(dir);
        let bundle_dir = src.join(rel);
        if bundle_dir.exists() {
            let target_dir = Path::new(dir);
            if dry_run {
                println!("  [dry-run] Would copy configs to: {}", dir);
                for entry in walkdir::WalkDir::new(&bundle_dir) {
                    let e = entry?;
                    if e.file_type().is_file() {
                        let relative = e.path().strip_prefix(&bundle_dir)?;
                        println!("         {}/{}", dir, relative.display());
                    }
                }
            } else {
                std::fs::create_dir_all(target_dir)?;
                for entry in walkdir::WalkDir::new(&bundle_dir) {
                    let e = entry?;
                    if e.file_type().is_file() {
                        let relative = e.path().strip_prefix(&bundle_dir)?;
                        let dest = target_dir.join(relative);
                        std::fs::create_dir_all(dest.parent().unwrap())?;
                        std::fs::copy(e.path(), &dest)?;
                        println!("  ✓ {} restored", dest.display());
                    }
                }
            }
        }
    }

    // Handle /etc/environment
    let bundle_env = src.join("environment");
    if bundle_env.exists() {
        let target_env = Path::new("/etc/environment");
        if dry_run {
            println!("  [dry-run] Would restore /etc/environment");
        } else {
            std::fs::copy(&bundle_env, target_env)?;
            println!("  ✓ /etc/environment restored");
        }
    }

    // Handle GRUB kernel params merge
    let bundle_grub = src.join("grub/grub");
    if bundle_grub.exists() {
        let target_grub = Path::new("/etc/default/grub");
        if dry_run {
            println!("  [dry-run] Would merge kernel params from bundle GRUB config");
        } else {
            let existing = std::fs::read_to_string(target_grub).unwrap_or_default();
            let bundle_content = std::fs::read_to_string(&bundle_grub)?;
            let bundle_cmdline = extract_grub_cmdline(&bundle_content);
            if !bundle_cmdline.is_empty() {
                let new_content = merge_grub_cmdline(&existing, &bundle_cmdline);
                std::fs::write(target_grub, &new_content)?;
                println!("  ✓ Kernel boot params merged: {}", bundle_cmdline);
            }
        }
    }

    Ok(())
}

fn extract_grub_cmdline(content: &str) -> String {
    content.lines()
        .find(|l| l.starts_with("GRUB_CMDLINE_LINUX_DEFAULT="))
        .and_then(|l| l.split('=').nth(1))
        .map(|v| v.trim_matches('"').to_string())
        .unwrap_or_default()
}

fn merge_grub_cmdline(existing: &str, bundle_params: &str) -> String {
    if bundle_params.is_empty() { return existing.to_string(); }
    let mut result = String::new();
    let mut merged = false;
    for line in existing.lines() {
        if line.starts_with("GRUB_CMDLINE_LINUX_DEFAULT=") {
            result.push_str(&format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"\n", bundle_params));
            merged = true;
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    if !merged {
        result.push_str(&format!("GRUB_CMDLINE_LINUX_DEFAULT=\"{}\"\n", bundle_params));
    }
    result
}
