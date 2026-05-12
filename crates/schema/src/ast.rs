//! Parsed markdown AST.
//!
//! Markdown is parsed by the builder into this tree, embedded in the wire
//! format, and walked by the renderer to emit native nodes or HTML strings.
//! The renderer never touches markdown text and never needs `inner_html`.

use serde::{Deserialize, Serialize};

/// A parsed markdown document: a sequence of block-level nodes.
pub type Markdown = Vec<BlockNode>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockNode {
	Heading {
		level: u8,
		content: Vec<InlineNode>,
	},
	Paragraph(Vec<InlineNode>),
	BlockQuote(Markdown),
	List {
		ordered: bool,
		items: Vec<Markdown>,
	},
	CodeBlock {
		language: Option<String>,
		content: String,
	},
	ThematicBreak,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineNode {
	Text(String),
	Emphasis(Vec<InlineNode>),
	Strong(Vec<InlineNode>),
	Strikethrough(Vec<InlineNode>),
	Code(String),
	Link {
		url: String,
		title: Option<String>,
		content: Vec<InlineNode>,
	},
	Image {
		url: String,
		alt: String,
		title: Option<String>,
	},
	LineBreak,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn roundtrip_simple_document() {
		let doc: Markdown = vec![BlockNode::Heading {
			level: 1,
			content: vec![InlineNode::Text("Hello".into())],
		}];
		let bytes = postcard::to_allocvec(&doc).unwrap();
		let decoded: Markdown = postcard::from_bytes(&bytes).unwrap();
		assert_eq!(doc, decoded);
	}
}
