use anyhow::Result;
use std::path::Path;

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("systemd");
    std::fs::create_dir_all(&dest)?;

    // System services
    let system = crate::util::capture_cmd_output(
        "systemctl list-unit-files --state=enabled --no-legend 2>/dev/null | awk '{print $1}'"
    );
    std::fs::write(dest.join("enabled-system"), system)?;

    // User services
    let user = crate::util::capture_cmd_output(
        "systemctl --user list-unit-files --state=enabled --no-legend 2>/dev/null | awk '{print $1}'"
    );
    std::fs::write(dest.join("enabled-user"), user)?;

    Ok(())
}

pub fn apply(bundle_path: &Path, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("systemd");
    if !src.exists() {
        println!("  No systemd services in bundle, skipping");
        return Ok(());
    }

    // System services
    let system_content = std::fs::read_to_string(src.join("enabled-system")).unwrap_or_default();
    let system_services: Vec<&str> = system_content.lines().filter(|l| !l.is_empty()).collect();

    if !system_services.is_empty() {
        if dry_run {
            for svc in &system_services {
                println!("  [dry-run] Would enable system service: {}", svc);
            }
        } else {
            for svc in &system_services {
                match crate::util::sudo_cmd(&format!("systemctl enable --now {} 2>/dev/null", svc)) {
                    Ok(_) => println!("  ✓ Enabled system service: {}", svc),
                    Err(_) => println!("  ⚠  Could not enable {} (may not exist on this distro)", svc),
                }
            }
        }
    }

    // User services
    let user_content = std::fs::read_to_string(src.join("enabled-user")).unwrap_or_default();
    let user_services: Vec<&str> = user_content.lines().filter(|l| !l.is_empty()).collect();

    if !user_services.is_empty() {
        if dry_run {
            for svc in &user_services {
                println!("  [dry-run] Would enable user service: {}", svc);
            }
        } else {
            for svc in &user_services {
                match std::process::Command::new("systemctl")
                    .args(["--user", "enable", "--now", svc])
                    .status()
                {
                    Ok(_) => println!("  ✓ Enabled user service: {}", svc),
                    Err(_) => println!("  ⚠  Could not enable user service: {}", svc),
                }
            }
        }
    }

    Ok(())
}
