//! Skill verification via GitHub's public API.
//!
//! Run once per build. Gathers a per-language map of repos providing
//! evidence, then matches the user's claimed skills against it.
//!
//! Algorithm:
//!   1. List owned public repos (non-forks).
//!   2. List recent public events; extract distinct repos pushed to,
//!      filtering forks via a repo-metadata lookup.
//!   3. For every repo in (1) and (2), fetch its language byte breakdown.
//!   4. Build a map: language → [repos with bytes in that language],
//!      ordered most-bytes-first.
//!   5. For each claimed skill, lowercase-match against the map keys.
//!
//! Non-fatal: any API failure logs a warning and produces an empty result.

mod github;
mod matcher;

use anyhow::Result;
use std::collections::BTreeMap;

use gist_id_schema::{SkillCategory, VerifiedSkill};

/// Verify a user's claimed skills against GitHub evidence.
pub async fn verify_skills(handle: &str, skills: &[SkillCategory]) -> Result<Vec<VerifiedSkill>> {
	let token = std::env::var("GITHUB_TOKEN").ok();
	if token.is_none() {
		tracing::warn!("GITHUB_TOKEN not set — skipping skill verification");
		return Ok(Vec::new());
	}
	let token = token.unwrap();

	let client = github::Client::new(token);

	let evidence_map = match build_evidence_map(&client, handle).await {
		Ok(m) => m,
		Err(e) => {
			tracing::warn!("skill verification failed: {e:#}");
			return Ok(Vec::new());
		}
	};

	tracing::info!(
		"Evidence map for {handle}: {} languages",
		evidence_map.len()
	);

	Ok(matcher::match_claims(handle, skills, &evidence_map))
}

/// Map language → list of (repo_full_name, bytes), highest-bytes-first.
pub type EvidenceMap = BTreeMap<String, Vec<(String, u64)>>;

async fn build_evidence_map(client: &github::Client, handle: &str) -> Result<EvidenceMap> {
	let owned = client.list_owned_public_repos(handle).await?;
	let events = client.list_push_event_repos(handle).await?;

	let mut all_repos: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
	for r in &owned {
		if !r.fork {
			all_repos.insert(r.full_name.clone());
		}
	}
	for r in events {
		all_repos.insert(r);
	}

	tracing::info!(
		"Repos to inspect for {handle}: {} ({} owned non-fork, plus push-event repos)",
		all_repos.len(),
		owned.iter().filter(|r| !r.fork).count()
	);
	for r in &all_repos {
		tracing::info!("  inspecting: {r}");
	}

	// language → vec of (repo, bytes)
	let mut map: EvidenceMap = BTreeMap::new();
	for full_name in &all_repos {
		match client.repo_languages(full_name).await {
			Ok(langs) => {
				for (lang, bytes) in langs {
					map.entry(lang)
						.or_default()
						.push((full_name.clone(), bytes));
				}
			}
			Err(e) => {
				tracing::warn!("languages lookup for {full_name} failed: {e:#}");
			}
		}
	}

	// Sort each language's repo list by bytes descending.
	for repos in map.values_mut() {
		repos.sort_by(|a, b| b.1.cmp(&a.1));
	}

	Ok(map)
}
