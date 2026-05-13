//! Parse `resume/projects.md` → `Vec<Project>`.

use anyhow::Result;
use gist_id_schema::{parse_markdown, BlockNode, Project};

use super::meta::{extract_metadata, flatten_inline, parse_date};

pub fn parse_projects(source: &str) -> Result<Vec<Project>> {
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	if matches!(iter.peek(), Some(BlockNode::Heading { level: 1, .. })) {
		iter.next();
	}

	let mut out: Vec<Project> = Vec::new();
	let mut current_name: Option<String> = None;
	let mut current_blocks: Vec<BlockNode> = Vec::new();

	for block in iter {
		match block {
			BlockNode::Heading {
				level: 2,
				ref content,
			} => {
				flush(&mut out, &mut current_name, &mut current_blocks);
				current_name = Some(flatten_inline(content).trim().to_string());
			}
			other => {
				if current_name.is_some() {
					current_blocks.push(other);
				}
			}
		}
	}
	flush(&mut out, &mut current_name, &mut current_blocks);

	Ok(out)
}

fn flush(
	out: &mut Vec<Project>,
	name: &mut Option<String>,
	blocks: &mut Vec<BlockNode>,
) {
	let Some(project_name) = name.take() else {
		blocks.clear();
		return;
	};
	let drained = std::mem::take(blocks);
	let section = extract_metadata(&drained);

	let roles: Vec<String> = section
		.meta
		.get("Roles")
		.map(|s| {
			s.split(',')
				.map(|p| p.trim().to_string())
				.filter(|p| !p.is_empty())
				.collect()
		})
		.unwrap_or_default();

	out.push(Project {
		name: project_name,
		start: section.meta.get("Start").and_then(|s| parse_date(s)),
		end: section.meta.get("End").and_then(|s| parse_date(s)),
		url: section.meta.get("URL").cloned(),
		roles,
		description: section.body,
	});
}
