//! Verified-skill schema.
//!
//! When the builder runs in CI, it queries GitHub's public API for the
//! profile owner's public repo activity and builds an *evidence set* of
//! programming languages observed in their repos. Each claimed skill is
//! then matched (case-insensitive) against that set. Matches become
//! `VerifiedSkill` entries.
//!
//! The renderer pairs claimed skills (which are always shown) with their
//! verification, if any. A claimed skill with no `VerifiedSkill` is still
//! displayed — just without an evidence badge.

use serde::{Deserialize, Serialize};

/// A skill claim that has matching observable evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedSkill {
	/// The skill name as it appears in the user's claimed skills.
	pub name: String,
	/// One or more pieces of evidence supporting this skill.
	pub evidence: Vec<Evidence>,
}

/// A single observable signal supporting a skill claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Evidence {
	/// One or more public repos contain code in this language. The user
	/// either owns those repos or has pushed to them recently.
	GitHubLanguage {
		/// The language name as GitHub reports it (e.g. "Rust", "C++").
		language: String,
		/// GitHub handle the verification applies to.
		handle: String,
		/// Full-names ("owner/repo") of repos providing evidence,
		/// ordered most-bytes-first. The renderer links to the first
		/// entry and may surface the count.
		#[serde(default)]
		repos: Vec<String>,
	},
}
