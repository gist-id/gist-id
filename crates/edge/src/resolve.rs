//! Resolution: turn a gist.id handle into a feed URL.
//!
//! By convention, every gist.id profile lives in a public repository named
//! `gist-id` owned by the user. The published feed is at:
//!
//! ```text
//! https://<lowercased-handle>.github.io/gist-id/feed.postcard
//! ```
//!
//! Hostnames are case-insensitive at the DNS layer, so we lowercase to keep
//! the URL canonical. The handle's *display* casing is whatever the feed
//! itself contains in `Feed.handle` — written at build time.

/// Repository name convention.
pub const REPO_NAME: &str = "gist-id";

/// Construct the public Pages URL for `feed.postcard` for a given handle.
pub fn feed_url(handle: &str) -> String {
	format!(
		"https://{}.github.io/{REPO_NAME}/feed.postcard",
		handle.trim().to_lowercase()
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn case_normalised() {
		assert_eq!(
			feed_url("FigmentEngine"),
			"https://figmentengine.github.io/gist-id/feed.postcard"
		);
		assert_eq!(
			feed_url("figmentengine"),
			"https://figmentengine.github.io/gist-id/feed.postcard"
		);
		assert_eq!(
			feed_url("FIGMENTENGINE"),
			"https://figmentengine.github.io/gist-id/feed.postcard"
		);
	}

	#[test]
	fn whitespace_trimmed() {
		assert_eq!(
			feed_url("  fitz  "),
			"https://fitz.github.io/gist-id/feed.postcard"
		);
	}
}
