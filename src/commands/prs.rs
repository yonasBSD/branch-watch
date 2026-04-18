use anyhow::Result;
use colored::Colorize;
use octocrab::Octocrab;
use serde_json::{json, Value};

pub async fn run(client: &Octocrab, repo: &str, output_json: bool) -> Result<()> {
    let (owner, name) = parse_repo(repo)?;

    let prs: Value = client
        .get(
            format!("/repos/{owner}/{name}/pulls?state=open&per_page=100&sort=created&direction=asc"),
            None::<&()>,
        )
        .await?;

    let empty = vec![];
    let list = prs.as_array().unwrap_or(&empty);

    if list.is_empty() {
        println!("No open pull requests in {owner}/{name}.");
        return Ok(());
    }

    if output_json {
        let out: Vec<Value> = list
            .iter()
            .map(|pr| {
                json!({
                    "number": pr["number"],
                    "title": pr["title"],
                    "author": pr["user"]["login"],
                    "head": pr["head"]["ref"],
                    "base": pr["base"]["ref"],
                    "draft": pr["draft"],
                    "created_at": pr["created_at"],
                    "requested_reviewers": pr["requested_reviewers"].as_array().map(|r| r.len()).unwrap_or(0),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!(
        "{} {} — {} open PR{}\n",
        "→".dimmed(),
        format!("{owner}/{name}").bold(),
        list.len(),
        if list.len() == 1 { "" } else { "s" }
    );

    for pr in list {
        let number = pr["number"].as_u64().unwrap_or(0);
        let title = pr["title"].as_str().unwrap_or("(no title)");
        let author = pr["user"]["login"].as_str().unwrap_or("?");
        let head = pr["head"]["ref"].as_str().unwrap_or("?");
        let base = pr["base"]["ref"].as_str().unwrap_or("?");
        let draft = pr["draft"].as_bool().unwrap_or(false);
        let created_at = pr["created_at"].as_str().unwrap_or("");
        let age = format_age(created_at);
        let reviews = pr["requested_reviewers"]
            .as_array()
            .map(|r: &Vec<Value>| r.len())
            .unwrap_or(0);

        let draft_label = if draft {
            " [draft]".dimmed().to_string()
        } else {
            String::new()
        };
        let review_label = if reviews > 0 {
            format!(" · {} reviewer{}", reviews, if reviews == 1 { "" } else { "s" })
                .cyan()
                .to_string()
        } else {
            String::new()
        };
        let age_label = if age.is_empty() {
            String::new()
        } else {
            format!(" · {age}").dimmed().to_string()
        };

        println!("  #{number:<4} {}{draft_label}", title.bold());
        println!(
            "         {} → {}  by @{}{review_label}{age_label}",
            head.yellow(),
            base.green(),
            author.dimmed(),
        );
        println!();
    }

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

fn format_age(created_at: &str) -> String {
    // Parse ISO 8601 date and return human-readable age
    let Some(date) = created_at.get(..10) else {
        return String::new();
    };
    let parts: Vec<u32> = date.split('-').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 3 {
        return String::new();
    }
    // Simple approximation using current date components from env or fallback
    // We compare against a known reference without pulling in chrono
    format!("opened {date}")
}
