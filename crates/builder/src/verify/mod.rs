//! Skill verification + suggestion via GitHub's public API.

mod github;
mod matcher;

use anyhow::Result;
use std::collections::BTreeMap;

use gist_id_schema::{SkillCategory, SuggestedSkill, VerifiedSkill};

/// Map language → list of (repo_full_name, bytes), highest-bytes-first.
pub type EvidenceMap = BTreeMap<String, Vec<(String, u64)>>;

/// Output of one verification pass: verified claims + suggested additions.
pub struct VerifyOutput {
	pub verified: Vec<VerifiedSkill>,
	pub suggested: Vec<SuggestedSkill>,
}

const SUGGESTION_LIMIT: usize = 5;

/// Verify a user's claimed skills against GitHub evidence, and produce
/// suggestions for unclaimed languages we found.
pub async fn verify_skills(handle: &str, skills: &[SkillCategory]) -> Result<VerifyOutput> {
	let token = std::env::var("GITHUB_TOKEN").ok();
	if token.is_none() {
		tracing::warn!("GITHUB_TOKEN not set — skipping skill verification");
		return Ok(VerifyOutput {
			verified: Vec::new(),
			suggested: Vec::new(),
		});
	}
	let token = token.unwrap();

	let client = github::Client::new(token);

	let evidence_map = match build_evidence_map(&client, handle).await {
		Ok(m) => m,
		Err(e) => {
			tracing::warn!("skill verification failed: {e:#}");
			return Ok(VerifyOutput {
				verified: Vec::new(),
				suggested: Vec::new(),
			});
		}
	};

	tracing::info!(
		"Evidence map for {handle}: {} languages",
		evidence_map.len()
	);

	let verified = matcher::match_claims(handle, skills, &evidence_map);
	let suggested = build_suggestions(skills, &evidence_map);

	Ok(VerifyOutput {
		verified,
		suggested,
	})
}

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

	for repos in map.values_mut() {
		repos.sort_by(|a, b| b.1.cmp(&a.1));
	}

	Ok(map)
}

/// Top-N languages by total bytes across all repos, excluding any
/// language already claimed (case-insensitive) by the user.
fn build_suggestions(
	claimed: &[SkillCategory],
	evidence: &EvidenceMap,
) -> Vec<SuggestedSkill> {
	let claimed_lower: std::collections::BTreeSet<String> = claimed
		.iter()
		.flat_map(|c| c.skills.iter().map(|s| s.name.trim().to_lowercase()))
		.collect();

	// Compute totals per language; skip claimed.
	let mut totals: Vec<(&String, &Vec<(String, u64)>, u64)> = evidence
		.iter()
		.filter(|(lang, _)| !claimed_lower.contains(&lang.to_lowercase()))
		.map(|(lang, repos)| {
			let sum: u64 = repos.iter().map(|(_, b)| *b).sum();
			(lang, repos, sum)
		})
		.collect();

	totals.sort_by(|a, b| b.2.cmp(&a.2));
	totals.truncate(SUGGESTION_LIMIT);

	totals
		.into_iter()
		.map(|(lang, repos, _)| SuggestedSkill {
			language: lang.clone(),
			repos: repos.iter().map(|(name, _)| name.clone()).collect(),
		})
		.collect()
}
