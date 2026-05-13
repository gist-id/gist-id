//! Parse `resume/work.md` → `Vec<Company>`.
//!
//! Two-level hierarchy: H2 is a company, H3 is a role within it.

use anyhow::{anyhow, Result};
use gist_id_schema::{parse_markdown, BlockNode, Company, Role};

use super::meta::{extract_metadata, flatten_inline, parse_date};

pub fn parse_work(source: &str) -> Result<Vec<Company>> {
	let blocks = parse_markdown(source);
	let mut iter = blocks.into_iter().peekable();

	// Skip an optional H1 (file title).
	if matches!(iter.peek(), Some(BlockNode::Heading { level: 1, .. })) {
		iter.next();
	}

	let mut companies: Vec<Company> = Vec::new();

	// Currently-accumulating company state.
	let mut current_company_name: Option<String> = None;
	let mut current_company_blocks: Vec<BlockNode> = Vec::new();
	let mut current_company_roles: Vec<Role> = Vec::new();

	// Currently-accumulating role state (within the current company).
	let mut current_role_title: Option<String> = None;
	let mut current_role_blocks: Vec<BlockNode> = Vec::new();

	let mut state = State::Top;

	for block in iter {
		match block {
			BlockNode::Heading {
				level: 2,
				ref content,
			} => {
				// Finalise the in-progress role (if any), then the in-progress
				// company (if any), then start the new one.
				finalise_role(
					&mut current_role_title,
					&mut current_role_blocks,
					&mut current_company_roles,
				)?;
				finalise_company(
					&mut companies,
					&mut current_company_name,
					&mut current_company_blocks,
					&mut current_company_roles,
				);

				current_company_name = Some(flatten_inline(content).trim().to_string());
				state = State::Company;
			}
			BlockNode::Heading {
				level: 3,
				ref content,
			} => {
				if current_company_name.is_none() {
					return Err(anyhow!(
						"work.md: found `### {}` before any `## Company`",
						flatten_inline(content).trim()
					));
				}
				// Finalise previous role first.
				finalise_role(
					&mut current_role_title,
					&mut current_role_blocks,
					&mut current_company_roles,
				)?;
				current_role_title = Some(flatten_inline(content).trim().to_string());
				state = State::Role;
			}
			other => match state {
				State::Top => {
					// Stray block before any company heading — ignore.
				}
				State::Company => current_company_blocks.push(other),
				State::Role => current_role_blocks.push(other),
			},
		}
	}

	// Final flush.
	finalise_role(
		&mut current_role_title,
		&mut current_role_blocks,
		&mut current_company_roles,
	)?;
	finalise_company(
		&mut companies,
		&mut current_company_name,
		&mut current_company_blocks,
		&mut current_company_roles,
	);

	Ok(companies)
}

enum State {
	Top,
	Company,
	Role,
}

/// Build a Role from the accumulated state and push it onto `current_company_roles`.
fn finalise_role(
	role_title: &mut Option<String>,
	role_blocks: &mut Vec<BlockNode>,
	roles_out: &mut Vec<Role>,
) -> Result<()> {
	let Some(title) = role_title.take() else {
		role_blocks.clear();
		return Ok(());
	};
	let drained = std::mem::take(role_blocks);
	let section = extract_metadata(&drained);

	let start = section
		.meta
		.get("Start")
		.and_then(|s| parse_date(s))
		.ok_or_else(|| anyhow!("role `{title}` is missing a Start date"))?;

	let end = section.meta.get("End").and_then(|s| parse_date(s));
	let location = section.meta.get("Location").cloned();
	let employment_type = section.meta.get("Type").cloned();

	roles_out.push(Role {
		title,
		start,
		end,
		location,
		employment_type,
		description: section.body,
	});
	Ok(())
}

/// Build a Company from the accumulated state and push it onto `companies`.
fn finalise_company(
	companies: &mut Vec<Company>,
	company_name: &mut Option<String>,
	company_blocks: &mut Vec<BlockNode>,
	company_roles: &mut Vec<Role>,
) {
	let Some(name) = company_name.take() else {
		company_blocks.clear();
		company_roles.clear();
		return;
	};
	let drained = std::mem::take(company_blocks);
	let section = extract_metadata(&drained);
	let url = section.meta.get("URL").cloned();

	companies.push(Company {
		name,
		url,
		roles: std::mem::take(company_roles),
	});
}
