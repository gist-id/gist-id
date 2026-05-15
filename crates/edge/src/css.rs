//! Inline CSS for the rendered profile page.
//!
//! Design intent: a printed-document feel, read-first, structure visible
//! through indentation and a left rule on each company block. One accent
//! colour (forest green) used for links and the evidence indicator.
//!
//! Units are relative throughout (rem, em, ch) so the reader's text-size
//! preference flows through to layout. The reading column is sized in
//! `ch` to keep line length around the 65–75 character sweet spot.

pub fn style_block() -> String {
	format!("<style>{CSS}</style>")
}

const CSS: &str = r#"
*, *::before, *::after { box-sizing: border-box; }
html, body { margin: 0; padding: 0; }

:root {
	--ink: #1a1a1a;
	--paper: #ffffff;
	--frame: #1a1a1a;
	--rule: #c9c9c9;
	--accent: #2d5016;
	--accent-strong: #1f3a0e;
	--muted: #555;
}

body {
	font-family: Charter, "Iowan Old Style", Georgia, "Times New Roman", serif;
	font-size: 1rem;
	line-height: 1.65;
	color: var(--ink);
	background: var(--paper);
	/* Black frame strips on either side of the page. */
	border-left: 0.5rem solid var(--frame);
	border-right: 0.5rem solid var(--frame);
	min-height: 100vh;
}

/* Layout: reading column + reserved sidebar gutter on the right. */
.gist-profile {
	max-width: 95ch;            /* reading (~65ch) + sidebar (~20ch) + gutter */
	margin: 0 auto;
	padding: 3rem 2rem;
	display: grid;
	grid-template-columns: minmax(0, 65ch) 20ch;
	gap: 2.5rem;
}

/* All section content goes in the left column. Sidebar placeholder lives
   in the right; for the MVP it's a static "key facts will appear here". */
.gist-profile > * { grid-column: 1; }
.gist-sidebar { grid-column: 2; grid-row: 1 / 99; }

/* Page header (name + headline + metadata). */
.gist-profile-header h1 {
	font-size: 1.75rem;
	font-weight: 600;
	margin: 0 0 0.25rem;
	line-height: 1.2;
}
.gist-headline {
	font-style: italic;
	color: var(--muted);
	margin: 0 0 1.5rem;
	font-size: 1.05rem;
}
.gist-meta {
	display: grid;
	grid-template-columns: max-content 1fr;
	gap: 0.25rem 1rem;
	margin: 0 0 2rem;
	font-size: 0.95rem;
}
.gist-meta dt {
	font-variant: small-caps;
	letter-spacing: 0.05em;
	color: var(--muted);
}
.gist-meta dd { margin: 0; }

/* Bio prose. */
.gist-bio { margin-bottom: 2.5rem; }
.gist-bio p { margin: 0 0 1em; }

/* Section headings — small-caps, separated with a thin rule. */
section > h2 {
	font-variant: small-caps;
	letter-spacing: 0.1em;
	font-size: 1rem;
	font-weight: 600;
	margin: 2.5rem 0 1.25rem;
	padding-bottom: 0.35rem;
	border-bottom: 1px solid var(--rule);
}

/* Companies: vertical left rule, indented contents. */
.gist-work-company {
	border-left: 2px solid var(--rule);
	padding-left: 1.25em;
	margin: 0 0 1.75rem;
}
.gist-work-company h3 {
	font-style: italic;
	font-weight: 500;
	font-size: 1.1rem;
	margin: 0 0 0.75rem;
}
.gist-work-company h3 a { color: var(--ink); text-decoration: none; }
.gist-work-company h3 a:hover { text-decoration: underline; }

/* Roles within a company. */
.gist-work-role {
	margin: 0 0 1.25rem;
}
.gist-work-role:last-child { margin-bottom: 0; }
.gist-work-role h4 {
	font-weight: 600;
	font-size: 1rem;
	margin: 0 0 0.15rem;
}

/* "When/where" metadata sits below the role title. */
.gist-when {
	font-style: italic;
	font-size: 0.9rem;
	color: var(--muted);
	margin: 0 0 0.5rem;
}

/* Markdown prose blocks (descriptions, bodies). */
.gist-prose p { margin: 0 0 1em; }
.gist-prose p:last-child { margin-bottom: 0; }
.gist-prose ul, .gist-prose ol { padding-left: 1.5em; margin: 0 0 1em; }
.gist-prose blockquote {
	margin: 0 0 1em;
	padding: 0 1em;
	border-left: 2px solid var(--rule);
	font-style: italic;
	color: var(--muted);
}
.gist-prose code {
	font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
	font-size: 0.9em;
}
.gist-prose pre {
	font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
	font-size: 0.9em;
	overflow-x: auto;
	margin: 0 0 1em;
}

/* Education. */
.gist-education-entry {
	margin: 0 0 1.5rem;
}
.gist-education-entry h3 {
	font-style: italic;
	font-weight: 500;
	font-size: 1.1rem;
	margin: 0 0 0.15rem;
}
.gist-qualification {
	margin: 0 0 0.5rem;
	font-size: 0.95rem;
}

/* Skills — definition-list style, category as small-caps "dt". */
.gist-skill-category {
	margin: 0 0 1.5rem;
}
.gist-skill-category h3 {
	font-variant: small-caps;
	letter-spacing: 0.08em;
	font-size: 0.9rem;
	font-weight: 600;
	color: var(--muted);
	margin: 0 0 0.5rem;
}
.gist-skill-list {
	list-style: none;
	padding: 0;
	margin: 0;
}
.gist-skill {
	margin: 0 0 0.35rem;
}
.gist-skill-name {
	font-weight: 500;
}
.gist-skill-since {
	font-style: italic;
	color: var(--muted);
	font-size: 0.9em;
}

/* Evidence link — the point of the product, made visible. */
.gist-evidence {
	color: var(--accent);
	font-weight: 600;
	text-decoration: underline;
	text-decoration-thickness: 1.5px;
	text-underline-offset: 0.15em;
	margin-left: 0.35rem;
}
.gist-evidence:hover {
	color: var(--accent-strong);
	text-decoration-thickness: 2px;
}

/* Projects + patents share visual conventions with work entries. */
.gist-project, .gist-patent {
	margin: 0 0 1.5rem;
}
.gist-project h3, .gist-patent h3 {
	font-style: italic;
	font-weight: 500;
	font-size: 1.1rem;
	margin: 0 0 0.15rem;
}
.gist-project h3 a, .gist-patent h3 a {
	color: var(--ink);
	text-decoration: underline;
	text-decoration-color: var(--rule);
	text-underline-offset: 0.15em;
}
.gist-project h3 a:hover, .gist-patent h3 a:hover {
	text-decoration-color: var(--accent);
}
.gist-patent-meta {
	font-size: 0.9rem;
	color: var(--muted);
	margin: 0 0 0.25rem;
}

/* Posts. */
.gist-post-list {
	list-style: none;
	padding: 0;
	margin: 0;
}
.gist-post {
	margin: 0 0 0.75rem;
}
.gist-post a {
	color: var(--ink);
	text-decoration: underline;
	text-decoration-color: var(--rule);
}
.gist-post a:hover { text-decoration-color: var(--accent); }
.gist-post-when, .gist-post-tags {
	color: var(--muted);
	font-style: italic;
	font-size: 0.9rem;
}

/* In-prose links (markdown body content). */
.gist-prose a {
	color: var(--accent);
	text-decoration: underline;
	text-underline-offset: 0.15em;
}
.gist-prose a:hover { color: var(--accent-strong); }

/* Footer. */
.gist-footer {
	margin-top: 4rem;
	padding-top: 1rem;
	border-top: 1px solid var(--rule);
	font-size: 0.85rem;
	color: var(--muted);
	font-style: italic;
}

/* Sidebar placeholder. Will hold "key facts" later (counts, verified
   ratio, links to canonical evidence). Today it shows intent. */

.gist-sidebar-facts {
    position: sticky;
    top: 3rem;
    padding: 1rem;
    border: 1px solid var(--rule);
    font-size: 0.85rem;
    line-height: 1.4;
}
.gist-sidebar-facts > strong {
    display: block;
    font-variant: small-caps;
    font-style: normal;
    letter-spacing: 0.08em;
    color: var(--muted);
    margin-bottom: 0.75rem;
    font-size: 0.8rem;
    font-weight: 600;
}
.gist-fact { margin: 0 0 1rem; }
.gist-fact:last-child { margin-bottom: 0; }
.gist-fact-heading {
    font-variant: small-caps;
    letter-spacing: 0.06em;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--muted);
    margin: 0 0 0.25rem;
}
.gist-fact p { margin: 0; }
.gist-fact-list {
    list-style: none;
    padding: 0;
    margin: 0;
}
.gist-fact-list li { margin: 0 0 0.15rem; }
.gist-sidebar-link {
    color: var(--accent);
    text-decoration: none;
    font-weight: 500;
}
.gist-sidebar-link:hover { text-decoration: underline; }
.gist-suggested {
    color: var(--muted);
    font-style: italic;
    font-weight: 400;
}
.gist-fact-nudge {
    margin: 0 0 0.4rem !important;
    color: var(--muted);
    font-style: italic;
}

/* nudge */
.gist-skills-nudge {
    margin: 0 0 1rem;
    color: var(--muted);
    font-style: italic;
}
.gist-skills-nudge code {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-style: normal;
    font-size: 0.9em;
}
.gist-evidence-suggested {
    color: var(--muted);
    font-style: italic;
    text-decoration: underline;
    text-decoration-color: var(--rule);
    text-underline-offset: 0.15em;
    margin-left: 0.35rem;
}
.gist-evidence-suggested:hover {
    color: var(--accent);
    text-decoration-color: var(--accent);
}

/* Mobile: collapse sidebar, soften the frame strips. */
@media (max-width: 55rem) {
	body { border-left-width: 0.25rem; border-right-width: 0.25rem; }
	.gist-profile {
		grid-template-columns: 1fr;
		padding: 2rem 1.25rem;
		gap: 1rem;
	}
	.gist-sidebar { grid-column: 1; grid-row: auto; }
	.gist-sidebar-facts { position: static; }
}
"#;
