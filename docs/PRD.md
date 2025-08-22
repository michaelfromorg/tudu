`tudu` (working title)
Two‑way TODO ↔ Issue Tracker sync (Rust)

Goal
A fast, incremental, **opt‑in** add‑on that keeps code comments and issues in sync. Lightweight inline format:

// TODO(<ref>[, attributes…]): human-friendly comment

---

TL;DR

- Default is **validate‑only** (read‑only). You can opt into two‑way sync per comment, run, or repo.
- Works great on **existing large repos**: respects .gitignore, recognizes legacy TODO styles, never edits without `--apply`.
- **First‑class Notion** support as a tracker (read/create/update/close via status) with a minimal, explicit database mapping.
- Stable, context‑based anchors survive line shifts/renames.
- Produces **patches** by default; can apply in‑place when asked.

---

Table of contents
• Motivation & prior art
• What problem this solves
• Non‑goals
• Adoption in existing repos (opt‑in path)
• Core concepts
• Comment syntax
• How it works
• Operating modes
• Quickstart
• Configuration
• Providers (Notion‑first) & languages
• Conflict resolution
• Performance
• Security & privacy
• CLI/JSON output & CI
• Architecture (Rust)
• MVP scope & measurable success
• Roadmap
• FAQ
• Contributing & license

---

Motivation & prior art
Projects accumulate TODO comments that drift from reality. Tools like todocheck showed that scanning comments and validating linked issues reduces rot (multi‑tracker support, YAML config, CI integration). `tudu` builds on that but adds **bi‑directional sync**, **creation of issues from code**, and **safe, incremental updates** to source files.

---

What problem this solves
• Create/link issues where work happens using `TODO(...)`.
• Keep code and tracker aligned:
– Closing an issue can remove/annotate its TODO.
– Resolving a TODO can update/close the issue.
• Enforce consistency in CI without migrating all legacy comments.

---

Non‑goals
• Replacing your issue tracker or code review.
• Editing code automatically without explicit opt‑in/approval.
• Heavy, intrusive AST rewriting (we anchor conservatively; AST later).

---

Adoption in existing repos (opt‑in path)

1. **Initial scan** grandfathers existing TODOs:
   - Plain `TODO:` comments without IDs are recognized but **not tracked** (untracked)
   - Only TODOs with explicit IDs (`TODO(TASK-123):`) are actively tracked/synced
   - No automatic rewriting of existing TODOs unless explicitly requested
2. **File tasks for untracked TODOs** using `tudu file` command:
   - Interactive or batch mode to create issues for plain TODOs
   - Rewrites selected TODOs with new issue IDs
3. **Validate tracked TODOs**: surfaces dangling/closed refs for TODOs with IDs.
4. **Opt‑in sync** per comment using `bidir` attribute or per run using `tudu sync`.
5. **Patch‑first** edits: `tudu` writes unified diffs to `.tudu/patches/*.diff`; apply with `--apply` or PR bot.
6. **Incremental runs** via `.tudu/state.json` (anchors + provider ETags/modified timestamps) to keep it fast on subsequent scans.

---

Core concepts
• **Untracked vs tracked TODOs**: Plain TODOs without IDs are untracked (ignored for sync). Only TODOs with explicit issue IDs are tracked.
• **Opt‑in 2‑way sync**: default validates only. `bidir` (per comment) or config/run flag flips to two‑way.
• **Stable anchors**: `file` + rolling context hash (N lines around comment) tolerate churn/renames.
• **Idempotent**: reruns converge without surprises.
• **Local state**: `.tudu/state.json` records mappings, anchors, and provider metadata.
• **Patch‑first**: all writes are proposed as diffs unless `--apply` is set.

---

Comment syntax

Canonical pattern

TODO([, = …]): <human‑friendly comment>

`<ref>` (one of)
• **Tracker‑scoped ID (required for tracking)**

- `gh:org/repo#1234` (GitHub)
- `gl:group/proj#77` (GitLab)
- `jira:PROJ-456` (Jira)
- `nt:PAGE_ID` (arbitrary Notion page)
- `TASK-123` or `BUG-456` (Notion ID property on the linked databases)
  • **Project‑local shorthand** (uses repo config origin)
- `#1234` (GitHub/GitLab when origin is known)
- `PROJ-456` (Jira when `issue_tracker: JIRA`)
  • **Untracked** (no ref = not synced, only reported)
- Plain `TODO:` without parentheses or ID
- `TODO(person):`, also untracked

Attributes (optional)

- `bidir | one_way` — per‑comment sync direction override.
- `labels=perf,cleanup`
- `assignee=@alice`
- `due=2025-09-01`
- `close_on_delete=true|false` — removing the TODO can close/resolve the issue.
- `section="parser"` — freeform metadata; forwarded when supported.
- **Notion‑specific** (optional):
  - `db=<id>` — specify which configured database (for `tudu file` command), ids may take alias (eg "tasks" instead of uuid)
  - `status="In Progress"` — initial status when creating.
  - Property overrides: `prop.<Name>=value` (maps to a Notion property with that name).

Compatibility
• Plain `TODO:` comments without IDs are **untracked** (reported but not synced)
• Legacy `TODO <ID>:` (e.g., `TODO JIRA-123:`) recognized as tracked TODOs
• Same for TODO(person), TODO(team)
• Only TODOs with explicit IDs participate in validation/sync

Examples

```rust
// Rust
// TODO(TASK-1234, bidir, labels=parser): rewrite tokenizer for streaming
// TODO: optimize this loop  // Untracked - won't sync
```

```typescript
// TypeScript
// TODO(BUG-456): fix memory leak in event handler
// TODO(TASK-789, labels=infra): replace local queue with SQS
```

```python
# Python
# TODO(TASK-9, due=2025-10-01): account for leap seconds in scheduler
# TODO: consider caching here  // Untracked
```

```shellscript
# Shell
# TODO(BUG-88, one_way): remove legacy flag once v3 is EOL
```

// Filing new tasks for untracked TODOs
// Before: TODO: implement retry logic
// After running 'tudu file --db tasks': TODO(TASK-1290): implement retry logic

Formal grammar (simplified)

todo := "TODO" "(" ref ("," attr)\* ")" ":" text
ref := qualified | shorthand | "new" [ "=" quoted ]
qualified := ("gh:" repo "#" num) | ("gl:" path "#" num) | ("jira:" key) | ("nt:" pageid)
shorthand := some validated string (e.g., Notion autoincrement ID property value)
attr := ( "bidir" | "one_way" )
| key "=" value
key := ident | "prop." ident

⸻

How it works

1. Scan files with language‑agnostic comment matchers (//, /_…_/, #, """…""", <!--…-->), honoring .gitignore and configurable ignore globs.
2. Parse TODO(...) and legacy TODO <ID>: variants. Emit precise errors for malformed cases.
3. Anchor each TODO by (file, span, contextHash) using a rolling hash of ±N lines; resilient to inserts/moves.
4. Resolve reference:
   • Fetch the issue for known IDs.
   • Stage creation for new (title from comment or override; Notion DB from config/attr).
5. Reconcile vs. policy (mode + flags):
   • Validate only (lint): flag closed/unknown.
   • Sync: propose edits (comment text/attrs), add/remove TODOs, update tracker fields.
6. Write either:
   • Patch (.tudu/patches/\*.diff) or
   • In‑place edits when --apply is set.
7. Persist state to .tudu/state.json for future incremental runs and conflict detection.

⸻

Operating modes
• validate (default): read‑only; non‑zero exit on malformed/unknown/closed issues.
• sync: read & write to plan (patches); require --apply for file edits.
• status: list deltas (what would be created/updated/closed).

Global policy flags
• --close-on-todo-delete=ask|never|always (default: ask)
• --create-on-missing=true|false (default: false)
• --update-from=issue|comment|none (default: none) — resolve text drift.
• --bidir-default=true|false (default: false)

⸻

Quickstart

# 1) Install (Rust)

cargo install tudu

# 2) From a repo root

tudu scan

# 3) Opt in to two-way sync

tudu sync --dry-run

# 4) Apply edits after review

tudu sync --apply

(Containers & Homebrew come later; see Roadmap.)

⸻

Configuration (.tudu.yaml)

origin: github.com/org/repo
issue_tracker: NOTION # NOTION | GITHUB | GITLAB | JIRA
mode: validate # validate | sync

scan:
ignore: - target/ - node_modules/ - vendor/**
include: # optional allowlist - "**/\*"
match_case_insensitive: false

notion:

# Support for multiple databases

databases:
tasks:
database*id: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
prefix: TASK # TODOs with TASK-* IDs map here
status*property: Status
labels_property: Labels
assignee_property: Assignee
due_property: Due
done_statuses: ["Done", "Resolved"]
bugs:
database_id: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
prefix: BUG # TODOs with BUG-* IDs map here
status_property: State
labels_property: Tags
assignee_property: Owner
priority_property: Priority
done_statuses: ["Fixed", "Won't Fix", "Closed"]

# Default database for 'tudu file' when --db not specified

default_database: tasks

github:

# Optional when using GitHub as provider

owner: org
repo: repo

sync:
default_direction: one_way # one_way | bidir
create_on_missing: false
close_on_todo_delete: ask # ask | never | always
update_from: none # none | issue | comment
patch_dir: .tudu/patches

auth:
type: apitoken # none | apitoken
tokens_cache: ~/.tudu/authtokens.yaml
options:
username: alice@example.com # for Jira when needed

output:
format: standard # standard | json
verbose: false

Environment override: TUDU_AUTH_TOKEN (for CI injection).

Auto‑detect: if origin omitted and repository uses GitHub/GitLab, derive from git remote.

⸻

Providers (Notion‑first) & languages

Notion (MVP priority)
• Reference forms:
• Existing tracked TODO: TASK-123, BUG-456 (matches prefix in config)
• Untracked TODO: plain TODO: without ID
• File new issues via `tudu file` command:
• Interactively select untracked TODOs to file
• Choose target database (tasks/bugs) or use default
• Automatically rewrites TODO with new ID after creation
• Multi-database support:
• Each database has own prefix (TASK/BUG)
• Each database has own property mappings
• TODOs routed to correct DB based on prefix
• Create: title from comment text, set properties from attributes:
• labels → multi‑select (string values created if missing)
• assignee=@username → People or text (config chooses the target property type)
• due=YYYY-MM-DD → Date
• status="Todo" → Select
• prop.X=value → mapped to property "X"
• Update:
• Only for tracked TODOs with IDs
• On TODO text drift (when --update-from=comment), set page title and optional description property.
• On issue status drift, annotate or remove TODO depending on policy.
• Close:
• Mark page's Status to one of done_statuses when close_on_delete=true and TODO removed.
• Config: multiple databases with prefix mapping; property names per database.

GitHub / GitLab / Jira
• MVP: read (validate IDs, open/closed). GitHub supports create/update/close (optional for MVP; see scope).
• Shorthand #123 resolves via origin when present.

Languages
• Match comment syntaxes:
• // … and /_ … _/ (C/C++/Java/Kotlin/Swift/Rust/TS/JS/etc.)
• # … (Shell/Python/YAML/etc.)
• """ … """ (Python)
• <!-- … --> (Twig/Vue/HTML)

⸻

Conflict resolution
• Both sides edited: mark as conflicted in JSON; no writes unless --apply and --update-from chosen.
• Issue closed, TODO present: propose removing or annotating // (closed); auto‑remove only if policy says.
• TODO removed, issue open + close_on_delete=true: close issue (set Done) and add a provider comment noting the commit.
• Reopened issue: warn only; never auto‑recreate TODOs.
• Anchor lost (context not found): fall back to filename search + heuristics; if ambiguous, require manual review (no edits).

⸻

Performance
• Parallel file walking using .gitignore (crate: ignore parallel walker).
• Incremental runs using .tudu/state.json with per‑TODO hashes & provider metadata.
• Provider requests batched & rate‑limited; basic HTTP caching (If‑Modified‑Since / ETags where available).
• Bypass provider lookups for TODOs that are:
• Legacy style without IDs (unless create_on_missing=true)
• Comments excluded by config/ignore rules

Targets
• Cold run on ~100k LOC: < 60s
• Incremental (<5% deltas): < 10s

⸻

Security & privacy
• Tokens stored at ~/.tudu/authtokens.yaml (0600) by default; OS keychain when available.
• Least‑privilege scope: read for validate, read/write for sync.
• No telemetry by default.

⸻

CLI/JSON output & CI

Commands

tudu scan

# Lists all TODOs: untracked (no ID) and tracked (with ID)

# Validates tracked TODOs for unknown IDs, closed issues

# Non‑zero exit only for malformed or invalid tracked TODOs

tudu file [--db tasks|bugs] [--interactive]

# File issues for untracked TODOs:

# Interactive mode: prompts for each untracked TODO

# Batch mode: files all or filtered set

# Rewrites TODOs with new issue IDs after creation

# Example: "Filed TASK-123 for TODO in src/parser.rs:42"

tudu sync --dry-run

# Plan changes for tracked TODOs only:

# "Remove TODO for closed issue TASK-77 (will propose patch)"

# "Update title for TASK-123 based on comment change"

tudu sync --apply

# Apply edits to files and optionally post changes to the tracker

tudu link --id TASK-321 --file src/lib.rs --line 120 -- "Refactor allocator"

# Add or update a TODO with specific ID at a known location

tudu status --since 2025-08-01

# Show issue↔code drift since a date for tracked TODOs

All commands accept: --config .tudu.yaml, --basepath ., --format json, --verbose.

Standard output (human)

ERROR: Malformed TODO
src/main.ts:17: // TODO(): missing ref; expected TODO(<ref>):
ERROR: Issue doesn't exist
src/parser.rs:42: // TODO(gh:org/repo#999999): …

JSON output (machine)

[
{"type":"Malformed TODO","filename":"src/main.ts","line":17,"message":"Expected TODO(<ref>):"},
{"type":"Issue doesn't exist","filename":"src/parser.rs","line":42,"metadata":{"ref":"gh:org/repo#999999"}}
]

CI example (GitHub Actions, patch artifacts)

- name: Run tudu validate
  run: tudu scan --format json | tee tudu.json

- name: Plan sync (no writes)
  run: tudu sync --dry-run || true

- name: Upload patches
  if: always()
  uses: actions/upload-artifact@v4
  with:
  name: tudu-patches
  path: .tudu/patches/\*.diff

⸻

Architecture (Rust)

Crates
• Core: ignore, regex, serde, serde_yaml, serde_json, reqwest, clap, anyhow, thiserror, tracing, tokio
• Text/patch: similar (diffs) or diffy
• Hashing/anchors: xxhash-rust
• Git integration: git2 (origin detection; optional)
• Testing: proptest, insta (snapshots), assert_cmd, wiremock (HTTP mocks)

Modules
• scanner — parallel, ignore‑aware walker + language‑agnostic comment extractors.
• parser — TODO(...) grammar + legacy TODO <ID>:; helpful error messages.
• anchor — context hashing and re‑location heuristics.
• providers — IssueProvider trait + impls (Notion, GitHub, GitLab, Jira).
• reconciler — computes plan: Create|Update|Annotate|Remove|Close|Noop|Conflict.
• writer — unified diff generator and in‑place editor (only with --apply).
• state — serde JSON: last‑seen mappings, anchors, provider metadata.
• cli — clap subcommands; tracing logs.

Key trait

#[async_trait::async_trait]
pub trait IssueProvider {
type Id: std::fmt::Display + Clone + Eq + std::hash::Hash;
async fn fetch(&self, id: &Self::Id) -> Result<Issue, ProviderError>;
async fn create(&self, new: NewIssue) -> Result<Issue, ProviderError>;
async fn update(&self, id: &Self::Id, patch: IssuePatch) -> Result<Issue, ProviderError>;
async fn close(&self, id: &Self::Id, reason: CloseReason) -> Result<Issue, ProviderError>;
fn to_ref(&self, issue: &Issue) -> String; // e.g., "nt:PAGE_ID", "gh:org/repo#123"
fn parse_ref(&self, s: &str) -> Option<Self::Id>;
}

Core types (abridged)

pub struct Issue {
pub id: String,
pub title: String,
pub state: IssueState, // Open | Closed | Custom(String)
pub labels: Vec<String>,
pub assignees: Vec<String>,
pub due: Option<NaiveDate>,
pub url: Option<String>,
pub raw: serde_json::Value, // provider payload
}

pub struct TodoComment {
pub file: PathBuf,
pub line: u32,
pub text: String, // trailing human text
pub ref\_: Ref, // parsed ref (ID/shorthand/new)
pub attrs: BTreeMap<String, StringOrBool>,
pub anchor: Anchor,
}

pub struct Anchor {
pub context_hash: u64, // xxhash of ±N lines window
pub window: (u32, u32), // lines before/after
}

Anchoring algorithm
• Capture ±N lines (default N=3, configurable) around the TODO.
• Compute xxhash over normalized whitespace.
• On re‑scan, search nearby region first; if not found, widen window; if still not found, fallback to filename fuzzy search; otherwise mark LostAnchor.

Writer
• Generates minimal unified diffs via similar::TextDiff.
• Preserves original comment style/indentation; edits only the TODO(...) slice.
• Never touches code outside matched TODO spans.

Errors & logging
• thiserror for parser/provider errors with actionable messages.
• tracing spans per file/provider call; --verbose dumps reconciliation plans.

Testing
• Parser round‑trip tests with fuzzing via proptest.
• Golden patch tests with insta.
• Mock providers with wiremock for Notion/GitHub flows.
• Pathological repo fixtures (renames, mass inserts) for anchoring stability.

State file (.tudu/state.json) sketch

{
"version": 1,
"todos": [{
"file":"src/lib.rs",
"line":120,
"ref":"nt:abcd1234...",
"anchor":{"context_hash":"0xDEADBEEF","window":[3,3]},
"etag":"W/\"...\"",
"last_seen":"2025-08-15T12:00:00Z"
}]
}

⸻

MVP scope & measurable success

Must‑have (v0.1)
• Parse/validate TODO(...) + legacy TODO <ID>:
• Providers:
• Notion: read + create + update + close (status) with configurable DB & property mapping
• GitHub: read (validate IDs); optional create/update behind feature flag
• GitLab, Jira: read (validate IDs)
• Modes: validate, sync --dry-run, sync --apply with patch generation
• .gitignore support; config file; env token; token cache
• JSON output for CI; non‑zero exit on problems

Success criteria
• Cold run < 60s on ~100k LOC; Incremental < 10s
• ≥ 95% stable anchoring across rename/insert diffs
• Zero silent destructive edits; all writes gated behind --apply

⸻

Roadmap
• Providers: full parity (GitHub/GitLab/Jira write), YouTrack, Azure Boards, Redmine, Pivotal
• IDE plugin (VS Code/JetBrains) to insert TODO(...) & preview sync
• PR bot mode: open PRs with patches vs. direct commits
• AST‑aware anchoring (optional tree-sitter)
• Packages: Homebrew, container images, Windows builds
• Repo health dashboards (e.g., “TODOs by label/age”)
• Migration command to tag legacy TODO: lines interactively

⸻

FAQ
Is bi‑directional sync forced?
No. Default is read‑only validation; opt in per comment/run/repo.

Will it rewrite my files automatically?
Never without --apply. In CI, default is to emit patches (or PR via bot later).

Can I keep my existing TODO <ID>: style?
Yes—validated only. Add parentheses when you want metadata or two‑way.

What happens when an issue closes?
Policy‑driven: warn, annotate, or remove the TODO with a patch.

⸻

Contributing & license
Contributions welcome: new providers, matchers, docs, tests.
License: MIT (see /LICENSE once repo is initialized).
