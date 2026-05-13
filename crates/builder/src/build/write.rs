//! Emit the build artefacts into `dist/`.

use anyhow::{Context, Result};
use std::path::Path;

use gist_id_schema::Feed;

const SIZE_CAP_BYTES: usize = 1_000_000;

pub fn write_postcard(out_dir: &Path, feed: &Feed) -> Result<()> {
	let bytes = postcard::to_allocvec(feed).context("serialising feed.postcard")?;
	if bytes.len() > SIZE_CAP_BYTES {
		anyhow::bail!(
			"feed.postcard would be {} bytes, over the {} byte cap",
			bytes.len(),
			SIZE_CAP_BYTES
		);
	}
	let path = out_dir.join("feed.postcard");
	std::fs::write(&path, &bytes).with_context(|| format!("writing {}", path.display()))?;
	tracing::info!("Wrote feed.postcard ({} bytes)", bytes.len());
	Ok(())
}

pub fn write_json_sidecar(out_dir: &Path, feed: &Feed) -> Result<()> {
	let json = serde_json::to_string_pretty(feed).context("serialising feed.json")?;
	let path = out_dir.join("feed.json");
	std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
	tracing::info!("Wrote feed.json");
	Ok(())
}

/// Emit `index.html` (noindex + redirect to gist.id) and `robots.txt`
/// (Disallow all). These prevent the GitHub Pages host from competing with
/// `gist.id/<handle>` for SEO.
pub fn write_pages_defence(out_dir: &Path, handle: &str) -> Result<()> {
	let canonical = format!("https://gist.id/{handle}");
	let index_html = format!(
		"<!doctype html>
<html lang=\"en\">
	<head>
		<meta charset=\"utf-8\">
		<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
		<meta name=\"robots\" content=\"noindex,nofollow\">
		<title>gist.id/{handle}</title>
		<link rel=\"canonical\" href=\"{canonical}\">
		<meta http-equiv=\"refresh\" content=\"0; url={canonical}\">
	</head>
	<body>
		<p>This is the source artefact for <a href=\"{canonical}\">{canonical}</a>.</p>
	</body>
</html>
"
	);
	std::fs::write(out_dir.join("index.html"), index_html)?;

	let robots = "User-agent: *\nDisallow: /\n";
	std::fs::write(out_dir.join("robots.txt"), robots)?;

	tracing::info!("Wrote Pages-defence files (index.html, robots.txt)");
	Ok(())
}
