//! Render Markdown AST to HTML.
//!
//! Walks `BlockNode` / `InlineNode` trees and emits semantic HTML strings.
//! Output is escaped — no raw HTML passes through. URLs in links and images
//! are passed through as-is (callers can trust them since the markdown
//! itself is authored by the profile owner; we're not rendering untrusted
//! third-party content).

use crate::{BlockNode, InlineNode, Markdown};

/// Render a sequence of block nodes (a `Markdown` body) to HTML.
pub fn render_markdown_html(blocks: &[BlockNode]) -> String {
	let mut out = String::new();
	for block in blocks {
		render_block(&mut out, block);
	}
	out
}

fn render_block(out: &mut String, block: &BlockNode) {
	match block {
		BlockNode::Paragraph(inlines) => {
			out.push_str("<p>");
			render_inlines(out, inlines);
			out.push_str("</p>");
		}
		BlockNode::Heading { level, content } => {
			let level = (*level).clamp(1, 6);
			out.push_str(&format!("<h{level}>"));
			render_inlines(out, content);
			out.push_str(&format!("</h{level}>"));
		}
		BlockNode::List { ordered, items } => {
			let tag = if *ordered { "ol" } else { "ul" };
			out.push('<');
			out.push_str(tag);
			out.push('>');
			for item in items {
				out.push_str("<li>");
				render_list_item(out, item);
				out.push_str("</li>");
			}
			out.push_str("</");
			out.push_str(tag);
			out.push('>');
		}
		BlockNode::BlockQuote(blocks) => {
			out.push_str("<blockquote>");
			for b in blocks {
				render_block(out, b);
			}
			out.push_str("</blockquote>");
		}
		BlockNode::CodeBlock { language, content } => {
			match language {
				Some(lang) if !lang.is_empty() => {
					out.push_str("<pre><code class=\"language-");
					out.push_str(&escape_attr(lang));
					out.push_str("\">");
				}
				_ => out.push_str("<pre><code>"),
			}
			out.push_str(&escape_text(content));
			out.push_str("</code></pre>");
		}
		BlockNode::ThematicBreak => out.push_str("<hr>"),
	}
}

/// List items in our AST are themselves `Markdown` (Vec<BlockNode>). If the
/// item is a single paragraph (the common case), render its inlines without
/// the wrapping `<p>` for tighter output. Otherwise render the blocks normally.
fn render_list_item(out: &mut String, item: &Markdown) {
	if item.len() == 1 {
		if let BlockNode::Paragraph(inlines) = &item[0] {
			render_inlines(out, inlines);
			return;
		}
	}
	for b in item {
		render_block(out, b);
	}
}

fn render_inlines(out: &mut String, inlines: &[InlineNode]) {
	for n in inlines {
		render_inline(out, n);
	}
}

fn render_inline(out: &mut String, n: &InlineNode) {
	match n {
		InlineNode::Text(s) => out.push_str(&escape_text(s)),
		InlineNode::Code(s) => {
			out.push_str("<code>");
			out.push_str(&escape_text(s));
			out.push_str("</code>");
		}
		InlineNode::Emphasis(children) => {
			out.push_str("<em>");
			render_inlines(out, children);
			out.push_str("</em>");
		}
		InlineNode::Strong(children) => {
			out.push_str("<strong>");
			render_inlines(out, children);
			out.push_str("</strong>");
		}
		InlineNode::Strikethrough(children) => {
			out.push_str("<del>");
			render_inlines(out, children);
			out.push_str("</del>");
		}
		InlineNode::Link {
			url,
			title,
			content,
		} => {
			out.push_str("<a href=\"");
			out.push_str(&escape_attr(url));
			out.push('"');
			if let Some(t) = title {
				if !t.is_empty() {
					out.push_str(" title=\"");
					out.push_str(&escape_attr(t));
					out.push('"');
				}
			}
			out.push('>');
			render_inlines(out, content);
			out.push_str("</a>");
		}
		InlineNode::Image { url, alt, title } => {
			out.push_str("<img src=\"");
			out.push_str(&escape_attr(url));
			out.push_str("\" alt=\"");
			out.push_str(&escape_attr(alt));
			out.push('"');
			if let Some(t) = title {
				if !t.is_empty() {
					out.push_str(" title=\"");
					out.push_str(&escape_attr(t));
					out.push('"');
				}
			}
			out.push('>');
		}
		InlineNode::LineBreak => out.push_str("<br>"),
	}
}

fn escape_text(s: &str) -> String {
	let mut out = String::with_capacity(s.len());
	for ch in s.chars() {
		match ch {
			'&' => out.push_str("&amp;"),
			'<' => out.push_str("&lt;"),
			'>' => out.push_str("&gt;"),
			_ => out.push(ch),
		}
	}
	out
}

fn escape_attr(s: &str) -> String {
	let mut out = String::with_capacity(s.len());
	for ch in s.chars() {
		match ch {
			'&' => out.push_str("&amp;"),
			'<' => out.push_str("&lt;"),
			'>' => out.push_str("&gt;"),
			'"' => out.push_str("&quot;"),
			_ => out.push(ch),
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn paragraph() {
		let blocks = vec![BlockNode::Paragraph(vec![InlineNode::Text("Hello".into())])];
		assert_eq!(render_markdown_html(&blocks), "<p>Hello</p>");
	}

	#[test]
	fn text_escaped() {
		let blocks = vec![BlockNode::Paragraph(vec![InlineNode::Text(
			"5 < 6 & \"true\"".into(),
		)])];
		assert_eq!(
			render_markdown_html(&blocks),
			"<p>5 &lt; 6 &amp; \"true\"</p>"
		);
	}

	#[test]
	fn nested_emphasis() {
		let blocks = vec![BlockNode::Paragraph(vec![
			InlineNode::Text("This is ".into()),
			InlineNode::Strong(vec![
				InlineNode::Text("very ".into()),
				InlineNode::Emphasis(vec![InlineNode::Text("important".into())]),
			]),
			InlineNode::Text(".".into()),
		])];
		assert_eq!(
			render_markdown_html(&blocks),
			"<p>This is <strong>very <em>important</em></strong>.</p>"
		);
	}

	#[test]
	fn link_attr_escaped() {
		let blocks = vec![BlockNode::Paragraph(vec![InlineNode::Link {
			url: "https://example.com/?q=\"a&b\"".into(),
			title: None,
			content: vec![InlineNode::Text("link".into())],
		}])];
		assert_eq!(
			render_markdown_html(&blocks),
			"<p><a href=\"https://example.com/?q=&quot;a&amp;b&quot;\">link</a></p>"
		);
	}

	#[test]
	fn link_with_title() {
		let blocks = vec![BlockNode::Paragraph(vec![InlineNode::Link {
			url: "https://example.com/".into(),
			title: Some("Example site".into()),
			content: vec![InlineNode::Text("example".into())],
		}])];
		assert_eq!(
			render_markdown_html(&blocks),
			"<p><a href=\"https://example.com/\" title=\"Example site\">example</a></p>"
		);
	}

	#[test]
	fn image_with_title() {
		let blocks = vec![BlockNode::Paragraph(vec![InlineNode::Image {
			url: "/diagram.png".into(),
			alt: "Architecture".into(),
			title: Some("System overview".into()),
		}])];
		assert_eq!(
			render_markdown_html(&blocks),
			"<p><img src=\"/diagram.png\" alt=\"Architecture\" title=\"System overview\"></p>"
		);
	}

	#[test]
	fn list_tight() {
		let blocks = vec![BlockNode::List {
			ordered: false,
			items: vec![
				vec![BlockNode::Paragraph(vec![InlineNode::Text("one".into())])],
				vec![BlockNode::Paragraph(vec![InlineNode::Text("two".into())])],
			],
		}];
		assert_eq!(
			render_markdown_html(&blocks),
			"<ul><li>one</li><li>two</li></ul>"
		);
	}

	#[test]
	fn heading_clamped() {
		let blocks = vec![BlockNode::Heading {
			level: 9,
			content: vec![InlineNode::Text("Big".into())],
		}];
		assert_eq!(render_markdown_html(&blocks), "<h6>Big</h6>");
	}

	#[test]
	fn code_block_with_language() {
		let blocks = vec![BlockNode::CodeBlock {
			language: Some("rust".into()),
			content: "fn main() {}".into(),
		}];
		assert_eq!(
			render_markdown_html(&blocks),
			"<pre><code class=\"language-rust\">fn main() {}</code></pre>"
		);
	}

	#[test]
	fn code_block_text_escaped() {
		let blocks = vec![BlockNode::CodeBlock {
			language: None,
			content: "<div>".into(),
		}];
		assert_eq!(
			render_markdown_html(&blocks),
			"<pre><code>&lt;div&gt;</code></pre>"
		);
	}

	#[test]
	fn blockquote() {
		let blocks = vec![BlockNode::BlockQuote(vec![BlockNode::Paragraph(vec![
			InlineNode::Text("quoted".into()),
		])])];
		assert_eq!(
			render_markdown_html(&blocks),
			"<blockquote><p>quoted</p></blockquote>"
		);
	}
}
