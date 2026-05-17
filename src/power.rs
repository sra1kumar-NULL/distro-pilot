use anyhow::Result;
use std::path::Path;

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("power");
    std::fs::create_dir_all(&dest)?;

    // ACPI platform profile
    let profile_path = Path::new("/sys/firmware/acpi/platform_profile");
    if profile_path.exists() {
        let profile = std::fs::read_to_string(profile_path)?;
        std::fs::write(dest.join("platform_profile"), profile.trim())?;
    }

    // power-profiles-daemon
    let ppd = Path::new("/etc/power-profiles-daemon");
    if ppd.exists() {
        crate::util::copy_dir(ppd, &dest.join("power-profiles-daemon"))?;
    }

    // TLP config
    let tlp = Path::new("/etc/tlp.conf");
    if tlp.exists() {
        std::fs::copy(tlp, dest.join("tlp.conf"))?;
    }

    // thermald config
    let thermald = Path::new("/etc/thermald");
    if thermald.exists() {
        crate::util::copy_dir(thermald, &dest.join("thermald"))?;
    }

    // sysfs power snapshot (readable files only, skip if not accessible)
    let sysfs_power = Path::new("/sys/power");
    if sysfs_power.exists() {
        let snap = dest.join("sysfs-snapshot");
        let _ = std::fs::create_dir_all(&snap);
        if let Ok(entries) = std::fs::read_dir(sysfs_power) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                    if let Ok(val) = std::fs::read_to_string(entry.path()) {
                        let _ = std::fs::write(snap.join(entry.file_name()), val.trim());
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn apply(bundle_path: &Path, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("power");
    if !src.exists() {
        println!("  No power state in bundle, skipping");
        return Ok(());
    }

    // Platform profile
    let profile = src.join("platform_profile");
    if profile.exists() {
        let val = std::fs::read_to_string(&profile)?;
        let target = Path::new("/sys/firmware/acpi/platform_profile");
        if target.exists() {
            if dry_run {
                println!("  [dry-run] Would set ACPI platform profile to: {}", val.trim());
            } else {
                let val_trimmed = val.trim();
                crate::util::sudo_cmd(&format!("echo '{}' > /sys/firmware/acpi/platform_profile", val_trimmed))?;
                println!("  ✓ ACPI platform profile set to: {}", val_trimmed);
            }
        } else {
            println!("  ⚠  /sys/firmware/acpi/platform_profile not available on this kernel");
        }
    }

    // TLP config
    let tlp_conf = src.join("tlp.conf");
    if tlp_conf.exists() {
        if dry_run {
            println!("  [dry-run] Would restore TLP config");
        } else {
            let target_dir = Path::new("/etc");
            std::fs::copy(&tlp_conf, target_dir.join("tlp.conf"))?;
            println!("  ✓ TLP config restored");
            // Enable TLP service
            let _ = crate::util::sudo_cmd("systemctl enable --now tlp 2>/dev/null");
        }
    }

    // thermald config
    let thermald_src = src.join("thermald");
    if thermald_src.exists() {
        if dry_run {
            println!("  [dry-run] Would restore thermald config");
        } else {
            let thermald_dst = Path::new("/etc/thermald");
            crate::util::copy_dir(&thermald_src, thermald_dst)?;
            let _ = crate::util::sudo_cmd("systemctl enable --now thermald 2>/dev/null");
            println!("  ✓ thermald config restored");
        }
    }

    Ok(())
}
