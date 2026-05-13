//! Markdown section parsers.
//!
//! Each function takes a markdown string and produces structured schema
//! types. Parsers walk the AST from `gist-id-schema` and recognise headings,
//! metadata blocks, and prose bodies.
//!
//! Shared conventions:
//!  - `# H1` is the file's title; usually descriptive, often ignored.
//!  - `## H2` is a section entry (a company, an education entry, a project).
//!  - `### H3` is a sub-entry (a role within a company).
//!  - A dash-prefixed metadata block (`- Key: value`) follows each heading.
//!  - Prose body is whatever block content remains after the metadata block.

mod meta;
mod profile;
mod work;
mod education;
mod skills;
mod projects;
mod patents;
mod posts;

pub use education::parse_education;
pub use patents::parse_patents;
pub use posts::parse_post;
pub use profile::parse_profile;
pub use projects::parse_projects;
pub use skills::parse_skills;
pub use work::parse_work;
