//! Sign a Feed with ed25519 if `GIST_ID_SIGNING_KEY` is present.
//!
//! Unsigned feeds (Signature::empty()) are valid for local preview builds.
//! Production builds in CI should always have the signing key available.

use anyhow::{anyhow, Context, Result};
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};

use gist_id_schema::{Feed, Signature};

const KEY_ENV: &str = "GIST_ID_SIGNING_KEY";

pub fn sign_feed(feed: &mut Feed) -> Result<()> {
	let Ok(key_b64) = std::env::var(KEY_ENV) else {
		tracing::warn!(
			"{KEY_ENV} not set — producing unsigned feed (preview only, not for production)"
		);
		return Ok(());
	};

	let key_bytes = base64::engine::general_purpose::STANDARD
		.decode(key_b64.trim())
		.with_context(|| format!("decoding {KEY_ENV} (must be base64)"))?;

	if key_bytes.len() != 32 {
		return Err(anyhow!(
			"{KEY_ENV} must decode to 32 bytes, got {}",
			key_bytes.len()
		));
	}

	let mut secret = [0u8; 32];
	secret.copy_from_slice(&key_bytes);
	let signing_key = SigningKey::from_bytes(&secret);
	let verifying_key = signing_key.verifying_key();

	// Sign the postcard-serialised feed with an *empty* signature in place,
	// so the verifier can reconstruct the same bytes by zeroing the
	// signature before checking.
	feed.signature = Signature::empty();
	feed.signature.public_key = verifying_key.to_bytes();
	let bytes = postcard::to_allocvec(&feed).context("serialising feed for signing")?;
	let signature = signing_key.sign(&bytes);
	feed.signature.signature = signature.to_bytes();

	tracing::info!("Feed signed with ed25519");
	Ok(())
}
