# Issue tracker: GitHub

Issues and specs live in GitHub Issues for `mathiasringhof/lml`.
Use the `gh` CLI from this clone; it infers the repository from origin.

## Conventions

- Create: `gh issue create --title "..." --body-file <path>`.
- Read: `gh issue view <number> --comments`. Fetch labels and structured
  fields with `--json` when needed.
- List: `gh issue list --state open --json number,title,body,labels,comments`.
  Apply label and state filters as needed.
- Comment: `gh issue comment <number> --body-file <path>`.
- Label: `gh issue edit <number> --add-label "..."` or
  `--remove-label "..."`.
- Close: `gh issue close <number> --comment "..."`.

For multiline bodies, write the exact Markdown to a temporary file and
pass `--body-file`.

“Publish to the issue tracker” means create a GitHub issue.
“Fetch the relevant ticket” means read the issue and its comments.

## Pull requests as a triage surface

**PRs as a request surface: no.**

Issues and PRs share a number space. When a reference is ambiguous,
resolve with `gh pr view <number>`, falling back to `gh issue view <number>`.

## Wayfinding operations

- Map: one issue labelled `wayfinder:map`, containing Notes,
  Decisions-so-far, and Fog.
- Child tickets: link them as GitHub sub-issues using `gh api`.
  If unavailable, use a task list in the map and put `Part of #<map>`
  at the top of each child.
- Ticket labels: `wayfinder:research`, `wayfinder:prototype`,
  `wayfinder:grilling`, or `wayfinder:task`.
- Blocking: use native issue dependencies. Add an edge with
  `gh api --method POST repos/mathiasringhof/lml/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>`.
  Obtain the database ID with
  `gh api repos/mathiasringhof/lml/issues/<blocker> --jq .id`.
  If dependencies are unavailable, use a `Blocked by: #<n>, #<n>` line.
- Frontier: select open children without an assignee or open blockers;
  first in map order wins. Native open blockers are reported by
  `issue_dependencies_summary.blocked_by`.
- Claim: `gh issue edit <number> --add-assignee @me`.
- Resolve: comment with the answer, close the child, then add a brief
  finding and link to the map's Decisions-so-far.
