//! Parse `resume/skills.md` → `Vec<SkillCategory>`.
//!
//! Format: `##` is a category, body is a single bullet list of skills.

use anyhow::Result;
use gist_id_schema::{parse_markdown, BlockNode, Skill, SkillCategory};

use super::meta::flatten_inline;

pub fn parse_skills(source: &str) -> Result<Vec<SkillCategory>> {
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	if matches!(iter.peek(), Some(BlockNode::Heading { level: 1, .. })) {
		iter.next();
	}

	let mut categories: Vec<SkillCategory> = Vec::new();
	let mut current_name: Option<String> = None;

	for block in iter {
		match block {
			BlockNode::Heading {
				level: 2,
				ref content,
			} => {
				current_name = Some(flatten_inline(content).trim().to_string());
				categories.push(SkillCategory {
					name: current_name.clone().unwrap(),
					skills: Vec::new(),
				});
			}
			BlockNode::List { items, .. } if current_name.is_some() => {
				let last = categories.last_mut().unwrap();
				for item in items {
					if let Some(skill) = parse_skill_item(&item) {
						last.skills.push(skill);
					}
				}
			}
			_ => {}
		}
	}

	Ok(categories)
}

fn parse_skill_item(item: &[BlockNode]) -> Option<Skill> {
	let first = item.first()?;
	let inlines = match first {
		BlockNode::Paragraph(ins) => ins,
		_ => return None,
	};
	let text = flatten_inline(inlines).trim().to_string();
	if text.is_empty() {
		return None;
	}

	// Parse "Skill (note)" form: extract anything in parentheses as note.
	// "Rust (since 2018)" → name="Rust", note="since 2018", since=2018 if parses.
	if let Some(open) = text.find('(') {
		if text.ends_with(')') {
			let name = text[..open].trim().to_string();
			let note = text[open + 1..text.len() - 1].trim().to_string();
			let since = parse_since(&note);
			return Some(Skill {
				name,
				since,
				note: if note.is_empty() { None } else { Some(note) },
			});
		}
	}

	Some(Skill {
		name: text,
		since: None,
		note: None,
	})
}

fn parse_since(note: &str) -> Option<i32> {
	// "since 2018" → 2018
	let lower = note.to_ascii_lowercase();
	let after = lower.strip_prefix("since ")?;
	after.trim().parse().ok()
}
