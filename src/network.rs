use anyhow::Result;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("network");
    std::fs::create_dir_all(&dest)?;

    let nm_conns = Path::new("/etc/NetworkManager/system-connections");
    if nm_conns.exists() {
        let count = std::fs::read_dir(nm_conns)
            .map(|e| e.flatten().count())
            .unwrap_or(0);
        if count > 0 {
            crate::util::copy_dir(nm_conns, &dest.join("NetworkManager"))?;
            println!("  ✓ {} NetworkManager connection profiles captured", count);
        } else {
            println!("  NetworkManager connections directory empty, skipping");
        }
    }

    Ok(())
}

pub fn apply(bundle_path: &Path, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("network");
    if !src.exists() {
        println!("  No network config in bundle, skipping");
        return Ok(());
    }

    let nm_src = src.join("NetworkManager");
    if nm_src.exists() {
        let nm_target = Path::new("/etc/NetworkManager/system-connections");
        if dry_run {
            println!("  [dry-run] Would restore {} NetworkManager connection profiles", nm_target.display());
        } else {
            std::fs::create_dir_all(nm_target)?;
            crate::util::copy_dir(&nm_src, nm_target)?;
            for entry in walkdir::WalkDir::new(nm_target) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let _ = std::fs::set_permissions(entry.path(), std::fs::Permissions::from_mode(0o600));
                }
            }
            let _ = crate::util::sudo_cmd("systemctl restart NetworkManager 2>/dev/null || systemctl restart NetworkManager.service 2>/dev/null");
            println!("  ✓ NetworkManager connections restored (service restarted)");
        }
    }

    Ok(())
}
