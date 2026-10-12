# Graphify integration (optional)

[Graphify](https://github.com/Graphify-Labs/graphify) turns this codebase into a
queryable knowledge graph. Coding agents (OpenCode, Claude Code, Codex, Gemini
CLI, ...) can then answer structural questions from the graph instead of
scanning every file, which keeps agent context small.

Graphify is an **optional analysis backend**. Nothing here is required to build,
test, or run the project.

## Layout

| Path | Purpose |
| - | - |
| `tools/graphify/graphify_adapter.py` | The single, isolated integration point. Shells out to the `graphify` CLI; never imports Graphify internals. |
| `.graphifyignore` | Limits what Graphify scans (build dirs, caches, vendored code, generated output). |
| `.github/workflows/graphify.yml` | Optional CI job. Manual (`workflow_dispatch`) and non-blocking by default. |
| `graphify-out/` | Generated graph + report + HTML. Git-ignored; never committed. |

## Install (per developer, opt-in)

Graphify is never required and is not wired into the build. Developers who want
the assistant integration install it themselves:

```bash
uv tool install graphifyy      # recommended
pipx install graphifyy         # alternative

# register the assistant skill for your client (this is your choice, not the repo's)
graphify install --platform opencode
graphify install --platform claude
```

Registering the skill writes assistant config (e.g. `~/.config/opencode/skills`,
`~/.claude/skills`) and/or project files such as `.opencode/`, `.claude/`,
`AGENTS.md`, `CLAUDE.md`. Those files are **intentionally not committed here** —
each developer decides whether to opt in and whether to keep them local or
version them. Add `--project` to scope the install to this working copy.

## Use

```bash
# Build the code graph (code-only: no API key / model needed)
python tools/graphify/graphify_adapter.py build --root . --out graphify-out

# Query it
python tools/graphify/graphify_adapter.py query "where is config parsed?"
python tools/graphify/graphify_adapter.py god-nodes
python tools/graphify/graphify_adapter.py explain "MyStruct"
python tools/graphify/graphify_adapter.py diagnose
```

Every subcommand prints a stable JSON object on stdout, so other tooling can
consume results without depending on Graphify's internals. If the `graphify` CLI
is not installed, the adapter reports `"available": false` and exits `0` unless
`--require` is passed.

## Keeping it optional

- Generated graphs go to `graphify-out/` and are **not** committed.
- The adapter is stdlib-only and Python-version agnostic.
- The CI job only runs when a repo opts in (`workflow_dispatch`, or by
  uncommenting the `push`/`pull_request` triggers in the workflow).
