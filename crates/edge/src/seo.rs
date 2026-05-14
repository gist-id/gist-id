//! SEO and structured-data metadata for the `<head>` of a profile page.
//!
//! Emits:
//!   - title and meta description
//!   - canonical link
//!   - Open Graph (title, description, url, type=profile)
//!   - Twitter Card (summary)
//!   - Schema.org Person JSON-LD with current jobTitle and worksFor
//!
//! Returns a single string to inject between the `<head>` open and close
//! tags. We assemble JSON-LD as a string rather than via serde_json — the
//! shape is small and uniform, and avoiding the dep keeps the WASM lean.

use gist_id_schema::{Feed, SkillCategory};

pub fn head_meta(feed: &Feed) -> String {
	let title = format!("{} — gist.id", feed.profile.name);
	let description = feed.profile.headline.clone();
	let canonical = format!("https://gist.id/{}", feed.handle);

	let title_esc = html_escape(&title);
	let description_esc = attr_escape(&description);
	let canonical_esc = attr_escape(&canonical);
	let name_esc = attr_escape(&feed.profile.name);

	let mut out = String::new();
	out.push_str("<meta charset=\"utf-8\">\n");
	out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
	out.push_str("<meta name=\"robots\" content=\"index, follow\">\n");
	out.push_str(&format!("<title>{title_esc}</title>\n"));
	out.push_str(&format!(
		"<meta name=\"description\" content=\"{description_esc}\">\n"
	));
	out.push_str(&format!(
		"<link rel=\"canonical\" href=\"{canonical_esc}\">\n"
	));

	// Open Graph
	out.push_str(&format!(
		"<meta property=\"og:title\" content=\"{name_esc}\">\n"
	));
	out.push_str(&format!(
		"<meta property=\"og:description\" content=\"{description_esc}\">\n"
	));
	out.push_str(&format!(
		"<meta property=\"og:url\" content=\"{canonical_esc}\">\n"
	));
	out.push_str("<meta property=\"og:type\" content=\"profile\">\n");

	// Twitter Card
	out.push_str("<meta name=\"twitter:card\" content=\"summary\">\n");
	out.push_str(&format!(
		"<meta name=\"twitter:title\" content=\"{name_esc}\">\n"
	));
	out.push_str(&format!(
		"<meta name=\"twitter:description\" content=\"{description_esc}\">\n"
	));

	// JSON-LD Person
	out.push_str("<script type=\"application/ld+json\">");
	out.push_str(&person_json_ld(feed));
	out.push_str("</script>\n");

	out
}

fn person_json_ld(feed: &Feed) -> String {
	let mut props: Vec<String> = Vec::new();
	props.push(r#""@context":"https://schema.org""#.into());
	props.push(r#""@type":"Person""#.into());
	props.push(format!(r#""name":{}"#, json_str(&feed.profile.name)));
	props.push(format!(
		r#""url":{}"#,
		json_str(&format!("https://gist.id/{}", feed.handle))
	));
	props.push(format!(
		r#""description":{}"#,
		json_str(&feed.profile.headline)
	));

	if let Some(avatar) = &feed.profile.avatar {
		props.push(format!(r#""image":{}"#, json_str(avatar)));
	}

	// Current role: the first role of the first company (companies are listed
	// newest first, roles within a company are listed newest first).
	if let Some(first_company) = feed.companies.first() {
		if let Some(current_role) = first_company.roles.first() {
			props.push(format!(r#""jobTitle":{}"#, json_str(&current_role.title)));
			let org = current_organization(&first_company.name, first_company.url.as_deref());
			props.push(format!(r#""worksFor":{org}"#));
		}
	}

	// Flat list of skills across all categories.
	let known_about = skills_flat(&feed.skills);
	if !known_about.is_empty() {
		let items: Vec<String> = known_about.iter().map(|s| json_str(s)).collect();
		props.push(format!(r#""knowsAbout":[{}]"#, items.join(",")));
	}

	// Education as alumniOf.
	if !feed.education.is_empty() {
		let items: Vec<String> = feed
			.education
			.iter()
			.map(|e| {
				let url = e.url.as_deref();
				let mut parts: Vec<String> = vec![
					r#""@type":"EducationalOrganization""#.into(),
					format!(r#""name":{}"#, json_str(&e.institution)),
				];
				if let Some(u) = url {
					parts.push(format!(r#""url":{}"#, json_str(u)));
				}
				format!("{{{}}}", parts.join(","))
			})
			.collect();
		props.push(format!(r#""alumniOf":[{}]"#, items.join(",")));
	}

	format!("{{{}}}", props.join(","))
}

fn current_organization(name: &str, url: Option<&str>) -> String {
	let mut parts: Vec<String> = vec![
		r#""@type":"Organization""#.into(),
		format!(r#""name":{}"#, json_str(name)),
	];
	if let Some(u) = url {
		parts.push(format!(r#""url":{}"#, json_str(u)));
	}
	format!("{{{}}}", parts.join(","))
}

fn skills_flat(categories: &[SkillCategory]) -> Vec<String> {
	let mut out = Vec::new();
	for cat in categories {
		for skill in &cat.skills {
			out.push(skill.name.clone());
		}
	}
	out
}

/// Escape text for HTML content (not attributes).
fn html_escape(s: &str) -> String {
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

/// Escape for HTML attribute values.
fn attr_escape(s: &str) -> String {
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

/// Encode a string as a JSON string literal, including surrounding quotes.
/// Handles the cases that actually occur in profile data — no need for full
/// JSON spec coverage since we never embed control characters here.
fn json_str(s: &str) -> String {
	let mut out = String::with_capacity(s.len() + 2);
	out.push('"');
	for ch in s.chars() {
		match ch {
			'"' => out.push_str(r#"\""#),
			'\\' => out.push_str(r"\\"),
			'\n' => out.push_str(r"\n"),
			'\r' => out.push_str(r"\r"),
			'\t' => out.push_str(r"\t"),
			'<' => out.push_str(r"\u003c"), // safe inside <script>
			'>' => out.push_str(r"\u003e"),
			'&' => out.push_str(r"\u0026"),
			c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
			c => out.push(c),
		}
	}
	out.push('"');
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn json_str_escapes_html() {
		assert_eq!(json_str("a<b>c"), r#""a\u003cb\u003ec""#);
		assert_eq!(json_str("a&b"), r#""a\u0026b""#);
	}

	#[test]
	fn json_str_escapes_quotes_and_backslashes() {
		assert_eq!(json_str(r#"a"b\c"#), r#""a\"b\\c""#);
	}
}
