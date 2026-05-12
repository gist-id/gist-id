//! LinkedIn archive importer.
//!
//! Reads an unzipped LinkedIn data export directory and writes markdown
//! into the current working directory, matching the layout in
//! `docs/layout.md`.

mod linkedin;
mod writers;

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const MARKER_FILE: &str = ".gist-id-template";

/// Run the importer.
pub fn run(input: &Path) -> Result<()> {
	if !Path::new(MARKER_FILE).exists() {
		bail!(
			"no `{MARKER_FILE}` found in the current directory. \
             Run this from the root of a gist.id profile repository \
             (created from the gist-id/template repo)."
		);
	}

	if input.is_file() && input.extension().and_then(|s| s.to_str()) == Some("zip") {
		bail!(
			"got a .zip file. Unzip the archive first:\n  \
             unzip {} -d linkedin-export\n  \
             gist-id import linkedin-export",
			input.display()
		);
	}

	if !input.is_dir() {
		bail!("input path {} is not a directory", input.display());
	}

	let mut summary = ImportSummary::default();

	import_profile(input, &mut summary)?;
	import_work(input, &mut summary)?;
	import_education(input, &mut summary)?;
	import_projects(input, &mut summary)?;
	import_patents(input, &mut summary)?;
	import_articles(input, &mut summary)?;
	log_skipped(input, &mut summary)?;

	summary.print();
	Ok(())
}

#[derive(Default)]
struct ImportSummary {
	profile_imported: bool,
	positions: usize,
	education: usize,
	projects: usize,
	patents: usize,
	articles: usize,
	skipped_private: Vec<String>,
	skipped_other: Vec<String>,
	skipped_existing: Vec<String>,
}

impl ImportSummary {
	fn print(&self) {
		use tracing::info;

		if self.profile_imported {
			info!("✓ Imported profile.md");
		}
		if self.positions > 0 {
			info!("✓ Imported {} work positions", self.positions);
		}
		if self.education > 0 {
			info!("✓ Imported {} education entries", self.education);
		}
		if self.projects > 0 {
			info!("✓ Imported {} projects", self.projects);
		}
		if self.patents > 0 {
			info!("✓ Imported {} patents", self.patents);
		}
		if self.articles > 0 {
			info!("✓ Imported {} long-form articles", self.articles);
		}
		for skipped in &self.skipped_private {
			info!("✗ Skipped {skipped} (private data — not imported)");
		}
		for skipped in &self.skipped_existing {
			info!("· Kept existing {skipped} (file already present)");
		}
		if !self.skipped_other.is_empty() {
			info!(
				"✗ Skipped {} other archive files (not part of the gist.id schema)",
				self.skipped_other.len()
			);
		}
	}
}

// ---- Section importers ------------------------------------------------------

fn import_profile(input: &Path, s: &mut ImportSummary) -> Result<()> {
	let profile_csv = input.join("Profile.csv");
	if !profile_csv.exists() {
		return Ok(());
	}
	let profile = linkedin::read_profile(&profile_csv).context("reading Profile.csv")?;
	let markdown = writers::render_profile(&profile);
	if write_if_changed(Path::new("profile.md"), &markdown, s, "profile.md")? {
		s.profile_imported = true;
	}
	Ok(())
}

fn import_work(input: &Path, s: &mut ImportSummary) -> Result<()> {
	let path = input.join("Positions.csv");
	if !path.exists() {
		return Ok(());
	}
	let positions = linkedin::read_positions(&path).context("reading Positions.csv")?;
	if positions.is_empty() {
		return Ok(());
	}
	let markdown = writers::render_work(&positions);
	if write_if_changed(Path::new("resume/work.md"), &markdown, s, "resume/work.md")? {
		s.positions = positions.len();
	}
	Ok(())
}

fn import_education(input: &Path, s: &mut ImportSummary) -> Result<()> {
	let path = input.join("Education.csv");
	if !path.exists() {
		return Ok(());
	}
	let entries = linkedin::read_education(&path).context("reading Education.csv")?;
	if entries.is_empty() {
		return Ok(());
	}
	let markdown = writers::render_education(&entries);
	if write_if_changed(
		Path::new("resume/education.md"),
		&markdown,
		s,
		"resume/education.md",
	)? {
		s.education = entries.len();
	}
	Ok(())
}

fn import_projects(input: &Path, s: &mut ImportSummary) -> Result<()> {
	let path = input.join("Projects.csv");
	if !path.exists() {
		return Ok(());
	}
	let projects = linkedin::read_projects(&path).context("reading Projects.csv")?;
	if projects.is_empty() {
		return Ok(());
	}
	let markdown = writers::render_projects(&projects);
	if write_if_changed(
		Path::new("resume/projects.md"),
		&markdown,
		s,
		"resume/projects.md",
	)? {
		s.projects = projects.len();
	}
	Ok(())
}

fn import_patents(input: &Path, s: &mut ImportSummary) -> Result<()> {
	let path = input.join("Patents.csv");
	if !path.exists() {
		return Ok(());
	}
	let patents = linkedin::read_patents(&path).context("reading Patents.csv")?;
	if patents.is_empty() {
		return Ok(());
	}
	let markdown = writers::render_patents(&patents);
	if write_if_changed(
		Path::new("resume/patents.md"),
		&markdown,
		s,
		"resume/patents.md",
	)? {
		s.patents = patents.len();
	}
	Ok(())
}

fn import_articles(input: &Path, s: &mut ImportSummary) -> Result<()> {
	let articles_dir = input.join("Articles").join("Articles");
	if !articles_dir.is_dir() {
		// Older exports may not have this layout — try a single level
		let alt = input.join("Articles");
		if !alt.is_dir() {
			return Ok(());
		}
		return import_articles_from(&alt, s);
	}
	import_articles_from(&articles_dir, s)
}

fn import_articles_from(dir: &Path, s: &mut ImportSummary) -> Result<()> {
	std::fs::create_dir_all("posts")?;
	let mut written = 0;
	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		if path.extension().and_then(|e| e.to_str()) != Some("html") {
			continue;
		}
		let article = linkedin::read_article(&path)
			.with_context(|| format!("reading article {}", path.display()))?;
		let filename = format!("posts/{}-{}.md", article.date, article.slug);
		let p = Path::new(&filename);
		if p.exists() {
			s.skipped_existing.push(filename);
			continue;
		}
		let markdown = writers::render_article(&article);
		std::fs::write(p, markdown)?;
		written += 1;
	}
	s.articles = written;
	Ok(())
}

// ---- Allow/deny logging -----------------------------------------------------

const PRIVATE_FILES: &[&str] = &[
	"messages.csv",
	"Connections.csv",
	"Invitations.csv",
	"Inferences_about_you.csv",
	"Ad_Targeting.csv",
	"Ads Clicked.csv",
	"PhoneNumbers.csv",
	"Email Addresses.csv",
	"Whatsapp Phone Numbers.csv",
	"Private_identity_asset.csv",
	"Logins.csv",
	"Security Challenges.csv",
	"Registration.csv",
	"SearchQueries.csv",
	"Saved_Items.csv",
	"SavedJobAlerts.csv",
	"guide_messages.csv",
	"learning_coach_messages.csv",
	"LearningCoachMessages.csv",
	"learning_role_play_messages.csv",
];

const HANDLED_FILES: &[&str] = &[
	"Profile.csv",
	"Profile Summary.csv",
	"Positions.csv",
	"Education.csv",
	"Skills.csv",
	"Projects.csv",
	"Patents.csv",
	"Shares.csv",
	"Articles",
];

fn log_skipped(input: &Path, s: &mut ImportSummary) -> Result<()> {
	for entry in walk_files(input)? {
		let name = entry
			.file_name()
			.and_then(|n| n.to_str())
			.unwrap_or_default()
			.to_string();
		if HANDLED_FILES.iter().any(|h| *h == name) {
			continue;
		}
		if PRIVATE_FILES.iter().any(|p| *p == name) {
			s.skipped_private.push(name);
		} else {
			s.skipped_other.push(name);
		}
	}
	Ok(())
}

fn walk_files(dir: &Path) -> Result<Vec<PathBuf>> {
	let mut out = Vec::new();
	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let path = entry.path();
		// Top-level only — we're categorising archive files, not recursing
		// into Articles/.
		if path.is_file() || path.is_dir() {
			out.push(path);
		}
	}
	Ok(out)
}

// ---- File-writing helpers ---------------------------------------------------

/// Write a file if it doesn't exist or still contains the template
/// placeholder content. Returns whether the write happened.
fn write_if_changed(
	path: &Path,
	new_content: &str,
	s: &mut ImportSummary,
	label: &str,
) -> Result<bool> {
	if let Some(parent) = path.parent() {
		if !parent.as_os_str().is_empty() {
			std::fs::create_dir_all(parent)?;
		}
	}

	if path.exists() {
		let existing = std::fs::read_to_string(path)?;
		if !is_template_placeholder(&existing) {
			s.skipped_existing.push(label.to_string());
			return Ok(false);
		}
	}

	std::fs::write(path, new_content)?;
	Ok(true)
}

/// Heuristic: a file is template placeholder content if it contains
/// the example name "Ada Renström" or the example company "Hexworks".
/// Good enough for MVP — users editing the template replace these.
fn is_template_placeholder(s: &str) -> bool {
	s.contains("Ada Renström")
		|| s.contains("Hexworks")
		|| s.contains("KTH Royal Institute")
		|| s.contains("raft-rs-mini")
}
