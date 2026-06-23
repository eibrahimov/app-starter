# Contribution prompts

Use these prompts when a human or AI agent discovers something that may improve
the App Starter template. They turn real-world friction into maintainable issues
and PRs instead of vague wishlists.

## Adoption friction issue

```text
Open an App Starter adoption-friction issue.

Context:
- I was trying to build: <personal app / startup app / internal tool / desktop app / other>
- I was using: <fresh template clone / generated project>
- Template commit or generated app baseline: <sha/version if known>

Friction:
- Where I got stuck:
- What I expected to exist or be explained:
- What I tried:
- Workaround, if any:

Impact:
- Stage: <first run / add resource / typegen / frontend / deploy / desktop / upgrade>
- Would this block a new user? <yes/no/unsure>
- Suggested fix, if known:
```

## Feature or pattern proposal

```text
Open an App Starter feature/pattern proposal.

Problem:
- What does the template make hard, unsafe, repetitive, or unclear today?

Evidence:
- Where did this show up? <fresh clone / generated app / repeated project / user report>
- Why is it template-level instead of app-specific?

Proposed direction:
- Which layer changes? <migration / domain / API / typegen / frontend / desktop / CI / docs>
- How does it apply to both todo and blog?
- What can users delete if they do not need it?

Safety:
- Does this touch security, CORS, dependencies, release, migration policy, or architecture defaults?
- What simpler alternative was considered?
- What validation would prove it works?
```

## Generated-app backport issue

```text
Open an App Starter generated-app backport issue.

Generated app context:
- What kind of app was being built?
- Which template commit/version did it start from?

Problem found:
- What failed or caused repeated friction?
- Why is this not domain-specific?

Backport shape:
- Template files likely affected:
- How the fix should be demonstrated through todo/blog:
- Existing behavior that must remain unchanged:

Validation:
- Commands run in the generated app:
- Commands expected in the template PR:
```

## Change request before risky work

Use this before touching approval-required areas.

```text
Open an App Starter change request for an approval-required area.

Area:
- <security/auth/CORS/dependencies/release/migrations/generated files/architecture/license>

Current behavior:
- What does the template do now?

Proposed change:
- What default, workflow, or convention would change?

Why now:
- What evidence shows this is needed before v1 or for generated apps?

Tradeoff:
- What complexity or maintenance burden does this add?
- What future scope could this invite?
- What is the smallest safer alternative?

Approval requested:
- What decision should maintainers make before implementation begins?
```

## PR description prompt

```text
Write an App Starter PR description.

Summary:
- What problem does this solve?
- Why is it good for App Starter, not just interesting?

Template impact:
- Does this affect generated projects? How?
- Does this preserve the one-binary + OpenAPI-source-of-truth identity?
- If a default changed, when should users not use it?

Validation:
- Commands run:
- Commands skipped and why:

Risk:
- Approval-required areas touched:
- Migration/generated-file/docs impact:
- Residual risks:
```

## AI agent handoff prompt

```text
You are working in App Starter. Treat it as a high-value template whose defaults propagate into generated apps.

Task:
- <describe the fix or improvement>

Rules:
- Inspect git status before editing and do not overwrite unrelated work.
- Keep the one-binary + OpenAPI-source-of-truth identity intact.
- Do not edit committed migrations or hand-edit interface/src/api/schema.d.ts.
- Ask before changing security, CORS, dependencies, release workflow, migration
  policy, generated-file policy, or architecture conventions.
- If this came from a generated app, port the reusable pattern, not the app domain.

Validation:
- Run the relevant validation matrix commands.
- Report commands run, skipped checks with reasons, changed files, and residual risks.
```
