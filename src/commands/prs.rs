use anyhow::Result;
use colored::Colorize;
use octocrab::Octocrab;
use serde_json::Value;

pub async fn run(client: &Octocrab, repo: &str) -> Result<()> {
    let (owner, name) = parse_repo(repo)?;

    let prs: Value = client
        .get(
            format!("/repos/{owner}/{name}/pulls?state=open&per_page=100"),
            None::<&()>,
        )
        .await?;

    let empty = vec![];
    let list = prs.as_array().unwrap_or(&empty);

    if list.is_empty() {
        println!("No open pull requests in {owner}/{name}.");
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

        println!("  #{number:<4} {}{draft_label}", title.bold());
        println!(
            "         {} → {}  by @{}{review_label}",
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
