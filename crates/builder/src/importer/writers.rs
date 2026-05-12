//! Markdown emitters for imported sections.

use super::linkedin::{
	LinkedInArticle, LinkedInCompany, LinkedInEducation, LinkedInPatent, LinkedInProfile,
	LinkedInProject,
};

// ---- profile.md -------------------------------------------------------------

pub fn render_profile(p: &LinkedInProfile) -> String {
	let mut out = String::new();
	let name = format!("{} {}", p.first_name.trim(), p.last_name.trim())
		.trim()
		.to_string();
	out.push_str(&format!("# {name}\n\n"));
	if !p.headline.is_empty() {
		out.push_str(p.headline.trim());
		out.push_str("\n\n");
	}
	let mut wrote_meta = false;
	if let Some(loc) = &p.location {
		if !wrote_meta {
			wrote_meta = true;
		}
		out.push_str(&format!("- Location: {loc}\n"));
	}
	if wrote_meta {
		out.push('\n');
	}
	if !p.summary.is_empty() {
		out.push_str("## About\n\n");
		out.push_str(p.summary.trim());
		out.push('\n');
	}
	out
}

// ---- resume/work.md ---------------------------------------------------------

pub fn render_work(companies: &[LinkedInCompany]) -> String {
	let mut out = String::from("# Work\n\n");
	for company in companies {
		out.push_str(&format!("## {}\n\n", company.name));
		for role in &company.roles {
			out.push_str(&format!("### {}\n", role.title));
			if let Some(d) = &role.start {
				out.push_str(&format!("- Start: {}\n", d.to_iso()));
			}
			match &role.end {
				Some(d) => out.push_str(&format!("- End: {}\n", d.to_iso())),
				None => out.push_str("- End: present\n"),
			}
			if let Some(loc) = &role.location {
				out.push_str(&format!("- Location: {loc}\n"));
			}
			out.push('\n');
			if !role.description.is_empty() {
				out.push_str(role.description.trim());
				out.push_str("\n\n");
			}
		}
	}
	out
}

// ---- resume/education.md ----------------------------------------------------

pub fn render_education(entries: &[LinkedInEducation]) -> String {
	let mut out = String::from("# Education\n\n");
	for e in entries {
		out.push_str(&format!("## {}\n", e.institution));
		if let Some(d) = &e.start {
			out.push_str(&format!("- Start: {}\n", d.to_iso()));
		}
		if let Some(d) = &e.end {
			out.push_str(&format!("- End: {}\n", d.to_iso()));
		}
		if let Some(q) = &e.degree {
			out.push_str(&format!("- Qualification: {q}\n"));
		}
		if let Some(f) = &e.field {
			out.push_str(&format!("- Field: {f}\n"));
		}
		out.push('\n');
		if let Some(notes) = &e.notes {
			out.push_str(notes.trim());
			out.push_str("\n\n");
		}
	}
	out
}

// ---- resume/projects.md -----------------------------------------------------

pub fn render_projects(projects: &[LinkedInProject]) -> String {
	let mut out = String::from("# Projects\n\n");
	for p in projects {
		out.push_str(&format!("## {}\n", p.title));
		if let Some(d) = &p.start {
			out.push_str(&format!("- Start: {}\n", d.to_iso()));
		}
		if let Some(d) = &p.end {
			out.push_str(&format!("- End: {}\n", d.to_iso()));
		}
		if let Some(url) = &p.url {
			out.push_str(&format!("- URL: {url}\n"));
		}
		out.push('\n');
		if !p.description.is_empty() {
			out.push_str(p.description.trim());
			out.push_str("\n\n");
		}
	}
	out
}

// ---- resume/patents.md ------------------------------------------------------

pub fn render_patents(patents: &[LinkedInPatent]) -> String {
	let mut out = String::from("# Patents\n\n");
	for p in patents {
		out.push_str(&format!("## {}\n", p.title));
		if let Some(n) = &p.number {
			out.push_str(&format!("- Number: {n}\n"));
		}
		if let Some(d) = &p.filed {
			out.push_str(&format!("- Filed: {}\n", d.to_iso()));
		}
		if let Some(d) = &p.granted {
			out.push_str(&format!("- Granted: {}\n", d.to_iso()));
		}
		if let Some(o) = &p.office {
			out.push_str(&format!("- Office: {o}\n"));
		}
		if let Some(url) = &p.url {
			out.push_str(&format!("- URL: {url}\n"));
		}
		out.push('\n');
		if !p.description.is_empty() {
			out.push_str(p.description.trim());
			out.push_str("\n\n");
		}
	}
	out
}

// ---- posts/*.md (articles) --------------------------------------------------

pub fn render_article(article: &LinkedInArticle) -> String {
	let mut out = String::new();
	out.push_str(&format!("# {}\n", article.title));
	if let Some(url) = &article.canonical_url {
		out.push_str(&format!("- Canonical: {url}\n"));
	}
	out.push('\n');
	out.push_str(article.body.trim());
	out.push('\n');
	out
}
