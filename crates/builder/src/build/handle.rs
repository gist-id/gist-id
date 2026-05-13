//! Resolve the gist.id handle (a GitHub username).
//!
//! In GitHub Actions: read `GITHUB_REPOSITORY_OWNER` (set automatically).
//! Locally: parse `git config --get remote.origin.url` and extract the owner.

use anyhow::{anyhow, Result};
use std::process::Command;

pub fn resolve() -> Result<String> {
	if let Ok(owner) = std::env::var("GITHUB_REPOSITORY_OWNER") {
		if !owner.is_empty() {
			return Ok(owner);
		}
	}

	let output = Command::new("git")
		.args(["config", "--get", "remote.origin.url"])
		.output()
		.map_err(|e| anyhow!("could not run git: {e}"))?;

	if !output.status.success() {
		return Err(anyhow!(
			"no GITHUB_REPOSITORY_OWNER env var and `git config --get remote.origin.url` failed. \
			 Are you in a git repo with a configured remote?"
		));
	}

	let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
	parse_github_owner(&url)
}

/// Parse `git@github.com:OWNER/REPO.git` or `https://github.com/OWNER/REPO.git`
/// (with or without the `.git` suffix) and return `OWNER`.
fn parse_github_owner(url: &str) -> Result<String> {
	let after_host = if let Some(rest) = url.strip_prefix("git@github.com:") {
		rest
	} else if let Some(rest) = url.strip_prefix("https://github.com/") {
		rest
	} else if let Some(rest) = url.strip_prefix("http://github.com/") {
		rest
	} else if let Some(rest) = url.strip_prefix("ssh://git@github.com/") {
		rest
	} else {
		return Err(anyhow!(
			"unrecognised git remote URL `{url}` — expected a github.com remote"
		));
	};

	let owner = after_host
		.split('/')
		.next()
		.ok_or_else(|| anyhow!("could not parse owner from `{url}`"))?;

	if owner.is_empty() {
		return Err(anyhow!("empty owner in remote URL `{url}`"));
	}

	Ok(owner.to_string())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_ssh_remote() {
		assert_eq!(
			parse_github_owner("git@github.com:fitz/gist-id.git").unwrap(),
			"fitz"
		);
	}

	#[test]
	fn parses_https_remote() {
		assert_eq!(
			parse_github_owner("https://github.com/fitz/gist-id.git").unwrap(),
			"fitz"
		);
		assert_eq!(
			parse_github_owner("https://github.com/fitz/gist-id").unwrap(),
			"fitz"
		);
	}

	#[test]
	fn parses_org_remote() {
		assert_eq!(
			parse_github_owner("git@github.com:FigmentEngine/gist-id.git").unwrap(),
			"FigmentEngine"
		);
	}

	#[test]
	fn rejects_non_github() {
		assert!(parse_github_owner("git@gitlab.com:foo/bar.git").is_err());
	}
}
