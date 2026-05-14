//! Leptos SSR views.
//!
//! D.1 scope: prove the Leptos→Worker pipeline works with a minimal profile
//! header. D.2 will add the full section components (work, education,
//! skills, projects, patents, posts) once this compiles and renders.
//!
//! Rendering path: `view! { ... }` returns an `impl IntoView`. We convert to
//! a string via the `RenderHtml::to_html` trait method (Leptos 0.8 Tachys
//! engine). No reactive runtime init, no `render_to_string` ceremony, no
//! axum, no resources. Fully synchronous.

use gist_id_schema::Feed;
use leptos::prelude::*;
use worker::{Response, Result};

pub fn profile_page(feed: &Feed) -> Result<Response> {
	let name = feed.profile.name.clone();
	let headline = feed.profile.headline.clone();
	let handle = feed.handle.clone();
	let schema_version = feed.schema_version;
	let generated_at = feed.generated_at.clone();
	let title = format!("{name} — gist.id");
	let canonical = format!("https://gist.id/{handle}");

	let body_html = view! {
		<ProfileHeader name=name.clone() headline=headline.clone() />
		<p><em>
			"schema_version=" {schema_version}
			", handle=" {handle.clone()}
			", generated_at=" {generated_at.clone()}
		</em></p>
		<p>"(D.1 placeholder. Sections coming in D.2.)"</p>
	}
	.to_html();

	let html = format!(
		"<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
<title>{title}</title>
<link rel=\"canonical\" href=\"{canonical}\">
</head>
<body>
{body_html}
</body>
</html>
"
	);

	Response::from_html(html)
}

#[component]
fn ProfileHeader(name: String, headline: String) -> impl IntoView {
	view! {
		<header>
			<h1>{name}</h1>
			<p class="headline">{headline}</p>
		</header>
	}
}

pub fn landing() -> Result<Response> {
	let body_html = view! {
		<h1>"gist.id"</h1>
		<p>"A chain of evidence against AI slop in hiring."</p>
		<p>"Visit " <code>"gist.id/<your-github-handle>"</code> " to see a profile."</p>
	}
	.to_html();

	let html = format!(
		"<!doctype html>
<html lang=\"en\">
<head><meta charset=\"utf-8\"><title>gist.id</title></head>
<body>
{body_html}
</body>
</html>
"
	);
	Response::from_html(html)
}
