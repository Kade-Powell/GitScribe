use clap::{Parser, Subcommand};

use gitscribe::{
    config::load_config, handle_init, handle_version_bump, util::print_banner,
    version::VersionDesignation,
};

#[derive(Parser)]
#[command(author, version, about)] // Read from `Cargo.toml`
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a new GitScribe configuration file
    Init,
    /// Bumps the version by a patch e.g. 1.0.0 -> 1.0.1
    Patch,
    /// Bumps the version by a minor e.g. 1.0.4 -> 1.1.0
    Minor,
    /// Bumps the version by a major e.g. 1.0.4 -> 2.0.0
    Major,
}

fn main() {
    print_banner();
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            handle_init();
        }
        Commands::Patch {} => {
            //load config file from gitscribe.json as str, if not there, create it
            let config = load_config();
            match config {
                Some(_) => {}
                None => std::process::exit(1),
            }
            handle_version_bump(config.unwrap(), VersionDesignation::Patch);
        }
        Commands::Minor {} => {
            //load config file from gitscribe.json as str, if not there, create it
            let config = load_config();
            match config {
                Some(_) => {}
                None => std::process::exit(1),
            }
            handle_version_bump(config.unwrap(), VersionDesignation::Minor);
        }
        Commands::Major {} => {
            //load config file from gitscribe.json as str, if not there, create it
            let config = load_config();
            match config {
                Some(_) => {}
                None => std::process::exit(1),
            }
            handle_version_bump(config.unwrap(), VersionDesignation::Major);
        }
    }
}
