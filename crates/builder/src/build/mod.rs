//! `gist-id build` — read the markdown, produce dist/.

mod handle;
mod sign;
mod write;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use gist_id_schema::{Feed, Post, Profile, Signature, SCHEMA_VERSION};

use crate::parse;

pub async fn run(out_dir: &Path) -> Result<()> {
	std::fs::create_dir_all(out_dir)
		.with_context(|| format!("creating output directory {}", out_dir.display()))?;

	let handle = handle::resolve()?;
	tracing::info!("Building profile for handle: {handle}");

	let profile = read_profile()?;
	let companies = read_section("resume/work.md", parse::parse_work)?.unwrap_or_default();
	let education =
		read_section("resume/education.md", parse::parse_education)?.unwrap_or_default();
	let skills = read_section("resume/skills.md", parse::parse_skills)?.unwrap_or_default();
	let projects = read_section("resume/projects.md", parse::parse_projects)?.unwrap_or_default();
	let patents = read_section("resume/patents.md", parse::parse_patents)?.unwrap_or_default();
	let posts = read_posts()?;

	let verify_out = crate::verify::verify_skills(&handle, &skills).await?;
	let verified_skills = verify_out.verified;
	let suggested_skills = verify_out.suggested;

	let generated_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

	let mut feed = Feed {
		schema_version: SCHEMA_VERSION,
		handle: handle.clone(),
		generated_at,
		builder_version: env!("CARGO_PKG_VERSION").to_string(),
		profile,
		companies,
		education,
		skills,
		projects,
		patents,
		posts,
		verified_skills,
		suggested_skills,
		signature: Signature::empty(),
	};

	sign::sign_feed(&mut feed)?;

	write::write_postcard(out_dir, &feed)?;
	write::write_json_sidecar(out_dir, &feed)?;
	write::write_pages_defence(out_dir, &handle)?;

	tracing::info!("Built dist/ at {}", out_dir.display());
	Ok(())
}

fn read_profile() -> Result<Profile> {
	let source = std::fs::read_to_string("profile.md").context("reading profile.md")?;
	parse::parse_profile(&source)
}

fn read_section<T, F>(path: &str, parser: F) -> Result<Option<T>>
where
	F: FnOnce(&str) -> Result<T>,
{
	let p = Path::new(path);
	if !p.exists() {
		return Ok(None);
	}
	let source = std::fs::read_to_string(p).with_context(|| format!("reading {path}"))?;
	let parsed = parser(&source).with_context(|| format!("parsing {path}"))?;
	Ok(Some(parsed))
}

fn read_posts() -> Result<Vec<Post>> {
	let dir = Path::new("posts");
	if !dir.is_dir() {
		return Ok(Vec::new());
	}

	let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
		.filter_map(|e| e.ok())
		.map(|e| e.path())
		.filter(|p| p.extension().and_then(|s| s.to_str()) == Some("md"))
		.collect();
	entries.sort();

	let mut posts: Vec<Post> = Vec::new();
	for path in entries {
		let filename = path
			.file_name()
			.and_then(|s| s.to_str())
			.unwrap_or_default()
			.to_string();
		let source = std::fs::read_to_string(&path)
			.with_context(|| format!("reading {}", path.display()))?;
		let post = parse::parse_post(&filename, &source)
			.with_context(|| format!("parsing {}", path.display()))?;
		posts.push(post);
	}

	// Sort by date descending.
	posts.sort_by(|a, b| b.date.to_iso().cmp(&a.date.to_iso()));

	Ok(posts)
}
