//! Leptos SSR views.
//!
//! Each section of a profile is a `#[component]` taking owned data by value.
//! Prose fields (bio, role descriptions, post bodies) are rendered to HTML
//! by `render_markdown_html` and spliced via `inner_html`.
//!
//! Rendering: `view! { ... }.to_html()` (Tachys engine, Leptos 0.8). Fully
//! synchronous; no reactive runtime, no Suspense, no resources.

use gist_id_schema::{
	render::render_markdown_html, Company, Education, Evidence, Feed, PartialDate, Patent,
	PatentStatus, Post, Profile, Project, Role, Skill, SkillCategory, VerifiedSkill,
};
use leptos::prelude::*;
use std::collections::BTreeMap;
use worker::{Response, Result};

pub fn profile_page(feed: &Feed) -> Result<Response> {
	let head_html = format!(
		"{}{}",
		crate::seo::head_meta(feed),
		crate::css::style_block(),
	);

	let verified_lookup = build_verified_lookup(&feed.verified_skills);

	let body_html = view! {
		<main class="gist-profile">
			<ProfileSection profile=feed.profile.clone() />
			<WorkSection companies=feed.companies.clone() />
			<EducationSection education=feed.education.clone() />
			<SkillsSection skills=feed.skills.clone() verified=verified_lookup.clone() />
			<ProjectsSection projects=feed.projects.clone() />
			<PatentsSection patents=feed.patents.clone() />
			<PostsSection posts=feed.posts.clone() handle=feed.handle.clone() />
			<Footer
				schema_version=feed.schema_version
				generated_at=feed.generated_at.clone()
				builder_version=feed.builder_version.clone()
			/>
			<aside class="gist-sidebar">
			  <div class="gist-sidebar-placeholder">
				 <strong>"Key facts"</strong>
				 "Verified skills, evidence summary, and quick links will appear here."
			  </div>
		   </aside>
		</main>
	}
	.to_html();

	let html = format!(
		"<!doctype html>
<html lang=\"en\">
<head>
{head_html}
</head>
<body>
{body_html}
</body>
</html>
"
	);
	Response::from_html(html)
}

pub fn landing() -> Result<Response> {
	let body_html = view! {
		<main>
			<h1>"gist.id"</h1>
			<p>"A chain of evidence against AI slop in hiring."</p>
			<p>"Visit " <code>"gist.id/<your-github-handle>"</code> " to see a profile."</p>
		</main>
	}
	.to_html();
	let html = format!(
		"<!doctype html>
<html lang=\"en\">
<head><meta charset=\"utf-8\"><title>gist.id</title></head>
<body>{body_html}</body>
</html>
"
	);
	Response::from_html(html)
}

// ── Profile section ─────────────────────────────────────────────────────

#[component]
fn ProfileSection(profile: Profile) -> impl IntoView {
	let bio_html = render_markdown_html(&profile.bio);
	view! {
		<header class="gist-profile-header">
			<h1>{profile.name}</h1>
			<p class="gist-headline">{profile.headline}</p>
			<ProfileMeta
				location=profile.location.clone()
				email=profile.email.clone()
				url=profile.url.clone()
				pronouns=profile.pronouns.clone()
			/>
		</header>
		<section class="gist-bio" inner_html=bio_html />
	}
}

#[component]
fn ProfileMeta(
	location: Option<String>,
	email: Option<String>,
	url: Option<String>,
	pronouns: Option<String>,
) -> impl IntoView {
	if location.is_none() && email.is_none() && url.is_none() && pronouns.is_none() {
		return None;
	}
	Some(view! {
		<dl class="gist-meta">
			{location.map(|v| view! { <dt>"Location"</dt> <dd>{v}</dd> })}
			{pronouns.map(|v| view! { <dt>"Pronouns"</dt> <dd>{v}</dd> })}
			{email.map(|v| view! { <dt>"Email"</dt> <dd><a href=format!("mailto:{v}")>{v.clone()}</a></dd> })}
			{url.map(|v| view! { <dt>"Web"</dt> <dd><a href=v.clone()>{v.clone()}</a></dd> })}
		</dl>
	})
}

// ── Work ───────────────────────────────────────────────────────────────

#[component]
fn WorkSection(companies: Vec<Company>) -> impl IntoView {
	if companies.is_empty() {
		return None;
	}
	Some(view! {
		<section class="gist-work">
			<h2>"Work"</h2>
			{companies.into_iter().map(|c| view! { <CompanyEntry company=c /> }).collect_view()}
		</section>
	})
}

#[component]
fn CompanyEntry(company: Company) -> impl IntoView {
	let name = company.name.clone();
	let name_node: AnyView = match company.url.clone() {
		Some(url) => view! { <a href=url>{name}</a> }.into_any(),
		None => view! { {name} }.into_any(),
	};
	view! {
		<article class="gist-work-company">
			<h3>{name_node}</h3>
			{company.roles.into_iter().map(|r| view! { <RoleEntry role=r /> }).collect_view()}
		</article>
	}
}

#[component]
fn RoleEntry(role: Role) -> impl IntoView {
	let description_html = render_markdown_html(&role.description);
	let dates = format_date_range(&Some(role.start), &role.end);
	view! {
		<section class="gist-work-role">
			<h4>{role.title}</h4>
			<p class="gist-when">
				{dates}
				{role.location.map(|l| view! { " · " {l} })}
				{role.employment_type.map(|t| view! { " · " {t} })}
			</p>
			<div class="gist-prose" inner_html=description_html />
		</section>
	}
}

// ── Education ──────────────────────────────────────────────────────────

#[component]
fn EducationSection(education: Vec<Education>) -> impl IntoView {
	if education.is_empty() {
		return None;
	}
	Some(view! {
		<section class="gist-education">
			<h2>"Education"</h2>
			{education.into_iter().map(|e| view! { <EducationEntry entry=e /> }).collect_view()}
		</section>
	})
}

#[component]
fn EducationEntry(entry: Education) -> impl IntoView {
	let description_html = render_markdown_html(&entry.description);
	let dates = format_date_range(&Some(entry.start), &entry.end);
	let institution = entry.institution.clone();
	let inst_node: AnyView = match entry.url.clone() {
		Some(url) => view! { <a href=url>{institution}</a> }.into_any(),
		None => view! { {institution} }.into_any(),
	};
	view! {
		<article class="gist-education-entry">
			<h3>{inst_node}</h3>
			<p class="gist-when">
				{dates}
				{entry.location.map(|l| view! { " · " {l} })}
			</p>
			{entry.qualification.map(|q| view! {
				<p class="gist-qualification">
					{q}
					{entry.field.clone().map(|f| view! { ", " {f} })}
				</p>
			})}
			{entry.score.map(|s| view! { <p class="gist-score">{s}</p> })}
			<div class="gist-prose" inner_html=description_html />
		</article>
	}
}

// ── Skills ─────────────────────────────────────────────────────────────

#[component]
fn SkillsSection(
	skills: Vec<SkillCategory>,
	verified: BTreeMap<String, Evidence>,
) -> impl IntoView {
	if skills.is_empty() {
		return None;
	}
	Some(view! {
		<section class="gist-skills">
			<h2>"Skills"</h2>
			{skills.into_iter().map(|c| view! {
				<SkillCategoryEntry category=c verified=verified.clone() />
			}).collect_view()}
		</section>
	})
}

#[component]
fn SkillCategoryEntry(
	category: SkillCategory,
	verified: BTreeMap<String, Evidence>,
) -> impl IntoView {
	view! {
		<section class="gist-skill-category">
			<h3>{category.name}</h3>
			<ul class="gist-skill-list">
				{category.skills.into_iter().map(|s| view! {
					<SkillItem skill=s verified=verified.clone() />
				}).collect_view()}
			</ul>
		</section>
	}
}

#[component]
fn SkillItem(skill: Skill, verified: BTreeMap<String, Evidence>) -> impl IntoView {
	let key = skill.name.trim().to_lowercase();
	let evidence_node: AnyView = match verified.get(&key) {
		Some(Evidence::GitHubLanguage { language, handle }) => {
			let url = format!(
				"https://github.com/{handle}?tab=repositories&language={}",
				language.to_lowercase()
			);
			view! {
				" "
				<a class="gist-evidence" href=url title=format!("Public {language} repos by {handle}")>
					"✓ evidence"
				</a>
			}
			.into_any()
		}
		None => view! {}.into_any(),
	};
	view! {
		<li class="gist-skill">
			<span class="gist-skill-name">{skill.name}</span>
			{skill.since.map(|y| view! {
				" " <span class="gist-skill-since">"(since " {y} ")"</span>
			})}
			{evidence_node}
		</li>
	}
}

// ── Projects ───────────────────────────────────────────────────────────

#[component]
fn ProjectsSection(projects: Vec<Project>) -> impl IntoView {
	if projects.is_empty() {
		return None;
	}
	Some(view! {
		<section class="gist-projects">
			<h2>"Projects"</h2>
			{projects.into_iter().map(|p| view! { <ProjectEntry project=p /> }).collect_view()}
		</section>
	})
}

#[component]
fn ProjectEntry(project: Project) -> impl IntoView {
	let description_html = render_markdown_html(&project.description);
	let dates = format_date_range(&project.start, &project.end);
	let name = project.name.clone();
	let name_node: AnyView = match project.url.clone() {
		Some(url) => view! { <a href=url>{name}</a> }.into_any(),
		None => view! { {name} }.into_any(),
	};
	let roles_text = if project.roles.is_empty() {
		None
	} else {
		Some(project.roles.join(", "))
	};
	view! {
		<article class="gist-project">
			<h3>{name_node}</h3>
			<p class="gist-when">
				{dates}
				{roles_text.map(|r| view! { " · " {r} })}
			</p>
			<div class="gist-prose" inner_html=description_html />
		</article>
	}
}

// ── Patents ────────────────────────────────────────────────────────────

#[component]
fn PatentsSection(patents: Vec<Patent>) -> impl IntoView {
	if patents.is_empty() {
		return None;
	}
	Some(view! {
		<section class="gist-patents">
			<h2>"Patents"</h2>
			{patents.into_iter().map(|p| view! { <PatentEntry patent=p /> }).collect_view()}
		</section>
	})
}

#[component]
fn PatentEntry(patent: Patent) -> impl IntoView {
	let description_html = render_markdown_html(&patent.description);
	let title = patent.title.clone();
	let title_node: AnyView = match patent.url.clone() {
		Some(url) => view! { <a href=url>{title}</a> }.into_any(),
		None => view! { {title} }.into_any(),
	};
	view! {
		<article class="gist-patent">
			<h3>{title_node}</h3>
			<p class="gist-patent-meta">
				{patent.number.map(|n| view! { <span>{n}</span> " · " })}
				{patent.office.map(|o| view! { <span>{o}</span> " · " })}
				{patent.status.map(|s| view! { <span>{format_status(s)}</span> })}
			</p>
			<p class="gist-when">
				{patent.filed.map(|d| view! { "Filed " {format_date(&d)} })}
				{patent.granted.map(|d| view! { " · Granted " {format_date(&d)} })}
			</p>
			<div class="gist-prose" inner_html=description_html />
		</article>
	}
}

// ── Posts ──────────────────────────────────────────────────────────────

#[component]
fn PostsSection(posts: Vec<Post>, handle: String) -> impl IntoView {
	if posts.is_empty() {
		return None;
	}
	Some(view! {
		<section class="gist-posts">
			<h2>"Posts"</h2>
			<ul class="gist-post-list">
				{posts
					.into_iter()
					.map(|p| view! { <PostListEntry post=p handle=handle.clone() /> })
					.collect_view()}
			</ul>
		</section>
	})
}

#[component]
fn PostListEntry(post: Post, handle: String) -> impl IntoView {
	let url = format!("https://gist.id/{handle}?post={}", post.slug);
	let date = format_date(&post.date);
	let tags_text = if post.tags.is_empty() {
		None
	} else {
		Some(post.tags.join(", "))
	};
	view! {
		<li class="gist-post">
			<a href=url>{post.title}</a>
			<span class="gist-post-when">" · " {date}</span>
			{tags_text.map(|t| view! { <span class="gist-post-tags">" · " {t}</span> })}
		</li>
	}
}

// ── Footer ─────────────────────────────────────────────────────────────

#[component]
fn Footer(schema_version: u16, generated_at: String, builder_version: String) -> impl IntoView {
	view! {
		<footer class="gist-footer">
			<p class="gist-build-info">
				"Built " {generated_at}
				" · schema v" {schema_version}
				" · builder " {builder_version}
			</p>
		</footer>
	}
}

// ── Date helpers ───────────────────────────────────────────────────────

fn format_date_range(start: &Option<PartialDate>, end: &Option<PartialDate>) -> String {
	let s = start.as_ref().map(format_date).unwrap_or_default();
	let e = end
		.as_ref()
		.map(format_date)
		.unwrap_or_else(|| "present".into());
	if s.is_empty() {
		e
	} else {
		format!("{s} – {e}")
	}
}

fn format_date(d: &PartialDate) -> String {
	match d {
		PartialDate::Year(y) => format!("{y}"),
		PartialDate::YearMonth { year, month } => format!("{} {year}", month_name(*month)),
		PartialDate::YearMonthDay { year, month, day } => {
			format!("{day} {} {year}", month_name(*month))
		}
	}
}

fn month_name(m: u8) -> &'static str {
	match m {
		1 => "Jan",
		2 => "Feb",
		3 => "Mar",
		4 => "Apr",
		5 => "May",
		6 => "Jun",
		7 => "Jul",
		8 => "Aug",
		9 => "Sep",
		10 => "Oct",
		11 => "Nov",
		12 => "Dec",
		_ => "",
	}
}

fn format_status(s: PatentStatus) -> &'static str {
	match s {
		PatentStatus::Filed => "Filed",
		PatentStatus::Pending => "Pending",
		PatentStatus::Granted => "Granted",
		PatentStatus::Lapsed => "Lapsed",
	}
}

fn build_verified_lookup(verified: &[VerifiedSkill]) -> BTreeMap<String, Evidence> {
	let mut out = BTreeMap::new();
	for v in verified {
		if let Some(ev) = v.evidence.first() {
			out.insert(v.name.trim().to_lowercase(), ev.clone());
		}
	}
	out
}
