//! Shared types and wire format for gist.id.
//!
//! Types here travel between the builder (which produces `feed.postcard`),
//! the edge worker (which renders profiles for `gist.id/<handle>`), and the
//! browser client. The crate uses std and works on all three targets.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

pub mod ast;
pub mod parse;
pub use ast::{BlockNode, InlineNode, Markdown};
pub use parse::{parse_markdown, parse_markdown_with_resolver};

/// Schema version. Bump major for breaking layout changes.
pub const SCHEMA_VERSION: u16 = 2;

// ---- Dates ------------------------------------------------------------------

/// A date with optional precision.
///
/// Open-ended ranges (jobs still in progress, etc.) are modelled as
/// `Option<PartialDate>` end fields where `None` means "present".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartialDate {
	Year(i32),
	YearMonth { year: i32, month: u8 },
	YearMonthDay { year: i32, month: u8, day: u8 },
}

impl PartialDate {
	pub fn year(&self) -> i32 {
		match *self {
			PartialDate::Year(y)
			| PartialDate::YearMonth { year: y, .. }
			| PartialDate::YearMonthDay { year: y, .. } => y,
		}
	}

	pub fn to_iso(&self) -> String {
		match *self {
			PartialDate::Year(y) => format!("{y:04}"),
			PartialDate::YearMonth { year, month } => format!("{year:04}-{month:02}"),
			PartialDate::YearMonthDay { year, month, day } => {
				format!("{year:04}-{month:02}-{day:02}")
			}
		}
	}
}

// ---- Profile ----------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
	pub name: String,
	pub headline: String,
	pub bio: Markdown,
	pub email: Option<String>,
	pub location: Option<String>,
	pub url: Option<String>,
	pub pronouns: Option<String>,
	pub avatar: Option<String>,
	pub external_identities: Vec<ExternalIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalIdentity {
	pub network: String,
	pub handle: String,
}

// ---- Resume -----------------------------------------------------------------

/// A company. Roles within a company are listed in the order they appear in
/// the source file (typically newest first). Boomerang employment (leaving
/// and returning) appears as two separate Company entries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Company {
	pub name: String,
	pub url: Option<String>,
	pub roles: Vec<Role>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
	pub title: String,
	pub start: PartialDate,
	/// `None` means present.
	pub end: Option<PartialDate>,
	pub location: Option<String>,
	pub employment_type: Option<String>,
	pub description: Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Education {
	pub institution: String,
	pub start: PartialDate,
	pub end: Option<PartialDate>,
	pub qualification: Option<String>,
	pub field: Option<String>,
	pub location: Option<String>,
	pub url: Option<String>,
	pub score: Option<String>,
	pub description: Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillCategory {
	pub name: String,
	pub skills: Vec<Skill>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
	pub name: String,
	pub since: Option<i32>,
	pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
	pub name: String,
	pub start: Option<PartialDate>,
	pub end: Option<PartialDate>,
	pub url: Option<String>,
	pub roles: Vec<String>,
	pub description: Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patent {
	pub title: String,
	pub number: Option<String>,
	pub status: Option<PatentStatus>,
	pub filed: Option<PartialDate>,
	pub granted: Option<PartialDate>,
	pub office: Option<String>,
	pub url: Option<String>,
	pub description: Markdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatentStatus {
	Filed,
	Pending,
	Granted,
	Lapsed,
}

// ---- Posts ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
	pub date: PartialDate,
	pub slug: String,
	pub title: String,
	pub tags: Vec<String>,
	pub canonical_url: Option<String>,
	pub body: Markdown,
}

// ---- Feed (wire format root) ------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feed {
	pub schema_version: u16,
	pub handle: String,
	pub generated_at: String,
	pub builder_version: String,

	pub profile: Profile,
	pub companies: Vec<Company>,
	pub education: Vec<Education>,
	pub skills: Vec<SkillCategory>,
	pub projects: Vec<Project>,
	pub patents: Vec<Patent>,
	pub posts: Vec<Post>,

	/// Day-4 verification output. Empty for MVP days 1–3.
	pub verified_skills: Vec<VerifiedSkill>,

	pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedSkill {
	pub name: String,
	pub summary: String,
	pub weight: u32,
	pub verified_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
	pub public_key: [u8; 32],
	#[serde(with = "BigArray")]
	pub signature: [u8; 64],
}

impl Signature {
	pub fn empty() -> Self {
		Self {
			public_key: [0; 32],
			signature: [0; 64],
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn partial_date_iso() {
		assert_eq!(PartialDate::Year(2024).to_iso(), "2024");
		assert_eq!(
			PartialDate::YearMonth {
				year: 2024,
				month: 3
			}
			.to_iso(),
			"2024-03"
		);
		assert_eq!(
			PartialDate::YearMonthDay {
				year: 2024,
				month: 3,
				day: 15
			}
			.to_iso(),
			"2024-03-15"
		);
	}

	#[test]
	fn empty_feed_roundtrips() {
		let feed = Feed {
			schema_version: SCHEMA_VERSION,
			handle: "ada".into(),
			generated_at: "2026-05-12T12:00:00Z".into(),
			builder_version: "0.1.0".into(),
			profile: Profile {
				name: "Ada".into(),
				headline: "Engineer".into(),
				bio: vec![],
				email: None,
				location: None,
				url: None,
				pronouns: None,
				avatar: None,
				external_identities: vec![],
			},
			companies: vec![],
			education: vec![],
			skills: vec![],
			projects: vec![],
			patents: vec![],
			posts: vec![],
			verified_skills: vec![],
			signature: Signature::empty(),
		};

		let bytes = postcard::to_allocvec(&feed).unwrap();
		let decoded: Feed = postcard::from_bytes(&bytes).unwrap();
		assert_eq!(feed, decoded);
	}
}
