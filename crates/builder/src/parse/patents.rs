//! Parse `resume/patents.md` → `Vec<Patent>`.

use anyhow::Result;
use gist_id_schema::{parse_markdown, BlockNode, Patent, PatentStatus};

use super::meta::{extract_metadata, flatten_inline, parse_date};

pub fn parse_patents(source: &str) -> Result<Vec<Patent>> {
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	if matches!(iter.peek(), Some(BlockNode::Heading { level: 1, .. })) {
		iter.next();
	}

	let mut out: Vec<Patent> = Vec::new();
	let mut current_title: Option<String> = None;
	let mut current_blocks: Vec<BlockNode> = Vec::new();

	for block in iter {
		match block {
			BlockNode::Heading {
				level: 2,
				ref content,
			} => {
				flush(&mut out, &mut current_title, &mut current_blocks);
				current_title = Some(flatten_inline(content).trim().to_string());
			}
			other => {
				if current_title.is_some() {
					current_blocks.push(other);
				}
			}
		}
	}
	flush(&mut out, &mut current_title, &mut current_blocks);

	Ok(out)
}

fn flush(
	out: &mut Vec<Patent>,
	title: &mut Option<String>,
	blocks: &mut Vec<BlockNode>,
) {
	let Some(patent_title) = title.take() else {
		blocks.clear();
		return;
	};
	let drained = std::mem::take(blocks);
	let section = extract_metadata(&drained);

	let status = section
		.meta
		.get("Status")
		.map(|s| s.to_ascii_lowercase())
		.and_then(|s| match s.as_str() {
			"filed" => Some(PatentStatus::Filed),
			"pending" => Some(PatentStatus::Pending),
			"granted" => Some(PatentStatus::Granted),
			"lapsed" => Some(PatentStatus::Lapsed),
			_ => None,
		});

	out.push(Patent {
		title: patent_title,
		number: section.meta.get("Number").cloned(),
		status,
		filed: section.meta.get("Filed").and_then(|s| parse_date(s)),
		granted: section.meta.get("Granted").and_then(|s| parse_date(s)),
		office: section.meta.get("Office").cloned(),
		url: section.meta.get("URL").cloned(),
		description: section.body,
	});
}
