use crate::distro::DistroInfo;
use anyhow::Result;
use std::path::Path;

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("firmware");
    std::fs::create_dir_all(&dest)?;

    let pm = crate::packages::pm_detect::detect()?;
    let info = match pm.as_str() {
        "pacman" => crate::util::capture_cmd_output("pacman -Qi linux-firmware 2>/dev/null"),
        "apt" => crate::util::capture_cmd_output("dpkg -s firmware-linux 2>/dev/null; dpkg -s firmware-linux-nonfree 2>/dev/null"),
        "dnf" => crate::util::capture_cmd_output("rpm -qi linux-firmware 2>/dev/null"),
        _ => String::new(),
    };
    std::fs::write(dest.join("manifest"), info)?;

    // Count firmware blobs
    let fw_dir = Path::new("/lib/firmware");
    if fw_dir.exists() {
        let count = walkdir::WalkDir::new(fw_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();
        std::fs::write(dest.join("blob-count"), count.to_string())?;
    }

    Ok(())
}

pub fn check(bundle_path: &Path, _target: &DistroInfo, dry_run: bool) -> Result<()> {
    let bundle_fw = bundle_path.join("firmware/manifest");
    if !bundle_fw.exists() {
        println!("  No firmware manifest in bundle, skipping");
        return Ok(());
    }

    if dry_run {
        println!("  [dry-run] Would check firmware versions");
        return Ok(());
    }

    let bundle_ver = std::fs::read_to_string(&bundle_fw)?;
    let first_line = bundle_ver.lines().next().unwrap_or("?");
    println!("  Bundle firmware package info: {}", first_line);
    println!("  Consider verifying linux-firmware is up-to-date on this system");

    // Check current firmware version
    let current_fw = Path::new("/lib/firmware");
    if current_fw.exists() {
        let count = walkdir::WalkDir::new(current_fw)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();
        let bundle_count = std::fs::read_to_string(bundle_path.join("firmware/blob-count"))
            .unwrap_or_default();
        println!("  Firmware blobs: {} on current system", count);
        if !bundle_count.is_empty() {
            println!("               : {} in bundle", bundle_count);
        }
    }

    Ok(())
}
