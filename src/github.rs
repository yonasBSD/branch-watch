use anyhow::{bail, Result};
use octocrab::Octocrab;
use serde_json::Value;

pub fn build_client(token: &str) -> Result<Octocrab> {
    Ok(Octocrab::builder().personal_token(token.to_string()).build()?)
}

pub struct CompareResult {
    pub behind: u64,
    pub ahead: u64,
}

pub async fn compare_branches(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    base: &str,
    head: &str,
) -> Result<CompareResult> {
    let base_head = format!("{base}...{head}");
    let resp: Value = client
        .get(
            format!("/repos/{owner}/{repo}/compare/{base_head}"),
            None::<&()>,
        )
        .await?;

    let behind = resp["behind_by"].as_u64().unwrap_or(0);
    let ahead = resp["ahead_by"].as_u64().unwrap_or(0);
    Ok(CompareResult { behind, ahead })
}

pub async fn default_branch(client: &Octocrab, owner: &str, repo: &str) -> Result<String> {
    let resp: Value = client
        .get(format!("/repos/{owner}/{repo}"), None::<&()>)
        .await?;
    match resp["default_branch"].as_str() {
        Some(b) => Ok(b.to_string()),
        None => bail!("Could not determine default branch for {owner}/{repo}"),
    }
}

pub async fn upstream_info(
    client: &Octocrab,
    owner: &str,
    repo: &str,
) -> Result<Option<(String, String, String)>> {
    let resp: Value = client
        .get(format!("/repos/{owner}/{repo}"), None::<&()>)
        .await?;
    if resp["fork"].as_bool().unwrap_or(false) {
        let parent = &resp["parent"];
        let up_owner = parent["owner"]["login"].as_str().unwrap_or("").to_string();
        let up_repo = parent["name"].as_str().unwrap_or("").to_string();
        let up_branch = parent["default_branch"]
            .as_str()
            .unwrap_or("main")
            .to_string();
        Ok(Some((up_owner, up_repo, up_branch)))
    } else {
        Ok(None)
    }
}
