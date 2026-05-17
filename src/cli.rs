use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "distropilot", about = "Save and restore Linux system state across distros", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Save current system state to a portable bundle
    Save(SaveArgs),
    /// Apply a saved bundle to this system
    Apply(ApplyArgs),
    /// Look up a package name across distros
    Map(MapArgs),
    /// Inspect a bundle without applying it
    Inspect(InspectArgs),
}

#[derive(clap::Args)]
pub struct SaveArgs {
    /// Output path (directory)
    pub output: String,
    /// Scan only, don't write anything
    #[arg(long)]
    pub dry_run: bool,
    /// Include SSH keys (~/.ssh/) in bundle (SECURITY: copies private keys to the bundle)
    #[arg(long)]
    pub include_ssh: bool,
    /// Exclude paths containing this substring (repeatable, e.g. --exclude Steam --exclude .cache)
    #[arg(long)]
    pub exclude: Vec<String>,
    /// Pack bundle as a single .tar.zst file instead of a directory
    #[arg(long)]
    pub bundle: bool,
}

#[derive(clap::Args)]
pub struct ApplyArgs {
    /// Path to the bundle
    pub bundle: String,
    /// Which step(s) to run: packages, drivers, firmware, power, audio, display, dotfiles, systemd, network, all
    #[arg(long, default_value = "all")]
    pub step: String,
    /// Show what would change without applying
    #[arg(long)]
    pub dry_run: bool,
    /// Skip validation step
    #[arg(long)]
    pub no_validate: bool,
}

#[derive(clap::Args)]
pub struct MapArgs {
    /// Canonical app name (e.g. neovim, discord)
    pub package: String,
    /// Source distro
    #[arg(long, default_value = "arch")]
    pub from: String,
    /// Target distro
    #[arg(long, default_value = "debian")]
    pub to: String,
}

#[derive(clap::Args)]
pub struct InspectArgs {
    /// Path to the bundle
    pub bundle: String,
}
