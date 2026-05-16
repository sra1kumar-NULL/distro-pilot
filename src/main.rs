mod cli;
mod save;
mod apply;
mod bundle;
mod packages;
mod configs;
mod dotfiles;
mod firmware;
mod power;
mod systemd;
mod distro;
mod hardware;
mod util;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Command::Save(args) => save::run(args),
        cli::Command::Apply(args) => apply::run(args),
        cli::Command::Map(args) => {
            let result = packages::mapping::lookup(&args.package, &args.from, &args.to)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        cli::Command::Inspect(args) => {
            let bundle = bundle::Bundle::read(std::path::Path::new(&args.bundle))?;
            println!("DistroPilot v{}", env!("CARGO_PKG_VERSION"));
            println!("Bundle: {}", args.bundle);
            println!("  Source distro: {} {}", bundle.distro.id, bundle.distro.version_id.unwrap_or_default());
            println!("  Kernel: {}", bundle.distro.kernel);
            println!("  Created: {}", bundle.created_at);
            Ok(())
        }
    }
}
