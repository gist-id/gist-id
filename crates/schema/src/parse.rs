//! Markdown to AST conversion.
//!
//! Uses pulldown-cmark's event stream and walks it into our tree shape.
//! The renderer never sees markdown text — only the resulting AST.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::ast::{BlockNode, InlineNode, Markdown};

/// Parse a markdown string into our AST.
///
/// Image URLs are passed through unchanged. Use `parse_markdown_with_resolver`
/// to rewrite relative paths (e.g. for the builder's GitHub raw URL rewriting).
pub fn parse_markdown(input: &str) -> Markdown {
	parse_markdown_with_resolver(input, |url| url.to_string())
}

/// Parse markdown, applying `resolve_image_url` to every image URL.
pub fn parse_markdown_with_resolver<F>(input: &str, resolve_image_url: F) -> Markdown
where
	F: Fn(&str) -> String,
{
	let mut options = Options::empty();
	options.insert(Options::ENABLE_STRIKETHROUGH);

	let parser = Parser::new_ext(input, options);
	let mut builder = AstBuilder::new(resolve_image_url);
	for event in parser {
		builder.handle(event);
	}
	builder.finish()
}

// ---- Builder ----------------------------------------------------------------

struct AstBuilder<F: Fn(&str) -> String> {
	/// Top-level blocks accumulated so far.
	blocks: Vec<BlockNode>,
	/// Stack of partially-constructed blocks (nested via blockquotes, lists).
	block_stack: Vec<BlockFrame>,
	/// Stack of partially-constructed inline nodes.
	inline_stack: Vec<InlineFrame>,
	/// Inline content waiting to be attached to the innermost block on close.
	pending_inline: Vec<InlineNode>,
	resolve_image_url: F,
}

#[derive(Debug)]
enum BlockFrame {
	Heading {
		level: u8,
	},
	Paragraph,
	BlockQuote {
		children: Markdown,
	},
	List {
		ordered: bool,
		items: Vec<Markdown>,
	},
	ListItem {
		children: Markdown,
	},
	CodeBlock {
		language: Option<String>,
		buf: String,
	},
}

#[derive(Debug)]
enum InlineFrame {
	Emphasis(Vec<InlineNode>),
	Strong(Vec<InlineNode>),
	Strikethrough(Vec<InlineNode>),
	Link {
		url: String,
		title: Option<String>,
		content: Vec<InlineNode>,
	},
	/// Image frames accumulate alt text via Text events.
	Image {
		url: String,
		title: Option<String>,
		content: Vec<InlineNode>,
	},
}

impl<F: Fn(&str) -> String> AstBuilder<F> {
	fn new(resolve_image_url: F) -> Self {
		Self {
			blocks: Vec::new(),
			block_stack: Vec::new(),
			inline_stack: Vec::new(),
			pending_inline: Vec::new(),
			resolve_image_url,
		}
	}

	fn finish(self) -> Markdown {
		self.blocks
	}

	fn handle(&mut self, event: Event<'_>) {
		match event {
			Event::Start(tag) => self.start_tag(tag),
			Event::End(tag) => self.end_tag(tag),
			Event::Text(text) => self.handle_text(text.into_string()),
			Event::Code(text) => self.push_inline(InlineNode::Code(text.into_string())),
			Event::SoftBreak | Event::HardBreak => {
				self.push_inline(InlineNode::LineBreak);
			}
			Event::Rule => self.push_block(BlockNode::ThematicBreak),
			Event::Html(_) | Event::InlineHtml(_) => {
				// Raw HTML is disallowed by the layout spec — strip silently.
			}
			_ => {}
		}
	}

	/// Route text events: into code-block buffer if one is open, otherwise
	/// into the inline content stream.
	fn handle_text(&mut self, text: String) {
		if let Some(BlockFrame::CodeBlock { buf, .. }) = self.block_stack.last_mut() {
			buf.push_str(&text);
			return;
		}
		self.push_inline(InlineNode::Text(text));
	}

	fn start_tag(&mut self, tag: Tag<'_>) {
		match tag {
			Tag::Heading { level, .. } => {
				self.block_stack
					.push(BlockFrame::Heading { level: level as u8 });
			}
			Tag::Paragraph => {
				self.block_stack.push(BlockFrame::Paragraph);
			}
			Tag::BlockQuote(_) => {
				self.block_stack.push(BlockFrame::BlockQuote {
					children: Vec::new(),
				});
			}
			Tag::List(first_number) => {
				self.block_stack.push(BlockFrame::List {
					ordered: first_number.is_some(),
					items: Vec::new(),
				});
			}
			Tag::Item => {
				self.block_stack.push(BlockFrame::ListItem {
					children: Vec::new(),
				});
			}
			Tag::CodeBlock(kind) => {
				let language = match kind {
					CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.into_string()),
					_ => None,
				};
				self.block_stack.push(BlockFrame::CodeBlock {
					language,
					buf: String::new(),
				});
			}
			Tag::Emphasis => {
				self.inline_stack.push(InlineFrame::Emphasis(Vec::new()));
			}
			Tag::Strong => {
				self.inline_stack.push(InlineFrame::Strong(Vec::new()));
			}
			Tag::Strikethrough => {
				self.inline_stack
					.push(InlineFrame::Strikethrough(Vec::new()));
			}
			Tag::Link {
				dest_url, title, ..
			} => {
				let title_opt = optional_string(title.into_string());
				self.inline_stack.push(InlineFrame::Link {
					url: dest_url.into_string(),
					title: title_opt,
					content: Vec::new(),
				});
			}
			Tag::Image {
				dest_url, title, ..
			} => {
				let title_opt = optional_string(title.into_string());
				let resolved_url = (self.resolve_image_url)(&dest_url);
				self.inline_stack.push(InlineFrame::Image {
					url: resolved_url,
					title: title_opt,
					content: Vec::new(),
				});
			}
			_ => {}
		}
	}

	fn end_tag(&mut self, tag: TagEnd) {
		match tag {
			TagEnd::Heading(_) => {
				if let Some(BlockFrame::Heading { level }) = self.block_stack.pop() {
					let content = std::mem::take(&mut self.pending_inline);
					self.push_block(BlockNode::Heading { level, content });
				}
			}
			TagEnd::Paragraph => {
				if let Some(BlockFrame::Paragraph) = self.block_stack.pop() {
					let content = std::mem::take(&mut self.pending_inline);
					self.push_block(BlockNode::Paragraph(content));
				}
			}
			TagEnd::BlockQuote(_) => {
				if !self.pending_inline.is_empty() {
					let content = std::mem::take(&mut self.pending_inline);
					if let Some(BlockFrame::BlockQuote { children }) = self.block_stack.last_mut() {
						children.push(BlockNode::Paragraph(content));
					}
				}
				if let Some(BlockFrame::BlockQuote { children }) = self.block_stack.pop() {
					self.push_block(BlockNode::BlockQuote(children));
				}
			}
			TagEnd::List(_) => {
				if let Some(BlockFrame::List { ordered, items }) = self.block_stack.pop() {
					self.push_block(BlockNode::List { ordered, items });
				}
			}
			TagEnd::Item => {
				// Flush any pending inline content as an implicit paragraph
				// (tight lists don't emit explicit Paragraph wrappers).
				if !self.pending_inline.is_empty() {
					let content = std::mem::take(&mut self.pending_inline);
					if let Some(BlockFrame::ListItem { children }) = self.block_stack.last_mut() {
						children.push(BlockNode::Paragraph(content));
					}
				}
				if let Some(BlockFrame::ListItem { children }) = self.block_stack.pop() {
					if let Some(BlockFrame::List { items, .. }) = self.block_stack.last_mut() {
						items.push(children);
					}
				}
			}
			TagEnd::CodeBlock => {
				if let Some(BlockFrame::CodeBlock { language, buf }) = self.block_stack.pop() {
					self.push_block(BlockNode::CodeBlock {
						language,
						content: buf,
					});
				}
			}
			TagEnd::Emphasis => {
				if let Some(InlineFrame::Emphasis(content)) = self.inline_stack.pop() {
					self.push_inline(InlineNode::Emphasis(content));
				}
			}
			TagEnd::Strong => {
				if let Some(InlineFrame::Strong(content)) = self.inline_stack.pop() {
					self.push_inline(InlineNode::Strong(content));
				}
			}
			TagEnd::Strikethrough => {
				if let Some(InlineFrame::Strikethrough(content)) = self.inline_stack.pop() {
					self.push_inline(InlineNode::Strikethrough(content));
				}
			}
			TagEnd::Link => {
				if let Some(InlineFrame::Link {
					url,
					title,
					content,
				}) = self.inline_stack.pop()
				{
					self.push_inline(InlineNode::Link {
						url,
						title,
						content,
					});
				}
			}
			TagEnd::Image => {
				if let Some(InlineFrame::Image {
					url,
					title,
					content,
				}) = self.inline_stack.pop()
				{
					let alt = flatten_inline_text(&content);
					self.push_inline(InlineNode::Image { url, alt, title });
				}
			}
			_ => {}
		}
	}

	/// Push an inline node into the innermost inline frame, or onto the
	/// pending-inline buffer feeding the next-to-close block frame.
	fn push_inline(&mut self, node: InlineNode) {
		if let Some(frame) = self.inline_stack.last_mut() {
			match frame {
				InlineFrame::Emphasis(c)
				| InlineFrame::Strong(c)
				| InlineFrame::Strikethrough(c)
				| InlineFrame::Link { content: c, .. }
				| InlineFrame::Image { content: c, .. } => {
					c.push(node);
					return;
				}
			}
		}
		self.pending_inline.push(node);
	}

	/// Push a completed block node into its containing block frame, or onto
	/// the top-level block list.
	fn push_block(&mut self, block: BlockNode) {
		if let Some(frame) = self.block_stack.last_mut() {
			match frame {
				BlockFrame::BlockQuote { children } => {
					children.push(block);
					return;
				}
				BlockFrame::ListItem { children } => {
					children.push(block);
					return;
				}
				_ => {}
			}
		}
		self.blocks.push(block);
	}
}

fn optional_string(s: String) -> Option<String> {
	if s.is_empty() {
		None
	} else {
		Some(s)
	}
}

fn flatten_inline_text(nodes: &[InlineNode]) -> String {
	let mut out = String::new();
	for n in nodes {
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

// ---- Tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn simple_heading() {
		let doc = parse_markdown("# Hello");
		assert_eq!(
			doc,
			vec![BlockNode::Heading {
				level: 1,
				content: vec![InlineNode::Text("Hello".into())],
			}]
		);
	}

	#[test]
	fn paragraph_with_emphasis() {
		let doc = parse_markdown("This is *important* text.");
		assert_eq!(
			doc,
			vec![BlockNode::Paragraph(vec![
				InlineNode::Text("This is ".into()),
				InlineNode::Emphasis(vec![InlineNode::Text("important".into())]),
				InlineNode::Text(" text.".into()),
			])]
		);
	}

	#[test]
	fn strong_and_strike() {
		let doc = parse_markdown("**bold** and ~~struck~~");
		assert_eq!(
			doc,
			vec![BlockNode::Paragraph(vec![
				InlineNode::Strong(vec![InlineNode::Text("bold".into())]),
				InlineNode::Text(" and ".into()),
				InlineNode::Strikethrough(vec![InlineNode::Text("struck".into())]),
			])]
		);
	}

	#[test]
	fn code_block_with_language() {
		let doc = parse_markdown("```rust\nfn main() {}\n```");
		assert_eq!(
			doc,
			vec![BlockNode::CodeBlock {
				language: Some("rust".into()),
				content: "fn main() {}\n".into(),
			}]
		);
	}

	#[test]
	fn inline_code() {
		let doc = parse_markdown("Use `cargo check` to verify.");
		assert_eq!(
			doc,
			vec![BlockNode::Paragraph(vec![
				InlineNode::Text("Use ".into()),
				InlineNode::Code("cargo check".into()),
				InlineNode::Text(" to verify.".into()),
			])]
		);
	}

	#[test]
	fn unordered_list() {
		let doc = parse_markdown("- one\n- two");
		assert_eq!(
			doc,
			vec![BlockNode::List {
				ordered: false,
				items: vec![
					vec![BlockNode::Paragraph(vec![InlineNode::Text("one".into())])],
					vec![BlockNode::Paragraph(vec![InlineNode::Text("two".into())])],
				],
			}]
		);
	}

	#[test]
	fn ordered_list() {
		let doc = parse_markdown("1. first\n2. second");
		assert_eq!(
			doc,
			vec![BlockNode::List {
				ordered: true,
				items: vec![
					vec![BlockNode::Paragraph(vec![InlineNode::Text("first".into())])],
					vec![BlockNode::Paragraph(vec![InlineNode::Text(
						"second".into()
					)])],
				],
			}]
		);
	}

	#[test]
	fn blockquote() {
		let doc = parse_markdown("> quoted text");
		assert_eq!(
			doc,
			vec![BlockNode::BlockQuote(vec![BlockNode::Paragraph(vec![
				InlineNode::Text("quoted text".into())
			])])]
		);
	}

	#[test]
	fn link_with_title() {
		let doc = parse_markdown("Visit [example](https://example.com \"title\").");
		assert_eq!(
			doc,
			vec![BlockNode::Paragraph(vec![
				InlineNode::Text("Visit ".into()),
				InlineNode::Link {
					url: "https://example.com".into(),
					title: Some("title".into()),
					content: vec![InlineNode::Text("example".into())],
				},
				InlineNode::Text(".".into()),
			])]
		);
	}

	#[test]
	fn image_alt_resolved_url() {
		let doc = parse_markdown_with_resolver("![diagram](architecture.png)", |url| {
			format!("https://cdn.example/{url}")
		});
		assert_eq!(
			doc,
			vec![BlockNode::Paragraph(vec![InlineNode::Image {
				url: "https://cdn.example/architecture.png".into(),
				alt: "diagram".into(),
				title: None,
			}])]
		);
	}

	#[test]
	fn thematic_break() {
		let doc = parse_markdown("---");
		assert_eq!(doc, vec![BlockNode::ThematicBreak]);
	}

	#[test]
	fn html_is_stripped() {
		let doc = parse_markdown("<script>alert(1)</script>\n\nHello");
		// The <script> tag and contents should be entirely absent.
		let mut found_alert = false;
		fn check(blocks: &[BlockNode], found: &mut bool) {
			for b in blocks {
				if let BlockNode::Paragraph(ins) = b {
					for i in ins {
						if let InlineNode::Text(t) = i {
							if t.contains("alert") {
								*found = true;
							}
						}
					}
				}
			}
		}
		check(&doc, &mut found_alert);
		assert!(!found_alert, "raw HTML content leaked: {doc:?}");
	}

	#[test]
	fn nested_list_with_emphasis() {
		let input = "- outer one\n  - inner *emphasised*\n- outer two";
		let doc = parse_markdown(input);
		// Just verify it parses without panic and produces a list.
		assert!(matches!(doc.as_slice(), [BlockNode::List { .. }]));
	}
}
