// The host admits bounded header projections for one account and two folders.
// The recipe recognizes explicit demo requests in English subject lines only.
type Row = { sourceRef: string; threadRef: string | null; timestamp: number;
  direction: "inbound" | "outbound" | "auto" | "draft" | "unknown"; subject?: string };
type Input = { cutoff: number; coverage: { inbox: boolean; sent: boolean }; rows: Row[] };

const input: Input = await Bun.stdin.json();
const threads = new Map<string, { request?: Row; latestSend?: number; unknown?: boolean }>();
const sources = new Set<string>();
const unassigned = new Set<string>();
const requestIntent = /\b(?:can (?:we|i)|could (?:we|i|you)|please|(?:we|i) (?:would like|want|need)|(?:schedule|request|book) (?:a|the))\b/i;
for (const row of input.rows) {
  if (row.timestamp > input.cutoff || sources.has(row.sourceRef)) continue;
  sources.add(row.sourceRef);
  const requested = row.direction === "inbound" && /\bdemo\b/i.test(row.subject ?? "") &&
    requestIntent.test(row.subject ?? "");
  if (!row.threadRef) {
    if (requested) unassigned.add(row.sourceRef);
    continue;
  }
  const thread = threads.get(row.threadRef) ?? {};
  if (requested && (!thread.request || row.timestamp > thread.request.timestamp ||
      (row.timestamp === thread.request.timestamp && row.sourceRef < thread.request.sourceRef))) thread.request = row;
  if (row.direction === "outbound") thread.latestSend = Math.max(thread.latestSend ?? -Infinity, row.timestamp);
  if (row.direction === "unknown") thread.unknown = true;
  threads.set(row.threadRef, thread);
}
const complete = input.coverage.inbox && input.coverage.sent;
const findings: Array<{threadRef: string; sourceRef: string}> = [];
for (const [threadRef, thread] of threads) {
  if (!thread.request) continue;
  if (thread.unknown) { unassigned.add(thread.request.sourceRef); continue; }
  if (complete && (thread.latestSend ?? -Infinity) <= thread.request.timestamp)
    findings.push({threadRef, sourceRef: thread.request.sourceRef});
}
findings.sort((a,b) => a.threadRef < b.threadRef ? -1 : a.threadRef > b.threadRef ? 1 : 0);
const unmatched = [...unassigned].sort();
console.log(JSON.stringify({unanswered: findings.slice(0,20), unassigned: unmatched.slice(0,20),
  coverage: complete ? "complete" : "partial", truncated: findings.length > 20 || unmatched.length > 20}));
