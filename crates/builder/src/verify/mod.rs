//! Skill verification via GitHub's public API.
//!
//! Run once per build. Gathers an *evidence set* of programming languages
//! observed in the user's public repository activity, then matches the
//! user's claimed skills against it.
//!
//! Algorithm:
//!   1. List owned public repos (non-forks).
//!   2. List recent public events; extract distinct repos pushed to.
//!   3. For every repo in (1) and (2), fetch its language byte breakdown.
//!   4. Union all language keys across all repos → evidence set.
//!   5. For each claimed skill, lowercase-match against the evidence set.
//!
//! Non-fatal: any API failure logs a warning and produces an empty result.
//! The signed feed remains valid; the rendered profile just shows claims
//! without verification badges.

mod github;
mod matcher;

use anyhow::Result;
use std::collections::BTreeSet;

use gist_id_schema::{SkillCategory, VerifiedSkill};

/// Verify a user's claimed skills against GitHub evidence.
///
/// Returns `Vec<VerifiedSkill>` for the skills that have matching evidence.
/// Skills without evidence are simply absent from the result.
pub async fn verify_skills(handle: &str, skills: &[SkillCategory]) -> Result<Vec<VerifiedSkill>> {
	let token = std::env::var("GITHUB_TOKEN").ok();
	if token.is_none() {
		tracing::warn!("GITHUB_TOKEN not set — skipping skill verification");
		return Ok(Vec::new());
	}
	let token = token.unwrap();

	let client = github::Client::new(token);

	let evidence_set = match build_evidence_set(&client, handle).await {
		Ok(s) => s,
		Err(e) => {
			tracing::warn!("skill verification failed: {e:#}");
			return Ok(Vec::new());
		}
	};

	tracing::info!(
		"Evidence set for {handle}: {} languages",
		evidence_set.len()
	);

	Ok(matcher::match_claims(handle, skills, &evidence_set))
}

async fn build_evidence_set(client: &github::Client, handle: &str) -> Result<BTreeSet<String>> {
	let owned = client.list_owned_public_repos(handle).await?;
	let events = client.list_push_event_repos(handle).await?;

	// Union of non-fork repo full-names.
	let mut all_repos: BTreeSet<String> = BTreeSet::new();
	for r in &owned {
		if !r.fork {
			all_repos.insert(r.full_name.clone());
		}
	}
	// PushEvent repos: we don't know fork-status from the event payload, but
	// list_push_event_repos already filters via the repo lookup. Just merge.
	for r in events {
		all_repos.insert(r);
	}

	tracing::info!(
		"Repos to inspect for {handle}: {} ({} owned non-fork, plus push-event repos)",
		all_repos.len(),
		owned.iter().filter(|r| !r.fork).count()
	);

	// Per-repo language byte breakdown. Aggregate keys.
	let mut langs: BTreeSet<String> = BTreeSet::new();
	for full_name in &all_repos {
		match client.repo_languages(full_name).await {
			Ok(map) => {
				for (lang, _bytes) in map {
					langs.insert(lang);
				}
			}
			Err(e) => {
				tracing::warn!("languages lookup for {full_name} failed: {e:#}");
			}
		}
	}

	Ok(langs)
}
