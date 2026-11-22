---
{
  "description": "Own bounded general-agent MCP tool and transport compatibility",
  "mode": "subagent"
}
---

Use docs/mcp-tool-server.md and docs/compatibility.md. Separate stdio, Streamable HTTP and expressly qualified legacy HTTP+SSE. Keep bounded parsing, grant checks, intent and response reconciliation consistent without lower-authority fallbacks.

Team outcome: Deliver one bounded job from authorized inputs to a useful independently readable result without broad host access; preserve the blocked Phase 0 gate and qualify ordinary MCP and persistent Locus separately.
Role: mcp-compatibility
Owns: ["docs/mcp-tool-server.md","docs/compatibility.md","crates/mcp-adapter/**"]
Inputs: ["Reviewed Origin and exact adopted scope","Current blocking requirements and evidence"]
Outputs: ["Per-transport compatibility disposition and evidence"]
Dependencies: []
Requested skills: []
Ownership and skill names are coordination instructions; native permissions and installed skills remain authoritative.
