use crate::bundle::Bundle;
use crate::packages;
use crate::configs;
use crate::dotfiles;
use crate::firmware;
use crate::power;
use crate::systemd;
use crate::hardware;
use crate::network;
use crate::audio;
use crate::display;
use crate::distro;
use crate::cli::ApplyArgs;
use crate::util;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;

pub fn run(args: ApplyArgs) -> Result<()> {
    let raw_path = Path::new(&args.bundle);

    // If the input is a .tar.zst file, extract to a temp dir
    let temp_cleanup: Option<PathBuf>;
    let bundle_path: PathBuf = if raw_path.is_file() {
        let temp = format!("{}.extracted", args.bundle);
        let p = Path::new(&temp);
        if p.exists() {
            std::fs::remove_dir_all(p)?;
        }
        println!("Extracting bundle from: {}", args.bundle);
        util::unpack_bundle(raw_path, p)?;
        temp_cleanup = Some(p.to_path_buf());
        p.to_path_buf()
    } else {
        temp_cleanup = None;
        raw_path.to_path_buf()
    };

    if !bundle_path.join("manifest.toml").exists() {
        anyhow::bail!("No manifest.toml found at '{}'. Is this a valid bundle?", bundle_path.display());
    }

    let bundle = Bundle::read(&bundle_path)?;
    let target = distro::detect()?;

    println!("Target: {} {} (kernel {})", target.name, target.version_id.as_deref().unwrap_or("?"), target.kernel.split_whitespace().next().unwrap_or("?"));
    println!("Bundle: {} {} from {}", bundle.distro.name, bundle.distro.version_id.as_deref().unwrap_or("?"), bundle.distro.kernel);

    // Keep sudo session alive throughout
    util::sudo_ensure()?;

    let do_all = args.step == "all";

    if do_all || args.step == "network" {
        println!("\n── Network ──");
        network::apply(&bundle_path, args.dry_run)?;
    }

    if do_all || args.step == "packages" {
        println!("\n── Packages ──");
        let pkgs = packages::read_manifest(&bundle_path)?;
        let mapped = packages::mapping::map_all(&pkgs, &bundle.distro.id, &target.id)?;
        packages::install_all(&mapped, args.dry_run)?;
    }

    if do_all || args.step == "drivers" {
        println!("\n── Drivers ──");
        configs::apply(&bundle_path, &target, args.dry_run)?;
        util::rebuild_initramfs(args.dry_run)?;
        util::update_bootloader(args.dry_run)?;
    }

    if do_all || args.step == "firmware" {
        println!("\n── Firmware ──");
        firmware::check(&bundle_path, &target, args.dry_run)?;
    }

    if do_all || args.step == "power" {
        println!("\n── Power ──");
        power::apply(&bundle_path, args.dry_run)?;
    }

    if do_all || args.step == "dotfiles" {
        println!("\n── Dotfiles ──");
        dotfiles::apply(&bundle_path, args.dry_run)?;
    }

    if do_all || args.step == "systemd" {
        println!("\n── Systemd ──");
        systemd::apply(&bundle_path, args.dry_run)?;
    }

    if do_all || args.step == "audio" {
        println!("\n── Audio ──");
        audio::apply(&bundle_path, args.dry_run)?;
    }

    if do_all || args.step == "display" {
        println!("\n── Display ──");
        display::apply(&bundle_path, args.dry_run)?;
    }

    if !args.no_validate && (do_all || args.step != "dotfiles") {
        println!("\n── Validation ──");
        hardware::validate()?;
    }

    // Clean up temp directory if we extracted from .tar.zst
    if let Some(temp) = temp_cleanup {
        let _ = std::fs::remove_dir_all(&temp);
    }

    println!("\n✅ Apply complete. Reboot recommended for all changes to take effect.");
    Ok(())
}
