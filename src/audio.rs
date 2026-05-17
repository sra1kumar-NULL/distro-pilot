use anyhow::Result;
use std::path::Path;

pub fn capture(output: &Path) -> Result<()> {
    let dest = output.join("audio");
    std::fs::create_dir_all(&dest)?;

    if let Some(config) = dirs::config_dir() {
        let pw = config.join("pipewire");
        if pw.exists() {
            crate::util::copy_dir(&pw, &dest.join("pipewire"))?;
        }

        let wp = config.join("wireplumber");
        if wp.exists() {
            crate::util::copy_dir(&wp, &dest.join("wireplumber"))?;
        }
    }

    if let Some(home) = dirs::home_dir() {
        let asoundrc = home.join(".asoundrc");
        if asoundrc.exists() {
            std::fs::copy(&asoundrc, dest.join("asoundrc"))?;
        }
    }

    let sys_pw = Path::new("/etc/pipewire");
    if sys_pw.exists() {
        crate::util::copy_dir(sys_pw, &dest.join("etc-pipewire"))?;
    }

    Ok(())
}

pub fn apply(bundle_path: &Path, dry_run: bool) -> Result<()> {
    let src = bundle_path.join("audio");
    if !src.exists() {
        println!("  No audio config in bundle, skipping");
        return Ok(());
    }

    if let Some(config) = dirs::config_dir() {
        let pw_src = src.join("pipewire");
        if pw_src.exists() {
            let pw_dst = config.join("pipewire");
            if dry_run {
                println!("  [dry-run] Would restore PipeWire user config");
            } else {
                std::fs::create_dir_all(&pw_dst)?;
                crate::util::copy_dir(&pw_src, &pw_dst)?;
                println!("  ✓ PipeWire user config restored");
            }
        }

        let wp_src = src.join("wireplumber");
        if wp_src.exists() {
            let wp_dst = config.join("wireplumber");
            if dry_run {
                println!("  [dry-run] Would restore WirePlumber user config");
            } else {
                std::fs::create_dir_all(&wp_dst)?;
                crate::util::copy_dir(&wp_src, &wp_dst)?;
                println!("  ✓ WirePlumber user config restored");
            }
        }
    }

    let asoundrc_src = src.join("asoundrc");
    if asoundrc_src.exists() {
        if let Some(home) = dirs::home_dir() {
            let asoundrc_dst = home.join(".asoundrc");
            if dry_run {
                println!("  [dry-run] Would restore ALSA config (~/.asoundrc)");
            } else {
                std::fs::copy(&asoundrc_src, &asoundrc_dst)?;
                println!("  ✓ ALSA config restored");
            }
        }
    }

    let etc_pw_src = src.join("etc-pipewire");
    if etc_pw_src.exists() {
        let etc_pw_dst = Path::new("/etc/pipewire");
        if dry_run {
            println!("  [dry-run] Would restore PipeWire system config to /etc/pipewire");
        } else {
            std::fs::create_dir_all(etc_pw_dst)?;
            crate::util::copy_dir(&etc_pw_src, etc_pw_dst)?;
            println!("  ✓ PipeWire system config restored");
        }
    }

    Ok(())
}
