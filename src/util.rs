use anyhow::Result;
use std::path::Path;

/// Ensure a valid sudo session exists (prompts user for password)
pub fn sudo_ensure() -> Result<()> {
    let status = std::process::Command::new("sudo")
        .args(["-v"])
        .status()
        .map_err(|e| anyhow::anyhow!("sudo not available: {}", e))?;
    if !status.success() {
        anyhow::bail!("sudo authentication failed");
    }
    Ok(())
}

/// Run a command via sudo and return stdout
pub fn sudo_cmd(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sudo")
        .args(["sh", "-c", cmd])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to execute sudo command: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("sudo command failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Run a command as the current user, capture stdout (no sudo)
pub fn capture_cmd_output(cmd: &str) -> String {
    std::process::Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Run a command as the current user, capture stdout (no sudo) — returns Result
pub fn run_cmd(cmd: &str) -> Result<String> {
    let output = std::process::Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map_err(|e| anyhow::anyhow!("failed to execute command: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("command failed: {} — {}", cmd, stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Recursively copy a directory's contents
pub fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src) {
        let e = entry?;
        let relative = e.path().strip_prefix(src)?;
        let target = dst.join(relative);
        if e.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            std::fs::copy(e.path(), &target)?;
        }
    }
    Ok(())
}

/// Rebuild initramfs based on detected distro
pub fn rebuild_initramfs(dry_run: bool) -> Result<()> {
    let pm = crate::packages::pm_detect::detect()?;
    let cmd = match pm.as_str() {
        "pacman" => "mkinitcpio -P",
        "apt" => "update-initramfs -u -k all",
        "dnf" => "dracut --force --regenerate-all",
        "zypper" => "mkinitrd",
        _ => "update-initramfs -u -k all",
    };
    if dry_run {
        println!("  [dry-run] Would rebuild initramfs: {}", cmd);
    } else {
        println!("  Rebuilding initramfs...");
        sudo_cmd(cmd)?;
        println!("  Initramfs rebuilt");
    }
    Ok(())
}

/// Update bootloader config
pub fn update_bootloader(dry_run: bool) -> Result<()> {
    let pm = crate::packages::pm_detect::detect()?;
    let cmd = match pm.as_str() {
        "pacman" => "grub-mkconfig -o /boot/grub/grub.cfg",
        "apt" => "update-grub",
        "dnf" => "grub2-mkconfig -o /boot/grub2/grub.cfg",
        "zypper" => "grub2-mkconfig -o /boot/grub2/grub.cfg",
        _ => "update-grub",
    };
    if dry_run {
        println!("  [dry-run] Would update bootloader: {}", cmd);
    } else {
        println!("  Updating bootloader...");
        sudo_cmd(cmd)?;
        println!("  Bootloader updated");
    }
    Ok(())
}
