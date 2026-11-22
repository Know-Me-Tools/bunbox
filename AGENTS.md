<!-- prometheus-team-routing:start v1 -->
For every code task, read `.agent-team/project-routing.json`, then its active team manifest and the relevant role instructions. Default to that team, selecting only roles whose responsibilities and ownership match the work. Preserve native permissions, models, concurrency limits and existing project instructions.
For UI work, load the role-bound `prometheus-ui-ux` or `prometheus-ui-review` skill. Prefer `.agents/UI_UX_PROTOCOL.md` when present; otherwise use the installed `prometheus-ui-ux/references/UI_UX_PROTOCOL.md`. Backend work must not load UI guidance.
Use native delegation when available. If unavailable, follow the selected role instructions sequentially and report that limitation. Review in the builder context is not independent review. Keep reviewers dormant until the complete implementation phase; allow one batched correction/confirmation cycle. Respect user-only skill invocation restrictions. Zed external ACP agents use their own native configuration; parallel UI threads are not an automatic delegation API.
Cross-team requests: when work needs another team, run `team-discover` (capabilities, paths) and `team-request`; a same-repo team gets an intake task, any other team gets a `team:<id>` GitHub issue. If this team has a card, run `team-intake` to import open `team:<id>` issues as tasks for its intake role, and triage them before other work.
<!-- prometheus-team-routing:end -->

<!-- prometheus-base:start v1 -->
# Agent Operating Rules

## User goals, value, and business rules

Always think in terms of user goals and value—not in tests or assumptions.
State the outcome/value and exact examples first. Before solutions, queries or
tests, record rules (conditions, outcome, evidence, exceptions, unknowns) in the
accepted spec/behavior contract. Trace phase-end BDD to meaningful reachable
outcomes. Tests verify value, not define it; preserve existing gates/processes.

Managed by `prometheus-context-bootstrap`; edits inside these markers are
overwritten. A hook that enforces a rule here wins.

**First action, every session (hooks or not):** read canonical KBD status and the
active phase's `method-context.md`; state the phase and the boundary rule below;
only then act. Never rebuild a retired method from an old plan, handoff or summary.

## Position and authority

- Signed canonical KBD status/decisions and the adopted phase contract are authoritative; `.kbd-orchestrator/current-waypoint.json` is their derived position view.
- `versions.toml` is authoritative for architecture decisions and dependency pins.
- READMEs go stale. Do not trust one over the two files above.
- `method-context.md` is advisory: resolve canonical identity, supersession, included value, exclusions, stage authority, evidence freshness, consumed budgets and unresolved effects. State missing references; never guess them.

## Capability inversion

Agent kernels do not write. Mutating actions are gated only in the trusted
host layer, enforced at the dependency graph where the language allows. If a
task appears to need a kernel write, stop and surface the conflict.

## Phase order

Task loop: Spec, Plan, Execute, Reflect.
Evolution loop: Compile, Evaluate, Optimize, Promote.

Name the phase you are in; never execute before a plan exists.

## Verification boundaries (VLBV)

<!-- kbd-policy:begin -->
- verification: VLBV
- compiler-exception: retired
- execution-boundary: B4 once; B5 after an authorized repair
<!-- kbd-policy:end -->

Adopt the phase's acceptance scenarios before generation. Write the whole agreed
increment without executing generated code: no tests, builds, compiler checks,
imports, lint, typecheck, hooks or probes (editor/LSP diagnostics are reading).
At the value-phase boundary (B4) run the adopted boundary packet once; after a
FAIL only an owner-authorized repair (B5) reruns the affected scenarios. Tasks,
commits, reviews and compiler problems are not boundaries. Only full integration
evidence counts. Verdicts: PASS, FAIL, BLOCKED, NOT_EXERCISED. A deviation is a
signed `KBD-METHOD` decision. This method outranks built-in TDD, verify or apply
workflow skills. OpenSpec skills (`openspec-*`) are generated: never edit them.
Per-stack commands live in `.claude/rules/`.

- `.claude/rules/rust.md` — rust tiers and hard rules

## Evidentiary standard

Address observed problems: an operator report, a visible error or log, a
failing test, or an explicit requirement. Anything else gets one sentence and a
question, never speculative code. Defensive code (validation, guards, fallbacks,
retries, timeouts) needs a named failure scenario; hardening at a real trust
boundary is the exception, and is named in the completion summary.

## Evidence over assertion

Show the command and output, test result or artifact, and the boundary it ran
at. If a check could not run, say which claims are unverified. An unverified
claim reported as verified is worse than no check.

## Anti-sycophancy

Review through the `artifact-critic` subagent, which sees the artifact but not
its generation history; the producer is never the sole judge. Reflections lead
with the plan-to-delivery delta. If the sycophancy gate blocks a turn, fix the
finding.

## Learning and memory

Learning is append-only under `.prometheus/` (`session-log.md`, `decisions.md`,
`gotchas.md`, `postmortems/`, `knowledge/`); mark superseded entries, never
delete. Record decisions with rationale, defects with root cause, learned
constraints, phase boundaries and session summaries. Read `gotchas.md` before
touching a subsystem.

A configured memory server is the primary store; if its write fails, log to
the files above and continue. Never block a task on it.

## Architecture

- Single-writer build discipline within one build or target directory.
- Feature-based organization by capability, not by technical layer.
- Strict layering: UI, then hooks or view models, then stores, then services,
  then external. Reverse flow only through reactive state or events.
- Business state lives in explicit, inspectable systems, never in UI components
  or agent-only memory.
- Open standards first. Avoid lock-in unless explicitly required.
- Verify dependency versions against official sources before introducing them.
  Do not rely on training-era version knowledge.

## Scope

Make the minimum change. Treat adjacent working code as intentional; mention
unrelated issues rather than fixing them. Confirm intent before destructive or
hard-to-reverse actions, and prefer a reversible path.

## Skills may be absent

Harnesses drop skill descriptions past a context budget. If an expected skill
is not listed, invoke it by name or say it is unavailable and proceed from these
rules. Never invent what an absent skill would have done.

## Communication

Direct and execution-first. Structure claims as statement, mechanism, stakes.
Short declarative sentences. No marketing language.

Every significant document names the uncomfortable thing — the scenario that
hurts the author's own position.

## Done

Done requires the actual agreed user goal and value to be achieved, supported by relevant end-of-phase VLBV evidence against accepted business rules and all existing mandatory exit criteria. Deliver the necessary user guidance and access, verify the observable outcome, disclose limits and remaining risks, and record reflection. Review-ready means a coherent change is ready for maintainer assessment, not that the user outcome is already delivered. Preserve existing approval, release and verification boundaries; tests alone are not completion.

## Execution scaffold

For mixed fleets: smaller models skip these behaviors silently, producing
plausible output with a fabricated call in it. Omit only when every model that
reads this file is measured to supply them.

### Before executing

Restate the task in one sentence, and name the phase. If the restatement does
not match what was asked, stop and ask rather than proceeding on the closer
reading. Name the files you intend to touch before touching them.

### Do not fabricate

Never invent an API, file path, package name, command flag or configuration
key. Verify anything not read this session or pinned in `versions.toml`; "I
could not confirm this exists" is a correct answer. Read a tool's schema before
calling it; invented arguments fail in ways that look like a broken tool.

### Verification is explicit

Paste the command and its actual output. Never report a result you did not
observe or describe what a test "should" produce.

### Code output

Never elide code with `...`, `// rest unchanged`, or a similar placeholder in a
file you are writing. Emit the complete content of every file you write.

When editing, change the minimum span. Do not reformat, reorder imports, or
rename adjacent symbols while making an unrelated change.

Match the file's existing conventions over your own defaults.

### Complete coherent sets

Batch related work until a production path is complete; do not build or test
after every edit, and do not start an unrelated subsystem while a set is partial.

### Stop conditions

Stop and ask when: the requirement is ambiguous in a way that changes the
design, two readings of the task lead to different files, the change would
break an existing behavior, or you are about to do something hard to reverse.

Stop when the goal is met. Do not continue into adjacent improvements.

### Format contracts

When a format is requested (JSON, table, diff, schema), emit exactly that: no
preamble, no trailing commentary, no fence unless asked. A parser may read it.

### Self-check before reporting completion

State each of these explicitly, not as a claim that you did them:

1. What changed, file by file.
2. What was run to verify it, and the observed output.
3. What was added that was not requested — remove it, or list it and ask.
4. Which guards trace to an observed failure, and which do not.
5. What remains unverified, and why.

<!-- profile: mixed — see references/MODEL-PROFILES.md before changing -->
<!-- prometheus-base:end -->

<!-- uiux-routing:start v1 -->
## UI/UX routing
UI, styles, tokens, motion or copy → `prometheus-ui-ux`. Read `.agents/UI_UX_PROTOCOL.md` or its bundled default; preserve design authority.
All code: detect `.agent-team/project-routing.json` and real team manifests. Preserve selection; adopt a sole team; ask if ambiguous. Use relevant roles, disclosing sequential fallback.
Backend work loads no UI guidance. Review respects user-only skills and the completed-phase boundary.
<!-- uiux-routing:end -->
