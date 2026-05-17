use anyhow::Result;
use std::path::Path;

const DISPLAY_FILES: &[&str] = &[
    "monitors.xml",
    "monitors.xml.previous",
    "kwinoutputconfig.json",
];

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("display");
    std::fs::create_dir_all(&dest)?;

    if let Some(config) = dirs::config_dir() {
        for file in DISPLAY_FILES {
            let path = config.join(file);
            if path.exists() {
                std::fs::copy(&path, dest.join(file))?;
            }
        }

        let kanshi = config.join("kanshi");
        if kanshi.exists() {
            crate::util::copy_dir(&kanshi, &dest.join("kanshi"))?;
        }
    }

    let xrandr = crate::util::capture_cmd_output(
        "xrandr --current 2>/dev/null"
    );
    if !xrandr.is_empty() {
        std::fs::write(dest.join("xrandr-current.txt"), &xrandr)?;
    }

    Ok(())
}

pub fn apply(bundle_path: &Path, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("display");
    if !src.exists() {
        println!("  No display config in bundle, skipping");
        return Ok(());
    }

    if let Some(config) = dirs::config_dir() {
        for file in DISPLAY_FILES {
            let bundle_file = src.join(file);
            if bundle_file.exists() {
                let target = config.join(file);
                if dry_run {
                    println!("  [dry-run] Would restore: {}", file);
                } else {
                    let _ = std::fs::copy(&bundle_file, &target);
                    println!("  ✓ Restored: {}", file);
                }
            }
        }

        let kanshi_src = src.join("kanshi");
        if kanshi_src.exists() {
            let kanshi_dst = config.join("kanshi");
            if dry_run {
                println!("  [dry-run] Would restore kanshi config");
            } else {
                std::fs::create_dir_all(&kanshi_dst)?;
                crate::util::copy_dir(&kanshi_src, &kanshi_dst)?;
                println!("  ✓ kanshi config restored");
            }
        }
    }

    Ok(())
}
