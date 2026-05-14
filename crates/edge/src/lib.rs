//! Cloudflare Worker for rendering gist.id profiles.
//!
//! Routes:
//!   GET /<handle>  → fetch feed, deserialise, return placeholder HTML
//!   GET /         → landing page (placeholder)
//!
//! SSR rendering comes in step D.

#![forbid(unsafe_code)]

pub mod resolve;

use gist_id_schema::Feed;
use worker::{event, Context, Env, Request, Response, Result};

#[event(fetch)]
pub async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
	let url = req.url()?;
	let path = url.path().trim_start_matches('/').to_string();

	if path.is_empty() {
		return landing();
	}

	// Treat the first path segment as the handle. Anything after is reserved
	// for future use (e.g. /<handle>?post=<slug>).
	let handle = path.split('/').next().unwrap_or_default();
	handle_profile(handle).await
}

async fn handle_profile(handle: &str) -> Result<Response> {
	let feed_url = resolve::feed_url(handle);

	let mut feed_resp = match worker::Fetch::Url(feed_url.parse()?).send().await {
		Ok(r) => r,
		Err(e) => {
			return Response::error(format!("could not fetch feed: {e}"), 502);
		}
	};

	if feed_resp.status_code() == 404 {
		return Response::error(format!("no gist.id profile found for `{handle}`"), 404);
	}
	if feed_resp.status_code() != 200 {
		return Response::error(
			format!("upstream returned {}", feed_resp.status_code()),
			502,
		);
	}

	let bytes = feed_resp.bytes().await?;
	let feed: Feed = match postcard::from_bytes(&bytes) {
		Ok(f) => f,
		Err(e) => {
			return Response::error(format!("could not parse feed: {e}"), 502);
		}
	};

	// Step B placeholder render. Step D will produce real HTML via Leptos SSR.
	let body = format!(
		"<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<title>{name} — gist.id</title>
</head>
<body>
<h1>{name}</h1>
<p>{headline}</p>
<p><em>schema_version={sv}, handle={h}, generated_at={g}</em></p>
<p>(Step B placeholder. Full SSR coming in step D.)</p>
</body>
</html>
",
		name = html_escape(&feed.profile.name),
		headline = html_escape(&feed.profile.headline),
		sv = feed.schema_version,
		h = html_escape(&feed.handle),
		g = html_escape(&feed.generated_at),
	);

	Response::from_html(body)
}

fn landing() -> Result<Response> {
	Response::from_html(
		"<!doctype html>
<html lang=\"en\">
<head><meta charset=\"utf-8\"><title>gist.id</title></head>
<body>
<h1>gist.id</h1>
<p>A chain of evidence against AI slop in hiring.</p>
<p>Visit <code>gist.id/&lt;your-github-handle&gt;</code> to see a profile.</p>
</body>
</html>
",
	)
}

/// Minimal HTML escaper for the placeholder body.
fn html_escape(s: &str) -> String {
	s.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
}
