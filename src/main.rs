mod commands;
mod config;
mod github;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bw", about = "Track branch and fork sync status on GitHub")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Save your GitHub personal access token
    Auth {
        token: String,
    },
    /// Show sync status of all your forked repositories
    Forks,
    /// Show branch sync status vs default branch
    Branches {
        /// Repository in owner/name format
        repo: String,
    },
    /// List open pull requests
    Prs {
        /// Repository in owner/name format
        repo: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Auth { token } => {
            let mut cfg = config::load()?;
            cfg.token = Some(token);
            config::save(&cfg)?;
        }
        Command::Forks => {
            let client = build_client()?;
            commands::forks::run(&client).await?;
        }
        Command::Branches { repo } => {
            let client = build_client()?;
            commands::branches::run(&client, &repo).await?;
        }
        Command::Prs { repo } => {
            let client = build_client()?;
            commands::prs::run(&client, &repo).await?;
        }
    }

    Ok(())
}

fn build_client() -> Result<octocrab::Octocrab> {
    let cfg = config::load()?;
    let token = cfg.token.or_else(|| std::env::var("GITHUB_TOKEN").ok());
    match token {
        Some(t) => Ok(github::build_client(&t)?),
        None => bail!("No GitHub token found. Run `bw auth <token>` or set GITHUB_TOKEN."),
    }
}
