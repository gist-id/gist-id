//! Shared helpers: metadata-block extraction and date parsing.

use std::collections::BTreeMap;

use gist_id_schema::{BlockNode, InlineNode, Markdown, PartialDate};

/// A parsed metadata block plus the remaining prose body.
pub struct Section {
	pub meta: BTreeMap<String, String>,
	pub body: Markdown,
}

/// Given a sequence of blocks that follow a heading, peel off any leading
/// list-as-metadata-block and return the metadata plus the rest of the body.
///
/// A list is treated as metadata if every item's first inline node is a
/// `Text` node matching `Key: value`. Mixed lists fall through unchanged
/// into the body.
pub fn extract_metadata(blocks: &[BlockNode]) -> Section {
	let mut meta = BTreeMap::new();

	// Look at the first block. If it's a list whose every item is a
	// "Key: value" line, consume it as metadata.
	if let Some(BlockNode::List { items, .. }) = blocks.first() {
		if let Some(parsed) = parse_metadata_list(items) {
			meta = parsed;
			return Section {
				meta,
				body: blocks[1..].to_vec(),
			};
		}
	}

	Section {
		meta,
		body: blocks.to_vec(),
	}
}

fn parse_metadata_list(items: &[Markdown]) -> Option<BTreeMap<String, String>> {
	let mut out = BTreeMap::new();
	for item in items {
		let line = flatten_first_paragraph(item)?;
		let (k, v) = split_kv(&line)?;
		out.insert(k, v);
	}
	if out.is_empty() {
		None
	} else {
		Some(out)
	}
}

/// Extract the plain-text content of a list item's first paragraph (or, for
/// tight lists, the direct inline content). Returns None if the item isn't
/// a single line of text.
fn flatten_first_paragraph(item: &Markdown) -> Option<String> {
	let first = item.first()?;
	let inlines = match first {
		BlockNode::Paragraph(ins) => ins,
		_ => return None,
	};
	let text = flatten_inline(inlines);
	Some(text.trim().to_string())
}

pub fn flatten_inline(inlines: &[InlineNode]) -> String {
	let mut out = String::new();
	for n in inlines {
		walk(&mut out, n);
	}
	out
}

fn walk(out: &mut String, n: &InlineNode) {
	match n {
		InlineNode::Text(t) | InlineNode::Code(t) => out.push_str(t),
		InlineNode::Emphasis(c)
		| InlineNode::Strong(c)
		| InlineNode::Strikethrough(c)
		| InlineNode::Link { content: c, .. } => {
			for x in c {
				walk(out, x);
			}
		}
		InlineNode::Image { alt, .. } => out.push_str(alt),
		InlineNode::LineBreak => out.push(' '),
	}
}

fn split_kv(line: &str) -> Option<(String, String)> {
	let (k, v) = line.split_once(':')?;
	let k = k.trim().to_string();
	let v = v.trim().to_string();
	if k.is_empty() {
		return None;
	}
	Some((k, v))
}

/// Parse a date string from a metadata block. Accepts:
///  - `2024`
///  - `2024-03`
///  - `2024-03-15`
///  - `present` (returns None — the caller decides what that means)
pub fn parse_date(s: &str) -> Option<PartialDate> {
	let s = s.trim();
	if s.is_empty() || s.eq_ignore_ascii_case("present") {
		return None;
	}
	let parts: Vec<&str> = s.split('-').collect();
	match parts.as_slice() {
		[y, m, d] => Some(PartialDate::YearMonthDay {
			year: y.parse().ok()?,
			month: m.parse().ok()?,
			day: d.parse().ok()?,
		}),
		[y, m] => Some(PartialDate::YearMonth {
			year: y.parse().ok()?,
			month: m.parse().ok()?,
		}),
		[y] => Some(PartialDate::Year(y.parse().ok()?)),
		_ => None,
	}
}
