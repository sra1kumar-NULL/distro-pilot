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
    /// Output path (directory or .tar.zst)
    pub output: String,
    /// Scan only, don't write anything
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(clap::Args)]
pub struct ApplyArgs {
    /// Path to the bundle
    pub bundle: String,
    /// Which step(s) to run: packages, drivers, dotfiles, systemd, all
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
