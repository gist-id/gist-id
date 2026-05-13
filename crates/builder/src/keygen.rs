//! `gist-id keygen` — generate an ed25519 keypair for signing feeds.
//!
//! Prints the base64-encoded private key for the user to paste into a
//! GitHub Actions secret. The public key is shown for reference but is
//! also embedded in every signed feed at build time, so does not need to
//! be stored separately.

use anyhow::Result;
use base64::Engine;
use ed25519_dalek::SigningKey;

pub fn run() -> Result<()> {
	let mut secret_bytes = [0u8; 32];
	getrandom::getrandom(&mut secret_bytes)
		.map_err(|e| anyhow::anyhow!("failed to read OS entropy: {e}"))?;
	let signing_key = SigningKey::from_bytes(&secret_bytes);
	let verifying_key = signing_key.verifying_key();

	let secret_b64 = base64::engine::general_purpose::STANDARD.encode(signing_key.to_bytes());
	let public_b64 = base64::engine::general_purpose::STANDARD.encode(verifying_key.to_bytes());

	println!("Generated a new ed25519 keypair for signing your gist.id feed.");
	println!();
	println!("Public key  (for reference):");
	println!("    {public_b64}");
	println!();
	println!("Private key (keep secret):");
	println!("    {secret_b64}");
	println!();
	println!("Next steps:");
	println!("  1. Open your profile repo on GitHub.");
	println!("  2. Settings → Secrets and variables → Actions → New repository secret.");
	println!("  3. Name:   GIST_ID_SIGNING_KEY");
	println!("     Value:  (paste the Private key from above)");
	println!("  4. The next push will produce a signed feed.");
	println!();
	println!("Local builds without GIST_ID_SIGNING_KEY set will produce");
	println!("unsigned feeds for preview. Only CI signs production feeds.");
	Ok(())
}
