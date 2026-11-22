@AGENTS.md

<!-- prometheus-team-routing:start v1 -->
For every code task, read `.agent-team/project-routing.json`, then its active team manifest and the relevant role instructions. Default to that team, selecting only roles whose responsibilities and ownership match the work. Preserve native permissions, models, concurrency limits and existing project instructions.
For UI work, load the role-bound `prometheus-ui-ux` or `prometheus-ui-review` skill. Prefer `.agents/UI_UX_PROTOCOL.md` when present; otherwise use the installed `prometheus-ui-ux/references/UI_UX_PROTOCOL.md`. Backend work must not load UI guidance.
Use native delegation when available. If unavailable, follow the selected role instructions sequentially and report that limitation. Review in the builder context is not independent review. Keep reviewers dormant until the complete implementation phase; allow one batched correction/confirmation cycle. Respect user-only skill invocation restrictions. Zed external ACP agents use their own native configuration; parallel UI threads are not an automatic delegation API.
Cross-team requests: when work needs another team, run `team-discover` (capabilities, paths) and `team-request`; a same-repo team gets an intake task, any other team gets a `team:<id>` GitHub issue. If this team has a card, run `team-intake` to import open `team:<id>` issues as tasks for its intake role, and triage them before other work.
<!-- prometheus-team-routing:end -->

<!-- uiux-routing:start v1 -->
## UI/UX routing
UI, styles, tokens, motion or copy → `prometheus-ui-ux`. Read `.agents/UI_UX_PROTOCOL.md` or its bundled default; preserve design authority.
All code: detect `.agent-team/project-routing.json` and real team manifests. Preserve selection; adopt a sole team; ask if ambiguous. Use relevant roles, disclosing sequential fallback.
Backend work loads no UI guidance. Review respects user-only skills and the completed-phase boundary.
<!-- uiux-routing:end -->
