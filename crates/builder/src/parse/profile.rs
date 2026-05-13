//! Parse `profile.md` → `Profile`.

use anyhow::{anyhow, Result};
use gist_id_schema::{
	parse_markdown, BlockNode, ExternalIdentity, Markdown, Profile,
};

use super::meta::{extract_metadata, flatten_inline};

pub fn parse_profile(source: &str) -> Result<Profile> {
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	// H1: name
	let name = match iter.next() {
		Some(BlockNode::Heading { level: 1, content }) => flatten_inline(&content),
		_ => return Err(anyhow!("profile.md must start with an H1 (your name)")),
	}
	.trim()
	.to_string();

	// First paragraph: headline
	let headline = match iter.peek() {
		Some(BlockNode::Paragraph(_)) => {
			if let Some(BlockNode::Paragraph(ins)) = iter.next() {
				flatten_inline(&ins).trim().to_string()
			} else {
				String::new()
			}
		}
		_ => String::new(),
	};

	// Optional metadata block + optional `## About` body.
	let remaining: Markdown = iter.collect();
	let section = extract_metadata(&remaining);

	let mut profile = Profile {
		name,
		headline,
		bio: Vec::new(),
		email: None,
		location: None,
		url: None,
		pronouns: None,
		avatar: None,
		external_identities: Vec::new(),
	};

	for (key, value) in &section.meta {
		match key.as_str() {
			"Email" => profile.email = Some(value.clone()),
			"Location" => profile.location = Some(value.clone()),
			"URL" => profile.url = Some(value.clone()),
			"Pronouns" => profile.pronouns = Some(value.clone()),
			"Avatar" => profile.avatar = Some(value.clone()),
			"LinkedIn" | "Mastodon" | "Bluesky" | "Twitter" => {
				profile.external_identities.push(ExternalIdentity {
					network: key.to_ascii_lowercase(),
					handle: value.clone(),
				});
			}
			_ => {} // unknown keys ignored for MVP
		}
	}

	// Walk remaining body for `## About` section — its body becomes `bio`.
	let mut body_iter = section.body.into_iter().peekable();
	while let Some(block) = body_iter.next() {
		if let BlockNode::Heading {
			level: 2,
			ref content,
		} = block
		{
			let heading = flatten_inline(content).to_lowercase();
			if heading.trim() == "about" {
				profile.bio = body_iter.collect();
				break;
			}
		}
	}

	Ok(profile)
}
