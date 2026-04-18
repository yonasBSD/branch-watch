use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use octocrab::Octocrab;
use serde_json::{json, Value};

use crate::github::{compare_branches, upstream_info};

pub async fn run(client: &Octocrab, behind_only: bool, output_json: bool) -> Result<()> {
    let repos: Value = client
        .get("/user/repos?type=fork&per_page=100", None::<&()>)
        .await?;

    let empty = vec![];
    let fork_list: Vec<(String, String)> = repos
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|r| {
            let owner = r["owner"]["login"].as_str()?.to_string();
            let name = r["name"].as_str()?.to_string();
            Some((owner, name))
        })
        .collect();

    if fork_list.is_empty() {
        println!("No forked repositories found.");
        return Ok(());
    }

    let pb = ProgressBar::new(fork_list.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.dim} [{pos}/{len}] {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );

    let mut rows: Vec<(String, String, u64, u64)> = vec![];
    for (owner, name) in &fork_list {
        pb.set_message(format!("{owner}/{name}"));
        if let Some((up_owner, up_repo, up_branch)) = upstream_info(client, owner, name).await? {
            let cmp = compare_branches(
                client,
                owner,
                name,
                &format!("{up_owner}:{up_branch}"),
                "HEAD",
            )
            .await
            .unwrap_or(crate::github::CompareResult { behind: 0, ahead: 0 });
            rows.push((
                format!("{owner}/{name}"),
                format!("{up_owner}/{up_repo}"),
                cmp.behind,
                cmp.ahead,
            ));
        }
        pb.inc(1);
    }
    pb.finish_and_clear();

    // sort by behind descending, then ahead descending
    rows.sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));

    if behind_only {
        rows.retain(|(_, _, behind, _)| *behind > 0);
    }

    if rows.is_empty() {
        println!("All forks are in sync with upstream.");
        return Ok(());
    }

    if output_json {
        let out: Vec<Value> = rows
            .iter()
            .map(|(repo, upstream, behind, ahead)| {
                json!({ "repo": repo, "upstream": upstream, "behind": behind, "ahead": ahead })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!("{}\n", "Forked repositories".bold());

    let name_width = rows.iter().map(|(n, _, _, _)| n.len()).max().unwrap_or(30);
    let up_width = rows.iter().map(|(_, u, _, _)| u.len()).max().unwrap_or(30);

    for (repo, upstream, behind, ahead) in &rows {
        let status = format_status(*behind, *ahead);
        println!(
            "  {:<nw$}  {:<uw$}  {status}",
            repo.bold(),
            upstream.dimmed(),
            nw = name_width,
            uw = up_width,
        );
    }
    println!();

    Ok(())
}

fn format_status(behind: u64, ahead: u64) -> String {
    match (behind, ahead) {
        (0, 0) => "✓ in sync".green().to_string(),
        (b, 0) => format!("↓ {b} behind").yellow().to_string(),
        (0, a) => format!("↑ {a} ahead").cyan().to_string(),
        (b, a) => format!("↓ {b} behind  ↑ {a} ahead").yellow().to_string(),
    }
}
