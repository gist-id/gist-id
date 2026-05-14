//! Minimal GitHub REST API client for skill verification.
//!
//! Uses `reqwest` (already a dependency for native non-WASM builds).
//! Authenticated via a token passed in `Authorization: Bearer <token>`.
//! Pagination handled via the `Link` header for endpoints that return
//! arrays.

use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;

use serde::Deserialize;

const API_BASE: &str = "https://api.github.com";
const USER_AGENT: &str = concat!("gist-id-builder/", env!("CARGO_PKG_VERSION"));

pub struct Client {
	http: reqwest::Client,
	token: String,
}

impl Client {
	pub fn new(token: String) -> Self {
		Self {
			http: reqwest::Client::new(),
			token,
		}
	}

	/// `GET /users/<handle>/repos?type=owner` — paginated.
	pub async fn list_owned_public_repos(&self, handle: &str) -> Result<Vec<Repo>> {
		let mut url = format!("{API_BASE}/users/{handle}/repos?per_page=100&type=owner");
		let mut out: Vec<Repo> = Vec::new();
		loop {
			let resp = self.send(&url).await?;
			let next = next_page(resp.headers().get("link").and_then(|v| v.to_str().ok()));
			let page: Vec<Repo> = resp.json().await.context("decoding repos page")?;
			out.extend(page);
			match next {
				Some(n) => url = n,
				None => break,
			}
		}
		Ok(out)
	}

	/// `GET /users/<handle>/events/public?per_page=100` — limited to the
	/// most recent 90 days, ~300 events. Returns unique repo full-names
	/// from PushEvents (with forks filtered out via a follow-up repo
	/// metadata lookup).
	pub async fn list_push_event_repos(&self, handle: &str) -> Result<Vec<String>> {
		let url = format!("{API_BASE}/users/{handle}/events/public?per_page=100");
		let resp = self.send(&url).await?;
		let events: Vec<Event> = resp.json().await.context("decoding events page")?;

		let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
		for ev in events {
			if ev.event_type == "PushEvent" {
				names.insert(ev.repo.name);
			}
		}

		// Filter out forks by fetching repo metadata.
		let mut out: Vec<String> = Vec::new();
		for name in names {
			match self.repo_metadata(&name).await {
				Ok(r) if !r.fork => out.push(name),
				Ok(_) => {} // fork, skip
				Err(e) => {
					tracing::warn!("metadata lookup for {name} failed: {e:#}");
				}
			}
		}
		Ok(out)
	}

	/// `GET /repos/<owner>/<repo>` — repo metadata.
	pub async fn repo_metadata(&self, full_name: &str) -> Result<Repo> {
		let url = format!("{API_BASE}/repos/{full_name}");
		let resp = self.send(&url).await?;
		resp.json::<Repo>()
			.await
			.with_context(|| format!("decoding metadata for {full_name}"))
	}

	/// `GET /repos/<owner>/<repo>/languages` — byte breakdown by language.
	pub async fn repo_languages(&self, full_name: &str) -> Result<BTreeMap<String, u64>> {
		let url = format!("{API_BASE}/repos/{full_name}/languages");
		let resp = self.send(&url).await?;
		resp.json::<BTreeMap<String, u64>>()
			.await
			.with_context(|| format!("decoding languages for {full_name}"))
	}

	async fn send(&self, url: &str) -> Result<reqwest::Response> {
		let resp = self
			.http
			.get(url)
			.header("user-agent", USER_AGENT)
			.header("accept", "application/vnd.github+json")
			.header("x-github-api-version", "2022-11-28")
			.bearer_auth(&self.token)
			.send()
			.await
			.with_context(|| format!("GET {url}"))?;

		if !resp.status().is_success() {
			let status = resp.status();
			let body = resp.text().await.unwrap_or_default();
			return Err(anyhow!("GET {url} → {status}: {body}"));
		}
		Ok(resp)
	}
}

#[derive(Debug, Deserialize)]
pub struct Repo {
	pub full_name: String,
	#[serde(default)]
	pub fork: bool,
}

#[derive(Debug, Deserialize)]
struct Event {
	#[serde(rename = "type")]
	event_type: String,
	repo: EventRepo,
}

#[derive(Debug, Deserialize)]
struct EventRepo {
	name: String, // `owner/repo`
}

/// Parse a `Link` header to find the `rel="next"` URL.
fn next_page(link_header: Option<&str>) -> Option<String> {
	let link = link_header?;
	// Format: `<url1>; rel="next", <url2>; rel="last"`
	for part in link.split(',') {
		let part = part.trim();
		if part.ends_with("rel=\"next\"") {
			let start = part.find('<')?;
			let end = part.find('>')?;
			return Some(part[start + 1..end].to_string());
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn next_page_extracts_url() {
		let h = "<https://api.github.com/u/x/repos?page=2>; rel=\"next\", \
		         <https://api.github.com/u/x/repos?page=5>; rel=\"last\"";
		assert_eq!(
			next_page(Some(h)).as_deref(),
			Some("https://api.github.com/u/x/repos?page=2")
		);
	}

	#[test]
	fn next_page_none_when_absent() {
		let h = "<https://api.github.com/u/x/repos?page=5>; rel=\"last\"";
		assert_eq!(next_page(Some(h)), None);
	}
}
