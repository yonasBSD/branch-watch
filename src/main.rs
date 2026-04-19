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
    Forks {
        /// Show only forks that are behind upstream
        #[arg(long)]
        behind_only: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Show forks for an organization instead of the authenticated user
        #[arg(long)]
        org: Option<String>,
    },
    /// Show branch sync status vs default branch (or --base)
    Branches {
        /// Repository in owner/name format
        repo: String,
        /// Show only branches that are behind the base branch
        #[arg(long)]
        behind_only: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
        /// Base branch to compare against (default: repository default branch)
        #[arg(long)]
        base: Option<String>,
    },
    /// List open pull requests
    Prs {
        /// Repository in owner/name format
        repo: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Manage the repository ignore list
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },
}

#[derive(Subcommand)]
enum IgnoreAction {
    /// Add a repository to the ignore list
    Add {
        /// Repository in owner/name format
        repo: String,
    },
    /// Remove a repository from the ignore list
    Remove {
        /// Repository in owner/name format
        repo: String,
    },
    /// List all ignored repositories
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Auth { token } => {
            let mut cfg = config::load()?;
            cfg.token = Some(token);
            config::save(&cfg)?;
            println!("Token saved.");
        }
        Command::Forks { behind_only, json, org } => {
            let client = build_client()?;
            commands::forks::run(&client, behind_only, json, org.as_deref()).await?;
        }
        Command::Branches { repo, behind_only, json, base } => {
            let client = build_client()?;
            commands::branches::run(&client, &repo, behind_only, json, base.as_deref()).await?;
        }
        Command::Prs { repo, json } => {
            let client = build_client()?;
            commands::prs::run(&client, &repo, json).await?;
        }
        Command::Ignore { action } => {
            handle_ignore(action)?;
        }
    }

    Ok(())
}

fn handle_ignore(action: IgnoreAction) -> Result<()> {
    let mut cfg = config::load()?;
    match action {
        IgnoreAction::Add { repo } => {
            if cfg.ignore.iter().any(|r| r == &repo) {
                println!("'{repo}' is already in the ignore list.");
            } else {
                cfg.ignore.push(repo.clone());
                config::save(&cfg)?;
                println!("Added '{repo}' to ignore list.");
            }
        }
        IgnoreAction::Remove { repo } => {
            let before = cfg.ignore.len();
            cfg.ignore.retain(|r| r != &repo);
            if cfg.ignore.len() == before {
                println!("'{repo}' was not found in the ignore list.");
            } else {
                config::save(&cfg)?;
                println!("Removed '{repo}' from ignore list.");
            }
        }
        IgnoreAction::List => {
            if cfg.ignore.is_empty() {
                println!("Ignore list is empty.");
            } else {
                println!("Ignored repositories:");
                for r in &cfg.ignore {
                    println!("  {r}");
                }
            }
        }
    }
    Ok(())
}

fn build_client() -> Result<octocrab::Octocrab> {
    let cfg = config::load()?;
    let token = cfg.token
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
        .or_else(gh_cli_token);

    match token {
        Some(t) => github::build_client(&t).map_err(|e| {
            if e.to_string().contains("401") {
                anyhow::anyhow!("GitHub token is invalid or expired. Run `bw auth <token>` to update.")
            } else if e.to_string().contains("403") {
                anyhow::anyhow!("GitHub API rate limit exceeded or insufficient token scope. Ensure your token has `repo` scope.")
            } else {
                e
            }
        }),
        None => bail!(
            "No GitHub token found.\n\
             Options:\n\
             1. Run `bw auth <token>` to save a token\n\
             2. Set the GITHUB_TOKEN environment variable\n\
             3. Install the GitHub CLI (gh) and run `gh auth login`\n\
             Generate a token at: https://github.com/settings/tokens"
        ),
    }
}

fn gh_cli_token() -> Option<String> {
    std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
