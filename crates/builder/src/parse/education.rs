//! Parse `resume/education.md` → `Vec<Education>`.

use anyhow::Result;
use gist_id_schema::{parse_markdown, BlockNode, Education};

use super::meta::{extract_metadata, flatten_inline, parse_date};

pub fn parse_education(source: &str) -> Result<Vec<Education>> {
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	// Optional H1 file title
	if matches!(iter.peek(), Some(BlockNode::Heading { level: 1, .. })) {
		iter.next();
	}

	let mut entries: Vec<Education> = Vec::new();
	let mut current_name: Option<String> = None;
	let mut current_blocks: Vec<BlockNode> = Vec::new();

	for block in iter {
		match block {
			BlockNode::Heading {
				level: 2,
				ref content,
			} => {
				flush(&mut entries, &mut current_name, &mut current_blocks);
				current_name = Some(flatten_inline(content).trim().to_string());
			}
			other => {
				if current_name.is_some() {
					current_blocks.push(other);
				}
			}
		}
	}
	flush(&mut entries, &mut current_name, &mut current_blocks);

	Ok(entries)
}

fn flush(
	entries: &mut Vec<Education>,
	name: &mut Option<String>,
	blocks: &mut Vec<BlockNode>,
) {
	let Some(institution) = name.take() else {
		blocks.clear();
		return;
	};
	let drained = std::mem::take(blocks);
	let section = extract_metadata(&drained);

	let start = section
		.meta
		.get("Start")
		.and_then(|s| parse_date(s))
		.unwrap_or(gist_id_schema::PartialDate::Year(0));
	let end = section.meta.get("End").and_then(|s| parse_date(s));

	entries.push(Education {
		institution,
		start,
		end,
		qualification: section.meta.get("Qualification").cloned(),
		field: section.meta.get("Field").cloned(),
		location: section.meta.get("Location").cloned(),
		url: section.meta.get("URL").cloned(),
		score: section.meta.get("Score").cloned(),
		description: section.body,
	});
}
