# gist.id

**A chain of evidence against AI slop in hiring.**

The hiring market is collapsing under generated content. Candidates use LLMs
to spin tailored CVs for every application. Recruiters use LLMs to screen
the flood. Both sides are talking through layers of generated text, and the
signal-to-noise ratio has fallen off a cliff.

gist.id is a professional identity that can't be generated. Your work
history, skills, and writing live in a Git repository you own, signed with
your own key, and verified against independently-checkable sources —
public GitHub activity, published packages, patent offices. The same
canonical URL serves every recruiter, every hiring manager, every future
colleague. There's no per-application tailoring because there's nothing to
tailor.

Built for engineers first. The kind of person who commits.

## What this is, concretely

- A markdown layout your profile lives in
- A Rust builder that turns the markdown into a signed binary artefact
- A reusable GitHub Action that runs the builder on push
- An edge-rendered website at `gist.id/<handle>` that shows the profile
- A discovery layer that lets people find you by skills you can prove

No accounts. No social graph. No feed. No engagement metrics. Your URL is
yours; your data is yours; you can leave the network by removing one topic
from your repo — your data stays where it always was.

## How it works

1. You fork a template repo and edit some markdown files
2. The builder reads them, queries public sources to verify your skills,
   and produces `feed.postcard` — a signed binary artefact served from
   GitHub Pages
3. The gist.id renderer at `gist.id/<handle>` fetches your artefact and
   displays your profile with verification evidence inline
4. The discovery layer indexes everyone with the `gist-id` topic on their
   repo and lets people search by verified skill

The whole pipeline is static files, public Git, and edge functions. There's
no platform server holding your data. You can move it elsewhere by changing
where the URL points.

## What's in this repository

This is the monorepo for the gist.id product surface. It contains:

| Path | Purpose |
| --- | --- |
| `crates/schema/` | Wire format types, postcard + serde, shared contract |
| `crates/builder/` | Binary that imports archives and builds `feed.postcard` |
| `crates/indexer/` | Cloudflare Worker that builds tag indexes |
| `crates/edge/` | Cloudflare Worker that SSRs `gist.id/<handle>` |
| `crates/client/` | Leptos CSR app that hydrates the SSR'd page |
| `action/` | Reusable GitHub Action wrapping the builder binary |
| `docs/` | Protocol spec, including `layout.md` (the markdown contract) |
| `www/` | Marketing site source for `www.gist.id` |

The schema crate compiles to native (for the builder), to WASM (for the
edge worker), and to WASM again (for the browser client). Same Rust types
end-to-end, no parsing duplication.

## Status

Pre-MVP. Active development. APIs and the schema will change without
warning until version 1.0.

## Project goals

- **Evidence over claims**: every skill, project, or credential should be
  checkable against an independent source. Claims with no evidence are
  rendered but don't contribute to discovery ranking.
- **Owned, not hosted**: your profile is files in your repo, not rows in
  our database. We're a lens, not a platform.
- **Portable**: the schema is open, the protocol is documented, the
  client is one of potentially many. Forks are welcome.
- **No engagement loop**: no feed, no notifications, no metrics. People
  visit when they have a reason and leave when they're done.
- **Engineers first**: the MVP audience is technical people with public
  artefacts. Other audiences come later as the network grows.

## Non-goals

- A social network. There is no follow, no feed, no graph.
- A messaging platform.
- A jobs board.
- A CV builder for non-engineers (eventually maybe; not now).
- Privacy through obscurity. Your repo is public. What you put in it is
  public. That's the trade-off.

## Getting involved

The protocol spec lives in [`docs/layout.md`](docs/layout.md). Start there
if you want to understand what's actually being built.

Issues, discussions, and PRs welcome. The project is small and the design
is still moving — engagement before code is encouraged.

## Chain of trust

The chain of evidence is structural, not cryptographic:

1. A gist.id handle resolves to a GitHub repository via the `gist-id` topic
2. That repository's owner is, by definition, the GitHub account being claimed
   — there's no separate "claim a GitHub identity" step that could be forged
3. The owner's public GitHub activity (commits, languages, packages) is
   queryable by anyone, independently, against GitHub's own APIs
4. Patent numbers, published packages, and other external references in the
   profile can be checked against their authoritative public sources

The trust anchor is the GitHub account itself. Whoever can push to the repo
is the GitHub identity being verified.

The builder runs locally (or in the user's own GitHub Action) and queries
public sources to compute verification numbers, which it caches in the
signed `feed.postcard`. **For MVP, the renderer trusts these cached
numbers.** A user technically could fork the builder, modify the verification
output, and sign a feed with inflated claims. This would be detectable by
anyone who clicks through to the underlying GitHub source — the discrepancy
would be visible — but the system does not automatically re-verify at render
time.

Independent re-verification at render and index time is a planned
post-MVP enhancement. The schema is designed to accommodate it without
breaking changes.

What this means in practice:

- The published evidence is a starting point for your own due diligence,
  not an unforgeable cryptographic proof
- Every claim links to its source, so verification is one click away
- The system is honest about what it does and doesn't check

This trade-off is deliberate. The marketing claim is **"every claim can be
cross-checked against an authoritative public source, by anyone, in seconds"**
— not "we have unbreakable proofs." The first is true today; the second
isn't really achievable for any system whose verification involves third-
party platforms.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
