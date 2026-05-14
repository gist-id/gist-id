//! Match claimed skills against the evidence set.
//!
//! Case-insensitive direct string equality. Whitespace trimmed.
//! "Javascript" → "javascript" matches GitHub's "JavaScript" → "javascript".
//! "C/C++/Visual Basic" doesn't match any single language → unverified.

use std::collections::BTreeSet;

use gist_id_schema::{Evidence, SkillCategory, VerifiedSkill};

/// For each claimed skill, if its lowercased name matches any lowercased
/// entry in the evidence set, produce a `VerifiedSkill`. Skills with no
/// match are absent from the result.
pub fn match_claims(
	handle: &str,
	categories: &[SkillCategory],
	evidence_set: &BTreeSet<String>,
) -> Vec<VerifiedSkill> {
	// Build a lowercase → canonical-cased lookup for the evidence set so we
	// can pass GitHub's canonical name through to the renderer for the URL.
	let canonical: std::collections::BTreeMap<String, String> = evidence_set
		.iter()
		.map(|s| (s.to_lowercase(), s.clone()))
		.collect();

	let mut out: Vec<VerifiedSkill> = Vec::new();
	let mut seen: BTreeSet<String> = BTreeSet::new();

	for cat in categories {
		for skill in &cat.skills {
			let key = skill.name.trim().to_lowercase();
			if key.is_empty() || seen.contains(&key) {
				continue;
			}
			if let Some(canonical_name) = canonical.get(&key) {
				out.push(VerifiedSkill {
					name: skill.name.clone(),
					evidence: vec![Evidence::GitHubLanguage {
						language: canonical_name.clone(),
						handle: handle.to_string(),
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

	fn evidence(items: &[&str]) -> BTreeSet<String> {
		items.iter().map(|s| s.to_string()).collect()
	}

	#[test]
	fn case_insensitive_match() {
		let cats = vec![cat("Languages", vec!["rust", "Python"])];
		let ev = evidence(&["Rust", "Python"]);
		let result = match_claims("FigmentEngine", &cats, &ev);
		assert_eq!(result.len(), 2);
		assert_eq!(result[0].name, "rust");
		assert!(matches!(
			&result[0].evidence[0],
			Evidence::GitHubLanguage { language, .. } if language == "Rust"
		));
	}

	#[test]
	fn no_evidence_means_no_entry() {
		let cats = vec![cat("Languages", vec!["Rust", "XML/XSLT"])];
		let ev = evidence(&["Rust"]); // GitHub doesn't typically report XML as primary
		let result = match_claims("FigmentEngine", &cats, &ev);
		assert_eq!(result.len(), 1);
		assert_eq!(result[0].name, "Rust");
	}

	#[test]
	fn duplicate_claims_deduped() {
		let cats = vec![
			cat("Languages", vec!["Rust"]),
			cat("Backend", vec!["rust"]),
		];
		let ev = evidence(&["Rust"]);
		let result = match_claims("FigmentEngine", &cats, &ev);
		assert_eq!(result.len(), 1);
	}

	#[test]
	fn no_evidence_set_no_matches() {
		let cats = vec![cat("Languages", vec!["Rust"])];
		let ev = BTreeSet::new();
		let result = match_claims("FigmentEngine", &cats, &ev);
		assert!(result.is_empty());
	}
}
