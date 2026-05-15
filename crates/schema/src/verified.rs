//! Verified-skill schema.

use serde::{Deserialize, Serialize};

/// A skill claim that has matching observable evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedSkill {
	pub name: String,
	pub evidence: Vec<Evidence>,
}

/// A single observable signal supporting a skill claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Evidence {
	GitHubLanguage {
		language: String,
		handle: String,
		#[serde(default)]
		repos: Vec<String>,
	},
}

/// A language observed in the user's public repos that they haven't
/// claimed in their skills. Surfaced in the rendered profile as a nudge
/// to consider adding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuggestedSkill {
	/// GitHub's canonical-cased language name.
	pub language: String,
	/// Full-names ("owner/repo") of repos where this language appears,
	/// ordered most-bytes-first.
	#[serde(default)]
	pub repos: Vec<String>,
}
