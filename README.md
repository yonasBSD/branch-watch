# branch-watch

[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-branch--watch-blue?logo=github)](https://github.com/marketplace/actions/branch-watch)
[![Release](https://img.shields.io/github/v/release/nuri-yoo/branch-watch)](https://github.com/nuri-yoo/branch-watch/releases)
[![PyPI](https://img.shields.io/pypi/v/branch-watch)](https://pypi.org/project/branch-watch/)
[![npm](https://img.shields.io/npm/v/branch-watch)](https://www.npmjs.com/package/branch-watch)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

**branch-watch** is a command-line tool that shows how your GitHub branches and forks compare to their base or upstream — behind, ahead, or in sync — in a single command.

> Check if your GitHub branches are behind main. Track all your forked repositories against upstream. List open pull requests across any repo. All from the terminal.

## What is branch-watch?

`branch-watch` (`bw`) uses the [GitHub REST API](https://docs.github.com/en/rest) to answer three questions developers ask every day:

- **Are my feature branches behind `main`?** → `bw branches owner/repo`
- **Are my forks behind upstream?** → `bw forks`
- **What pull requests are open?** → `bw prs owner/repo`

Works with personal accounts, organization repos, and any public or private GitHub repository you have access to.

```
$ bw forks

Forked repositories

  alice/rust          rust-lang/rust        ↓ 42 behind
  alice/tokio         tokio-rs/tokio        ↓ 7 behind   ↑ 2 ahead
  alice/serde         serde-rs/serde        ✓ in sync
  acme/axum           tokio-rs/axum         ↓ 15 behind  ↑ 8 ahead
  acme/reqwest        seanmonstar/reqwest   ↑ 3 ahead
```

```
$ bw branches owner/repo

→ owner/repo (base: main)

  feature/auth        ↓ 3 behind  ↑ 2 ahead
  feat/dashboard      ↓ 14 behind
  fix/login           ✓ up to date
```

## Features

- **Fork sync status** — lists all your forked repositories and shows how many commits each fork is behind or ahead of its upstream default branch
- **Branch status** — for any repository, shows how every branch compares to the default branch (main/master/etc.)
- **Open PR list** — shows open pull requests with author, head → base branch, draft status, and pending reviewers
- **Works with any GitHub account** — personal accounts, organization forks, company repositories
- **Single binary, no runtime** — written in Rust; install via Homebrew, pip, npm, or download directly

## Installation

### Homebrew (macOS / Linux)

```sh
brew install nuri-yoo/tap/branch-watch
```

### pip

```sh
pip install branch-watch
```

### npm

```sh
npm install -g branch-watch
```

### Pre-built binaries

Download the latest binary for your platform from the [releases page](https://github.com/nuri-yoo/branch-watch/releases):

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `branch-watch-*-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `branch-watch-*-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `branch-watch-*-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `branch-watch-*-aarch64-unknown-linux-gnu.tar.gz` |

```sh
tar xzf branch-watch-*.tar.gz
sudo mv bw /usr/local/bin/
```

### From source

Requires [Rust](https://rustup.rs) 1.80+.

```sh
git clone https://github.com/nuri-yoo/branch-watch
cd branch-watch
cargo install --path .
```

The binary is installed as `bw`.

## Authentication

Generate a GitHub Personal Access Token with `repo` scope at [github.com/settings/tokens](https://github.com/settings/tokens), then:

```sh
bw auth <your-token>
```

The token is saved to `~/.branch-watch.toml`. Alternatively, set the `GITHUB_TOKEN` environment variable:

```sh
export GITHUB_TOKEN=<your-token>
bw forks
```

## Usage

### How to check if branches are behind main

```sh
bw branches owner/repo
```

Shows every branch in the repository and how many commits it is ahead or behind the default branch. Useful for spotting stale branches before they become hard to merge.

### How to check if forks are behind upstream

```sh
bw forks
```

Lists all your forked repositories with their sync status against the upstream default branch.

### How to list open pull requests

```sh
bw prs owner/repo
```

Shows open PRs with author, head → base branch, draft status, and pending reviewers.

### How to save your GitHub token

```sh
bw auth ghp_xxxxxxxxxxxxxxxxxxxx
```

## Use in GitHub Actions

Add branch-watch to your CI workflow to automatically surface stale branches:

```yaml
- uses: nuri-yoo/branch-watch@v1
  with:
    command: branches
    repo: ${{ github.repository }}
```

Available on the [GitHub Marketplace](https://github.com/marketplace/actions/branch-watch).

## FAQ

**Does branch-watch work with private repositories?**
Yes. As long as your GitHub token has `repo` scope, branch-watch works with any private repository you have access to.

**Does it support GitHub Enterprise?**
Not yet. Currently supports github.com only.

**What permissions does the token need?**
`repo` scope is sufficient for both public and private repositories.

**How is this different from `git fetch`?**
`git fetch` requires a local clone. branch-watch works entirely via the GitHub API — no local clone needed.

## GitHub API Usage

`branch-watch` integrates with the following GitHub REST API endpoints:

| Endpoint | Purpose |
|----------|---------|
| `GET /user/repos?type=fork` | List authenticated user's forked repositories |
| `GET /repos/{owner}/{repo}` | Fetch repository metadata including upstream parent info |
| `GET /repos/{owner}/{repo}/branches` | List all branches in a repository |
| `GET /repos/{owner}/{repo}/compare/{base}...{head}` | Compare two refs to get ahead/behind commit counts |
| `GET /repos/{owner}/{repo}/pulls?state=open` | List open pull requests |

## Use cases

**Open-source contributor** — maintain forks of multiple upstream projects and instantly see which ones have drifted, without opening a browser.

**Team lead** — run `bw branches owner/company-repo` before a sprint to catch feature branches that have fallen behind `main`.

**Release manager** — use `bw prs owner/repo` to get a clean list of open PRs without GitHub's UI noise.

## License

MIT
