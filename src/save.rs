use crate::bundle::Bundle;
use crate::packages;
use crate::configs;
use crate::dotfiles;
use crate::firmware;
use crate::power;
use crate::hardware;
use crate::systemd;
use crate::distro;
use crate::cli::SaveArgs;
use crate::util;
use anyhow::Result;
use std::path::Path;

pub fn run(args: SaveArgs) -> Result<()> {
    let distro = distro::detect()?;
    let output = Path::new(&args.output);
    let bundle = Bundle::new(&distro);

    if args.dry_run {
        println!("[dry-run] Would save system state to: {}", args.output);
        println!("  Distro: {} {}", distro.id, distro.version_id.unwrap_or_default());
        println!("  Kernel: {}", distro.kernel);
        println!("  Scanning packages...");
        let pkgs = packages::scan_all()?;
        println!("  Found {} packages", pkgs.len());
        println!("  Scanning driver configs (modprobe.d, modules-load.d, udev, grub)...");
        println!("  Config dirs to capture: {}", configs::source_dirs().len());
        println!("  Scanning power state...");
        println!("  Scanning firmware...");
        println!("  Scanning dotfiles...");
        if args.include_ssh {
            println!("  [--include-ssh] Will include SSH keys");
        }
        println!("  Scanning systemd services...");
        println!("  Scanning hardware...");
        if args.bundle {
            println!("  [--bundle] Will pack as .tar.zst");
        }
        println!("\nDry-run complete. Re-run without --dry-run to save.");
        return Ok(());
    }

    std::fs::create_dir_all(output)?;

    println!("Saving system state to: {}", args.output);
    println!("  Distro: {} {}", distro.id, distro.version_id.unwrap_or_default());
    println!("  Kernel: {}", distro.kernel);

    bundle.write_manifest(output)?;
    println!("  ✓ Manifest written");

    let pkgs = packages::scan_all()?;
    packages::write_manifest(output, &pkgs)?;
    println!("  ✓ {} packages captured", pkgs.len());

    configs::capture(output)?;
    println!("  ✓ Driver configs captured");

    firmware::capture(output)?;
    println!("  ✓ Firmware manifest captured");

    power::capture(output)?;
    println!("  ✓ Power state captured");

    dotfiles::capture(output, args.include_ssh)?;
    println!("  ✓ Dotfiles captured");

    systemd::capture(output)?;
    println!("  ✓ Systemd services captured");

    hardware::capture(output)?;
    println!("  ✓ Hardware profile captured");

    if args.bundle {
        let bundle_path = util::pack_bundle(output)?;
        println!("\n✅ Bundle saved to: {}", bundle_path.display());
    } else {
        println!("\n✅ Bundle saved to: {}", args.output);
    }

    Ok(())
}
