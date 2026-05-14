//! Match claimed skills against the per-language evidence map.

use std::collections::BTreeSet;

use gist_id_schema::{Evidence, SkillCategory, VerifiedSkill};

use crate::verify::EvidenceMap;

/// For each claimed skill, if its lowercased name matches any lowercased
/// key in the evidence map, produce a `VerifiedSkill` carrying the repos.
pub fn match_claims(
	handle: &str,
	categories: &[SkillCategory],
	evidence: &EvidenceMap,
) -> Vec<VerifiedSkill> {
	// lowercase key → (canonical-cased name, sorted repo list)
	let by_lower: std::collections::BTreeMap<String, (&String, &Vec<(String, u64)>)> = evidence
		.iter()
		.map(|(k, v)| (k.to_lowercase(), (k, v)))
		.collect();

	let mut out: Vec<VerifiedSkill> = Vec::new();
	let mut seen: BTreeSet<String> = BTreeSet::new();

	for cat in categories {
		for skill in &cat.skills {
			let key = skill.name.trim().to_lowercase();
			if key.is_empty() || seen.contains(&key) {
				continue;
			}
			if let Some((canonical, repos)) = by_lower.get(&key) {
				let repo_names: Vec<String> =
					repos.iter().map(|(name, _bytes)| name.clone()).collect();
				out.push(VerifiedSkill {
					name: skill.name.clone(),
					evidence: vec![Evidence::GitHubLanguage {
						language: (*canonical).clone(),
						handle: handle.to_string(),
						repos: repo_names,
					}],
				});
				seen.insert(key);
			}
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;
	use gist_id_schema::Skill;

	fn cat(name: &str, skills: Vec<&str>) -> SkillCategory {
		SkillCategory {
			name: name.into(),
			skills: skills
				.into_iter()
				.map(|s| Skill {
					name: s.into(),
					since: None,
					note: None,
				})
				.collect(),
		}
	}

	fn ev(items: &[(&str, &[(&str, u64)])]) -> EvidenceMap {
		items
			.iter()
			.map(|(lang, repos)| {
				(
					lang.to_string(),
					repos
						.iter()
						.map(|(r, b)| (r.to_string(), *b))
						.collect::<Vec<_>>(),
				)
			})
			.collect()
	}

	#[test]
	fn match_carries_repos_in_byte_order() {
		let cats = vec![cat("Languages", vec!["Rust"])];
		let evidence = ev(&[("Rust", &[("a/big", 100_000), ("b/small", 500)])]);
		let result = match_claims("FigmentEngine", &cats, &evidence);
		assert_eq!(result.len(), 1);
		let Evidence::GitHubLanguage { repos, .. } = &result[0].evidence[0];
		assert_eq!(repos, &vec!["a/big".to_string(), "b/small".to_string()]);
	}

	#[test]
	fn case_insensitive_match() {
		let cats = vec![cat("Languages", vec!["rust", "Python"])];
		let evidence = ev(&[("Rust", &[("a/r", 1)]), ("Python", &[("a/p", 1)])]);
		let result = match_claims("FigmentEngine", &cats, &evidence);
		assert_eq!(result.len(), 2);
	}

	#[test]
	fn no_evidence_means_no_entry() {
		let cats = vec![cat("Languages", vec!["Rust", "XML/XSLT"])];
		let evidence = ev(&[("Rust", &[("a/r", 1)])]);
		let result = match_claims("FigmentEngine", &cats, &evidence);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0].name, "Rust");
	}

	#[test]
	fn duplicates_deduped() {
		let cats = vec![
			cat("Languages", vec!["Rust"]),
			cat("Backend", vec!["rust"]),
		];
		let evidence = ev(&[("Rust", &[("a/r", 1)])]);
		let result = match_claims("FigmentEngine", &cats, &evidence);
		assert_eq!(result.len(), 1);
	}
}
