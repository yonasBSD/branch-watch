use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use octocrab::Octocrab;
use serde_json::{json, Value};

use crate::github::{compare_branches, default_branch};

pub async fn run(client: &Octocrab, repo: &str, behind_only: bool, output_json: bool) -> Result<()> {
    let (owner, name) = parse_repo(repo)?;
    let base = default_branch(client, owner, name).await?;

    let branches: Value = client
        .get(
            format!("/repos/{owner}/{name}/branches?per_page=100"),
            None::<&()>,
        )
        .await?;

    let empty = vec![];
    let branch_names: Vec<String> = branches
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|b| b["name"].as_str().map(str::to_string))
        .filter(|b| b != &base)
        .collect();

    if branch_names.is_empty() {
        println!("No branches other than '{base}' found in {owner}/{name}.");
        return Ok(());
    }

    let pb = ProgressBar::new(branch_names.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim} [{pos}/{len}] {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    let mut rows: Vec<(String, u64, u64)> = vec![];
    for branch in &branch_names {
        pb.set_message(branch.to_string());
        let cmp = compare_branches(client, owner, name, &base, branch).await?;
        rows.push((branch.clone(), cmp.behind, cmp.ahead));
        pb.inc(1);
    }
    pb.finish_and_clear();

    // sort by behind descending, then ahead descending
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));

    if behind_only {
        rows.retain(|(_, behind, _)| *behind > 0);
    }

    if rows.is_empty() {
        println!("All branches are up to date with '{base}'.");
        return Ok(());
    }

    if output_json {
        let out: Vec<Value> = rows
            .iter()
            .map(|(branch, behind, ahead)| {
                json!({ "branch": branch, "base": base, "behind": behind, "ahead": ahead })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!(
        "{} {} (base: {})\n",
        "→".dimmed(),
        format!("{owner}/{name}").bold(),
        base.cyan()
    );

    let name_width = rows.iter().map(|(n, _, _)| n.len()).max().unwrap_or(20);

    for (branch, behind, ahead) in &rows {
        let status = format_status(*behind, *ahead);
        println!("  {:<width$}  {status}", branch.bold(), width = name_width);
    }
    println!();

    Ok(())
}

fn parse_repo(repo: &str) -> Result<(&str, &str)> {
    let mut parts = repo.splitn(2, '/');
    let owner = parts.next().unwrap_or("");
    let name = parts.next().unwrap_or("");
    if owner.is_empty() || name.is_empty() {
        anyhow::bail!("Repo must be in 'owner/name' format");
    }
    Ok((owner, name))
}

fn format_status(behind: u64, ahead: u64) -> String {
    match (behind, ahead) {
        (0, 0) => "✓ up to date".green().to_string(),
        (b, 0) => format!("↓ {b} behind").yellow().to_string(),
        (0, a) => format!("↑ {a} ahead").cyan().to_string(),
        (b, a) => format!("↓ {b} behind  ↑ {a} ahead").yellow().to_string(),
    }
}
