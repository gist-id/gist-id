//! gist-id-builder: builds a profile artefact from a markdown source repo.

mod build;
mod importer;
mod keygen;
mod parse;
mod verify;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gist-id", version, about = "Builds a gist.id profile")]
struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	/// Import an unzipped LinkedIn data archive directory into the current repo.
	Import { path: std::path::PathBuf },
	/// Build the profile: parse markdown, produce feed.postcard.
	Build {
		#[arg(short, long, default_value = "dist")]
		out: std::path::PathBuf,
	},
	/// Serve the built profile locally for preview.
	Preview {
		#[arg(short, long, default_value_t = 4000)]
		port: u16,
	},
	/// Generate an ed25519 keypair for signing feeds.
	Keygen,
}

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "gist_id=info,gist_id_builder=info".into()),
		)
		.without_time()
		.with_target(false)
		.init();

	let cli = Cli::parse();
	match cli.command {
		Command::Import { path } => importer::run(&path),
		Command::Build { out } => build::run(&out).await,
		Command::Preview { port } => {
			tracing::info!("preview stub: would serve on port {port}");
			Ok(())
		}
		Command::Keygen => keygen::run(),
	}
}
