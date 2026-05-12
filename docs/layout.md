# Profile Layout

This document specifies the on-disk markdown layout that every gist.id profile
repository follows. The builder reads these files; the renderer displays them.
The convention is the contract — change it carefully.

This document describes schema version 1.

## What gist.id is

gist.id is a directory of professional identities, navigable by verified
evidence. People are found by what they've done, not by who they know.

There is no feed, no follow, no graph. Profiles are owned by the people they
describe, hosted in their own Git repositories, and built into machine-readable
artefacts the network can index.

Claims about skills and experience are paired with observable evidence —
public GitHub activity, published packages, signed artefacts. Self-attested
content (job titles, bios, posts) is rendered, but only directly-verifiable
signals contribute to discovery ranking.

## File-as-section

Each file at a known path corresponds to one section of the profile. Filenames
are the type signature; the builder reads what it finds and ignores the rest.

| Path | Section | Required |
| --- | --- | --- |
| `profile.md` | Basics (name, headline, bio, contact) | yes |
| `resume/work.md` | Work experience | recommended |
| `resume/education.md` | Education | recommended |
| `resume/skills.md` | Skills | recommended |
| `resume/projects.md` | Notable projects | optional |
| `resume/patents.md` | Patents | optional |
| `resume/certificates.md` | Certifications | optional |
| `resume/awards.md` | Awards | optional |
| `resume/publications.md` | Publications | optional |
| `resume/volunteer.md` | Volunteer work | optional |
| `resume/languages.md` | Spoken languages | optional |
| `posts/YYYY-MM-DD-slug.md` | Individual posts | optional |

Naming follows the [JSON Resume](https://jsonresume.org) taxonomy so the
builder can export to `resume.json` for ATS-friendly PDF/DOCX generation.
The directory is called `resume/` regardless of regional terminology — the
term "resume" is used here for consistency with the JSON Resume standard.

## Markdown dialect

CommonMark plus the following extensions: tables, strikethrough, task lists,
footnotes. Raw inline HTML is **not** supported and will be stripped — express
formatting through markdown features only.

Visual styling (CSS classes, inline styles, fonts, colours) is the client's
responsibility. Profiles render consistently across all gist.id clients;
authors control content, not presentation.

## Metadata blocks

Resume entries and posts carry a small dash-prefixed key-value block
immediately after their heading. This is **not** YAML front matter — it sits
inside the markdown body so GitHub's preview renders it naturally, and the
parser treats it as a leading list before the prose.

Example:

```markdown
## Staff Engineer — Hexworks
- Start: 2024-03
- End: present
- Location: Berlin, DE
- URL: https://hexworks.example

Joined to lead the platform reliability team. Designed the observability
rollout that took mean detection time from 28 minutes to under 4.

- Cut critical incident MTTR by 6x across 40 services
- Authored the internal Rust style guide adopted org-wide
```

The parser reads leading `- Key: value` lines as metadata. The first non-
metadata line marks the start of the prose body. Inline bullet lists inside
the body remain ordinary markdown lists.

Dates use ISO 8601 with optional precision: `2025`, `2025-08`, or `2025-08-15`.
The literal `present` is valid for open-ended ranges.

## profile.md

The headline file. Loose structure, parsed positionally.

```markdown
# Ada Renström

Staff engineer working on distributed systems and developer tooling.
Currently at Hexworks, previously SRE and platform roles across fintech.

- Email: ada@example.com
- Location: Berlin, DE
- URL: https://gist.id/ada
- Pronouns: she/her
- Mastodon: @ada@hachyderm.io

## About

I write Rust for a living, mostly on async runtimes and observability
pipelines. I'm interested in the boring middle layer of distributed systems
— the bits that decide whether your week is calm or on fire.

Outside work I maintain a couple of small open-source crates, run a quiet
blog on systems engineering, and grow tomatoes badly.
```

- **H1** is your name (one line, plain text)
- **First paragraph** is your headline (one or two sentences)
- **Metadata block** follows: contact, location, pronouns, etc.
- **`## About`** opens the longer bio prose

All metadata fields are optional. Provide what you want to publish.

Recognised metadata keys:

| Key | Notes |
| --- | --- |
| `Email` | Contact email |
| `Location` | Human-readable, e.g. "Berlin, DE" |
| `URL` | Canonical profile URL |
| `Pronouns` | Free text |
| `Avatar` | Path to image in repo or absolute URL |
| `LinkedIn` | LinkedIn handle |
| `Mastodon` | `@user@instance` |
| `Bluesky` | Handle |
| `Twitter` | Handle |

Unknown keys are preserved as-is and exposed to themes but not validated.

The `LinkedIn` and other external-identity keys serve a specific purpose
beyond display: the network uses them as identity links. Other gist.id
participants who have you in their local matching data (e.g. an unpublished
LinkedIn import) can detect that you've joined.

The GitHub handle is not declared in profile.md — it is derived from the
repository owner. Whoever owns the gist-id-topicked repo is the GitHub
account whose public activity is queried for verification.

## resume/work.md

One position per `##` heading. Title and company separated by an em-dash or
hyphen.

```markdown
# Work

## Staff Engineer — Hexworks
- Start: 2024-03
- End: present
- Location: Berlin, DE
- URL: https://hexworks.example

Leads the platform reliability team. Designed the observability rollout.

- Cut critical incident MTTR by 6x across 40 services
- Authored the internal Rust style guide adopted org-wide

## Senior SRE — Linwave Payments
- Start: 2020-08
- End: 2024-02
- Location: London, UK

Owned the production resilience programme for the card-processing platform.
Built the chaos engineering practice from scratch.

## Backend Engineer — Forecastable
- Start: 2017-01
- End: 2020-07
- Location: Stockholm, SE

Early engineer on a weather-data API used by agritech customers.
```

The H1 is optional and ignored; it exists so the file reads naturally on
GitHub. Each `##` is a position. Entries are sorted by start date descending
in the output regardless of file order.

Metadata keys: `Start`, `End`, `Location`, `URL`, `Type` (full-time, contract,
freelance).

## resume/education.md

Same pattern as work.

```markdown
# Education

## KTH Royal Institute of Technology
- Start: 2013
- End: 2017
- Qualification: MSc Computer Science
- Field: Distributed Systems

Thesis on consensus protocols in partially-synchronous networks.
```

Metadata keys: `Start`, `End`, `Qualification`, `Field`, `URL`, `Location`,
`Score` (GPA, honours, etc.).

## resume/skills.md

The load-bearing file. Hierarchical: `##` is a category, bullets list skills
within. Each skill may carry an optional parenthetical year or keyword set.

```markdown
# Skills

## Languages
- Rust (since 2018)
- Python (since 2012)
- Go (since 2019)
- TypeScript

## Distributed Systems
- Tokio
- Apache Kafka
- Raft (implemented)
- gRPC

## Observability
- OpenTelemetry
- Prometheus
- Grafana
- Loki
```

Categories map to JSON Resume's `skills[].name`; bullet items map to
`keywords[]`. Each claimed skill is matched by the builder's verification
pass against observable signals — GitHub language bytes, crate authorship,
public repo activity. Skills with strong evidence rank higher in network
discovery; skills with no evidence are still displayed but marked as
unverified.

## resume/projects.md

```markdown
# Projects

## raft-rs-mini
- Start: 2022-06
- End: present
- URL: https://github.com/ada-rs/raft-rs-mini
- Roles: Author, Maintainer

A teaching implementation of Raft in 1500 lines of Rust. Used as the
reference codebase in two university distributed systems courses.
```

## resume/patents.md

```markdown
# Patents

## Method and apparatus for distributed consensus under partial synchrony
- Number: US-12,345,678
- Status: Granted
- Filed: 2019-03-15
- Granted: 2022-08-09
- Office: USPTO
- URL: https://patents.google.com/patent/US12345678
- Inventors: Ada Renström, Other Person

Optional prose describing the invention's contribution.
```

Metadata keys: `Number`, `Status` (filed, pending, granted, lapsed), `Filed`,
`Granted`, `Office` (USPTO, EPO, WIPO, etc.), `URL`, `Inventors`.

Patent numbers and URLs can be cross-checked against public patent office
databases — strong evidence by design.

## resume/awards.md, certificates.md, publications.md, volunteer.md

Same pattern. Each `##` is an entry, dash-prefixed metadata block, optional
prose body.

## resume/languages.md

Spoken languages, not programming languages.

```markdown
# Languages

- Swedish (native)
- English (fluent)
- German (conversational)
```

## posts/

Each post is one file, named `YYYY-MM-DD-slug.md`. The filename carries date
and slug; no front matter needed.

```markdown
# Why I keep rewriting my Raft implementation
- Tags: rust, distributed-systems, raft

Every couple of years I throw out my pet Raft codebase and start again.
Here's what I've learned about consensus protocols from doing that.

Body markdown here.
```

- **First H1** is the post title
- **Metadata block** follows, dash-prefixed key-value pairs as elsewhere
- **Body** is everything after the metadata block

Posts are sorted by filename date descending. The slug is used for URLs:
`gist.id/<handle>?post=<slug>`.

Recognised metadata keys:

| Key | Notes |
| --- | --- |
| `Tags` | Comma-separated list, used for discovery |
| `Canonical` | Original URL if this post was first published elsewhere |

Example with the full metadata set:

```markdown
# Post title
- Tags: rust, distributed-systems
- Canonical: https://example.com/post

Body...
```

Posts are part of a profile — supplementary content showing thinking,
writing style, or technical depth. They are not feed primitives; gist.id has
no feed and no following.

## Images and assets

Place images alongside the file that references them, or in an adjacent
`images/` directory. Relative paths in markdown are rewritten by the builder
to absolute URLs pointing at the GitHub raw endpoint.

```markdown
![Architecture diagram](images/architecture.png)
```

becomes, after build:

```
https://raw.githubusercontent.com/<user>/<repo>/main/posts/images/architecture.png
```

No image resizing, format conversion, or thumbnailing happens in MVP.

## On imported content

The builder ships with an importer for LinkedIn data archives. Imported
content is treated identically to hand-authored content — the markdown
files it produces have no special status, no provenance metadata, no
attestation of having come from LinkedIn.

This is deliberate. LinkedIn data exports are CSV files the user has
already had local access to. They could have been edited at any point
between download and import. They are useful for bootstrapping a profile
quickly but they carry no third-party trust signal worth recording.

After import, the markdown is yours: edit it, correct it, expand it.
Verification of any claim — skill, project, patent — happens against
independently-checkable sources (GitHub APIs, patent offices, published
packages), not against the import provenance.

## What the builder produces

From this markdown source, the builder emits:

- `feed.postcard` — the canonical wire format (binary, signed)
- `feed.json` — debug sidecar, not authoritative
- `resume.json` — JSON Resume export for ATS / PDF / DOCX tooling
- `index.html` — Pages-defence shell (noindex, redirect to gist.id)
- `robots.txt` — disallow all crawlers on the Pages host

The Cloudflare edge renderer at `gist.id/<handle>` then embeds Schema.org
Person + WorkExperience JSON-LD in the rendered HTML, making the profile
machine-readable for search engines and LLMs without exposing the postcard
itself.

## What's deliberately excluded

The following are not part of gist.id and won't be parsed even if present:

- A feed or timeline of others' content
- A following or connections graph
- Recommendations or endorsements (third-party-authored content)
- Engagement metrics (likes, views, impressions)
- Private messages

gist.id is a directory of owned, publishable, human-authored, individually-
verifiable professional identities. If you need feeds, recommendations, or
messaging, those belong in other systems.

## Schema versioning

The builder writes `schema_version` into every artefact it produces. Major
version bumps indicate breaking layout changes that may require running
`gist-id migrate` to update your repo's structure. Minor bumps are additive
and require no action.
