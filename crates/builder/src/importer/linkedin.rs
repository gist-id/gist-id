//! Parsers for LinkedIn data archive CSV files.
//!
//! Column names vary across LinkedIn export versions; we tolerate
//! reasonable variations and fall back gracefully when columns are absent.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

use gist_id_schema::PartialDate;

// ---- Profile ----------------------------------------------------------------

#[derive(Debug, Default)]
pub struct LinkedInProfile {
	pub first_name: String,
	pub last_name: String,
	pub headline: String,
	pub summary: String,
	pub location: Option<String>,
}

pub fn read_profile(path: &Path) -> Result<LinkedInProfile> {
	let mut reader = csv::Reader::from_path(path)?;
	let headers = reader.headers()?.clone();

	let row = reader
		.records()
		.next()
		.context("Profile.csv has no rows")??;

	let mut p = LinkedInProfile::default();
	for (i, header) in headers.iter().enumerate() {
		let val = row.get(i).unwrap_or("").trim().to_string();
		match header {
			"First Name" => p.first_name = val,
			"Last Name" => p.last_name = val,
			"Headline" => p.headline = val,
			"Summary" => p.summary = val,
			"Geo Location" | "Location" => {
				if !val.is_empty() {
					p.location = Some(val);
				}
			}
			_ => {}
		}
	}
	Ok(p)
}

// ---- Positions → Companies + Roles -----------------------------------------

/// A company with one or more sequential roles.
#[derive(Debug)]
pub struct LinkedInCompany {
	pub name: String,
	pub roles: Vec<LinkedInRole>,
}

#[derive(Debug)]
pub struct LinkedInRole {
	pub title: String,
	pub description: String,
	pub location: Option<String>,
	pub start: Option<PartialDate>,
	pub end: Option<PartialDate>,
}

#[derive(Debug, Deserialize)]
struct PositionRow {
	#[serde(rename = "Company Name", alias = "Company")]
	company: String,
	#[serde(rename = "Title")]
	title: String,
	#[serde(rename = "Description", default)]
	description: String,
	#[serde(rename = "Location", default)]
	location: String,
	#[serde(rename = "Started On", default)]
	started_on: String,
	#[serde(rename = "Finished On", default)]
	finished_on: String,
}

/// Read positions, grouping consecutive rows with the same company name into
/// one company with multiple roles. LinkedIn already lists positions in
/// company-grouped order, so a single pass suffices. Boomerang employment
/// (non-consecutive same name) appears as two separate Company entries.
pub fn read_positions(path: &Path) -> Result<Vec<LinkedInCompany>> {
	let mut reader = csv::Reader::from_path(path)?;
	let mut out: Vec<LinkedInCompany> = Vec::new();
	for row in reader.deserialize::<PositionRow>() {
		let row = row?;
		let role = LinkedInRole {
			title: row.title.trim().into(),
			description: row.description.trim().into(),
			location: optional(row.location),
			start: parse_linkedin_date(&row.started_on),
			end: parse_linkedin_date(&row.finished_on),
		};
		let company_name = row.company.trim().to_string();

		match out.last_mut() {
			Some(last) if last.name == company_name => {
				last.roles.push(role);
			}
			_ => {
				out.push(LinkedInCompany {
					name: company_name,
					roles: vec![role],
				});
			}
		}
	}
	Ok(out)
}

// ---- Education --------------------------------------------------------------

#[derive(Debug)]
pub struct LinkedInEducation {
	pub institution: String,
	pub degree: Option<String>,
	pub field: Option<String>,
	pub notes: Option<String>,
	pub start: Option<PartialDate>,
	pub end: Option<PartialDate>,
}

#[derive(Debug, Deserialize)]
struct EducationRow {
	#[serde(rename = "School Name", alias = "School")]
	school: String,
	#[serde(rename = "Start Date", default)]
	start: String,
	#[serde(rename = "End Date", default)]
	end: String,
	#[serde(rename = "Notes", default)]
	notes: String,
	#[serde(rename = "Degree Name", alias = "Degree", default)]
	degree: String,
	#[serde(rename = "Activities", default)]
	_activities: String,
}

pub fn read_education(path: &Path) -> Result<Vec<LinkedInEducation>> {
	let mut reader = csv::Reader::from_path(path)?;
	let mut out = Vec::new();
	for row in reader.deserialize::<EducationRow>() {
		let row = row?;
		out.push(LinkedInEducation {
			institution: row.school.trim().into(),
			degree: optional(row.degree),
			field: None,
			notes: optional(row.notes),
			start: parse_linkedin_date(&row.start),
			end: parse_linkedin_date(&row.end),
		});
	}
	Ok(out)
}

// ---- Projects ---------------------------------------------------------------

#[derive(Debug)]
pub struct LinkedInProject {
	pub title: String,
	pub description: String,
	pub url: Option<String>,
	pub start: Option<PartialDate>,
	pub end: Option<PartialDate>,
}

#[derive(Debug, Deserialize)]
struct ProjectRow {
	#[serde(rename = "Title", alias = "Name")]
	title: String,
	#[serde(rename = "Description", default)]
	description: String,
	#[serde(rename = "Url", alias = "URL", default)]
	url: String,
	#[serde(rename = "Started On", default)]
	started_on: String,
	#[serde(rename = "Finished On", default)]
	finished_on: String,
}

pub fn read_projects(path: &Path) -> Result<Vec<LinkedInProject>> {
	let mut reader = csv::Reader::from_path(path)?;
	let mut out = Vec::new();
	for row in reader.deserialize::<ProjectRow>() {
		let row = row?;
		out.push(LinkedInProject {
			title: row.title.trim().into(),
			description: row.description.trim().into(),
			url: optional(row.url),
			start: parse_linkedin_date(&row.started_on),
			end: parse_linkedin_date(&row.finished_on),
		});
	}
	Ok(out)
}

// ---- Patents ----------------------------------------------------------------

#[derive(Debug)]
pub struct LinkedInPatent {
	pub title: String,
	pub number: Option<String>,
	pub url: Option<String>,
	pub description: String,
	pub filed: Option<PartialDate>,
	pub granted: Option<PartialDate>,
	pub office: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatentRow {
	#[serde(rename = "Title")]
	title: String,
	#[serde(
		rename = "Patent Number",
		alias = "Patent or Application Number",
		default
	)]
	number: String,
	#[serde(rename = "Description", default)]
	description: String,
	#[serde(rename = "Url", alias = "URL", default)]
	url: String,
	#[serde(rename = "Issuer", alias = "Patent Office", alias = "Country", default)]
	office: String,
	#[serde(rename = "Filed On", alias = "Filing Date", alias = "Date", default)]
	filed: String,
	#[serde(rename = "Issued On", alias = "Issue Date", default)]
	granted: String,
}

pub fn read_patents(path: &Path) -> Result<Vec<LinkedInPatent>> {
	let mut reader = csv::Reader::from_path(path)?;
	let mut out = Vec::new();
	for row in reader.deserialize::<PatentRow>() {
		let row = row?;
		out.push(LinkedInPatent {
			title: row.title.trim().into(),
			number: optional(row.number),
			url: optional(row.url),
			description: row.description.trim().into(),
			filed: parse_linkedin_date(&row.filed),
			granted: parse_linkedin_date(&row.granted),
			office: optional(row.office),
		});
	}
	Ok(out)
}

// ---- Articles (long-form HTML) ----------------------------------------------

#[derive(Debug)]
pub struct LinkedInArticle {
	pub date: String,
	pub slug: String,
	pub title: String,
	pub canonical_url: Option<String>,
	pub body: String,
}

pub fn read_article(path: &Path) -> Result<LinkedInArticle> {
	let html = std::fs::read_to_string(path)?;
	let (title, body_html) = extract_article_html(&html);
	let canonical_url = extract_between(&html, r#"<h1><a href=""#, r#"">"#);
	let body = html_to_markdown(&body_html);

	let date = extract_article_date(&html)
		.or_else(|| {
			let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
			split_filename_date(stem).0
		})
		.unwrap_or_else(|| "1970-01-01".to_string());

	let stem = path
		.file_stem()
		.and_then(|s| s.to_str())
		.unwrap_or("article");
	let (_, name_remainder) = split_filename_date(stem);
	let slug = slugify(name_remainder, 0);

	Ok(LinkedInArticle {
		date,
		slug,
		title,
		canonical_url,
		body,
	})
}

fn extract_article_date(html: &str) -> Option<String> {
	for marker in [
		r#"<p class="published">Published on "#,
		r#"<p class="created">Created on "#,
	] {
		if let Some(i) = html.find(marker) {
			let rest = &html[i + marker.len()..];
			if rest.len() >= 10 {
				let candidate = &rest[..10];
				if is_iso_date(candidate) {
					return Some(candidate.to_string());
				}
			}
		}
	}
	None
}

fn is_iso_date(s: &str) -> bool {
	s.len() == 10
		&& s.chars().enumerate().all(|(i, c)| match i {
			4 | 7 => c == '-',
			_ => c.is_ascii_digit(),
		})
}

fn extract_article_html(html: &str) -> (String, String) {
	let title = extract_between(html, "<title>", "</title>")
		.or_else(|| extract_between(html, "<h1>", "</h1>"))
		.unwrap_or_else(|| "Untitled".to_string());
	let mut body = extract_between(html, "<body>", "</body>").unwrap_or_else(|| html.to_string());

	body = strip_first_tag(&body, "h1");
	body = strip_first_class_paragraph(&body, "created");
	body = strip_first_class_paragraph(&body, "published");
	body = strip_first_tag(&body, "img");

	(title.trim().to_string(), body)
}

fn strip_first_tag(html: &str, tag: &str) -> String {
	let open = format!("<{tag}");
	let close = format!("</{tag}>");
	if let Some(start) = html.find(&open) {
		if let Some(after_open) = html[start..].find('>') {
			let after_open_idx = start + after_open + 1;
			if tag == "img" {
				return format!("{}{}", &html[..start], &html[after_open_idx..]);
			}
			if let Some(close_idx) = html[after_open_idx..].find(&close) {
				let after_close = after_open_idx + close_idx + close.len();
				return format!("{}{}", &html[..start], &html[after_close..]);
			}
		}
	}
	html.to_string()
}

fn strip_first_class_paragraph(html: &str, class: &str) -> String {
	let marker = format!(r#"<p class="{class}""#);
	let close = "</p>";
	if let Some(start) = html.find(&marker) {
		if let Some(close_idx) = html[start..].find(close) {
			let after_close = start + close_idx + close.len();
			return format!("{}{}", &html[..start], &html[after_close..]);
		}
	}
	html.to_string()
}

fn extract_between(s: &str, start: &str, end: &str) -> Option<String> {
	let i = s.find(start)? + start.len();
	let j = s[i..].find(end)?;
	Some(s[i..i + j].to_string())
}

fn html_to_markdown(html: &str) -> String {
	let mut out = String::new();
	let mut tag_buf = String::new();
	let mut in_tag = false;
	let mut current_tag = String::new();
	let mut href: Option<String> = None;
	let mut link_text = String::new();
	let mut in_link = false;

	for ch in html.chars() {
		match ch {
			'<' => {
				in_tag = true;
				tag_buf.clear();
			}
			'>' if in_tag => {
				in_tag = false;
				let lower = tag_buf.to_ascii_lowercase();
				let lower = lower.trim();
				if let Some(stripped) = lower.strip_prefix('/') {
					match stripped {
						"p" | "div" => out.push_str("\n\n"),
						"h1" => out.push_str("\n\n"),
						"h2" => out.push_str("\n\n"),
						"h3" => out.push_str("\n\n"),
						"li" => out.push('\n'),
						"ul" | "ol" => out.push('\n'),
						"strong" | "b" => out.push_str("**"),
						"em" | "i" => out.push('*'),
						"a" if in_link => {
							if let Some(h) = href.take() {
								out.push_str(&format!("[{link_text}]({h})"));
							} else {
								out.push_str(&link_text);
							}
							link_text.clear();
							in_link = false;
						}
						_ => {}
					}
					current_tag.clear();
				} else {
					let tag = lower.split_whitespace().next().unwrap_or("");
					current_tag = tag.to_string();
					match tag {
						"h1" => out.push_str("\n\n# "),
						"h2" => out.push_str("\n\n## "),
						"h3" => out.push_str("\n\n### "),
						"strong" | "b" => out.push_str("**"),
						"em" | "i" => out.push('*'),
						"li" => out.push_str("- "),
						"br" | "br/" => out.push('\n'),
						"a" => {
							in_link = true;
							href = extract_href(lower);
						}
						_ => {}
					}
				}
			}
			_ if in_tag => tag_buf.push(ch),
			_ if in_link => link_text.push(ch),
			_ => out.push(ch),
		}
	}

	let out = out.replace("&amp;", "&");
	let out = out.replace("&lt;", "<");
	let out = out.replace("&gt;", ">");
	let out = out.replace("&quot;", "\"");
	let out = out.replace("&#39;", "'");
	let out = out.replace("&nbsp;", " ");

	let mut collapsed = String::with_capacity(out.len());
	let mut newline_run = 0;
	for ch in out.chars() {
		if ch == '\n' {
			newline_run += 1;
			if newline_run <= 2 {
				collapsed.push(ch);
			}
		} else {
			newline_run = 0;
			collapsed.push(ch);
		}
	}

	collapsed.trim().to_string()
}

fn extract_href(tag: &str) -> Option<String> {
	let i = tag.find("href=")? + 5;
	let rest = &tag[i..];
	let rest = rest.trim_start_matches(['"', '\'']);
	let end = rest.find(['"', '\'', ' ', '>']).unwrap_or(rest.len());
	Some(rest[..end].to_string())
}

// ---- Helpers ----------------------------------------------------------------

fn optional(s: String) -> Option<String> {
	let trimmed = s.trim();
	if trimmed.is_empty() {
		None
	} else {
		Some(trimmed.to_string())
	}
}

fn parse_linkedin_date(s: &str) -> Option<PartialDate> {
	let s = s.trim();
	if s.is_empty() {
		return None;
	}

	if let Some(d) = try_iso_date(s) {
		return Some(d);
	}
	if let Some(d) = try_month_day_year(s) {
		return Some(d);
	}
	if let Some((month_name, year_str)) = split_month_year(s) {
		if let (Some(month), Ok(year)) = (parse_month(month_name), year_str.parse::<i32>()) {
			return Some(PartialDate::YearMonth { year, month });
		}
	}
	if let Ok(year) = s.parse::<i32>() {
		if (1900..3000).contains(&year) {
			return Some(PartialDate::Year(year));
		}
	}
	None
}

fn try_month_day_year(s: &str) -> Option<PartialDate> {
	let s = s.replace(',', "");
	let mut parts = s.split_whitespace();
	let month_name = parts.next()?;
	let day_str = parts.next()?;
	let year_str = parts.next()?;
	let month = parse_month(month_name)?;
	let day: u8 = day_str.parse().ok()?;
	let year: i32 = year_str.parse().ok()?;
	Some(PartialDate::YearMonthDay { year, month, day })
}

fn try_iso_date(s: &str) -> Option<PartialDate> {
	let parts: Vec<&str> = s.split('-').collect();
	match parts.as_slice() {
		[y, m, d] => {
			let year = y.parse().ok()?;
			let month = m.parse().ok()?;
			let day = d.parse().ok()?;
			Some(PartialDate::YearMonthDay { year, month, day })
		}
		[y, m] => {
			let year = y.parse().ok()?;
			let month = m.parse().ok()?;
			Some(PartialDate::YearMonth { year, month })
		}
		_ => None,
	}
}

fn split_month_year(s: &str) -> Option<(&str, &str)> {
	let mut parts = s.splitn(2, ' ');
	let a = parts.next()?;
	let b = parts.next()?.trim();
	Some((a, b))
}

fn parse_month(s: &str) -> Option<u8> {
	let s = s.to_ascii_lowercase();
	Some(match s.as_str() {
		"jan" | "january" => 1,
		"feb" | "february" => 2,
		"mar" | "march" => 3,
		"apr" | "april" => 4,
		"may" => 5,
		"jun" | "june" => 6,
		"jul" | "july" => 7,
		"aug" | "august" => 8,
		"sep" | "sept" | "september" => 9,
		"oct" | "october" => 10,
		"nov" | "november" => 11,
		"dec" | "december" => 12,
		_ => return None,
	})
}

fn split_filename_date(stem: &str) -> (Option<String>, &str) {
	if stem.len() >= 10 {
		let prefix = &stem[..10];
		if prefix.chars().enumerate().all(|(i, c)| match i {
			4 | 7 => c == '-',
			_ => c.is_ascii_digit(),
		}) {
			let rest = &stem[10..];
			let rest = rest.trim_start_matches(|c: char| !c.is_alphabetic());
			return (Some(prefix.to_string()), rest);
		}
	}
	(None, stem)
}

fn slugify(s: &str, idx: usize) -> String {
	let mut out = String::new();
	let mut last_was_dash = true;
	for ch in s.chars().take(80) {
		if ch.is_ascii_alphanumeric() {
			out.push(ch.to_ascii_lowercase());
			last_was_dash = false;
		} else if !last_was_dash {
			out.push('-');
			last_was_dash = true;
		}
	}
	let trimmed = out.trim_matches('-').to_string();
	if trimmed.is_empty() {
		format!("post-{idx}")
	} else {
		trimmed
	}
}
