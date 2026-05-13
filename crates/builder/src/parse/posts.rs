//! Parse a single `posts/YYYY-MM-DD-slug.md` → `Post`.

use anyhow::{anyhow, Result};
use gist_id_schema::{parse_markdown, BlockNode, PartialDate, Post};

use super::meta::{extract_metadata, flatten_inline};

/// Parse a post given its filename (without directory, e.g.
/// `2025-10-29-the-crisis.md`) and its markdown source.
pub fn parse_post(filename: &str, source: &str) -> Result<Post> {
	let (date, slug) = parse_filename(filename)?;
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	let title = match iter.next() {
		Some(BlockNode::Heading { level: 1, content }) => flatten_inline(&content).trim().to_string(),
		_ => return Err(anyhow!("post {filename} must start with an H1 title")),
	};

	let remaining: Vec<BlockNode> = iter.collect();
	let section = extract_metadata(&remaining);

	let tags: Vec<String> = section
		.meta
		.get("Tags")
		.map(|s| {
			s.split(',')
				.map(|t| t.trim().to_string())
				.filter(|t| !t.is_empty())
				.collect()
		})
		.unwrap_or_default();

	let canonical_url = section.meta.get("Canonical").cloned();

	Ok(Post {
		date,
		slug,
		title,
		tags,
		canonical_url,
		body: section.body,
	})
}

fn parse_filename(filename: &str) -> Result<(PartialDate, String)> {
	// Strip optional .md extension.
	let stem = filename.strip_suffix(".md").unwrap_or(filename);

	// Expect `YYYY-MM-DD-slug`.
	if stem.len() < 11 || &stem[10..11] != "-" {
		return Err(anyhow!(
			"post filename {filename} must be `YYYY-MM-DD-slug.md`"
		));
	}
	let date_part = &stem[..10];
	let slug = stem[11..].to_string();

	let mut parts = date_part.split('-');
	let year: i32 = parts
		.next()
		.and_then(|s| s.parse().ok())
		.ok_or_else(|| anyhow!("bad year in {filename}"))?;
	let month: u8 = parts
		.next()
		.and_then(|s| s.parse().ok())
		.ok_or_else(|| anyhow!("bad month in {filename}"))?;
	let day: u8 = parts
		.next()
		.and_then(|s| s.parse().ok())
		.ok_or_else(|| anyhow!("bad day in {filename}"))?;

	Ok((PartialDate::YearMonthDay { year, month, day }, slug))
}
