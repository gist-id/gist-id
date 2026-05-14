//! Cloudflare Worker for rendering gist.id profiles.
//!
//! Step A: resolution. Steps B–F: Worker handler, SSR, deployment.

#![forbid(unsafe_code)]

pub mod resolve;
