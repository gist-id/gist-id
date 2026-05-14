//! Cloudflare Worker for rendering gist.id profiles.

#![forbid(unsafe_code)]

pub mod resolve;
mod view;

use gist_id_schema::Feed;
use worker::{event, Context, Env, Request, Response, Result};

#[event(fetch)]
pub async fn fetch(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
	let url = req.url()?;
	let path = url.path().trim_start_matches('/').to_string();

	if path.is_empty() {
		return view::landing();
	}

	let handle = path.split('/').next().unwrap_or_default();
	handle_profile(handle).await
}

async fn handle_profile(handle: &str) -> Result<Response> {
	let feed_url = resolve::feed_url(handle);

	let mut feed_resp = match worker::Fetch::Url(feed_url.parse()?).send().await {
		Ok(r) => r,
		Err(e) => return Response::error(format!("could not fetch feed: {e}"), 502),
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
		Err(e) => return Response::error(format!("could not parse feed: {e}"), 502),
	};

	view::profile_page(&feed)
}
