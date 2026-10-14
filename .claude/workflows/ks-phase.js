export const meta = {
  name: 'ks-phase',
  description: 'key-switch-rs roadmap phase: run ks-lane items in order (each merged into master before the next starts), then sync the mdBook with the doc-writer',
  whenToUse: 'A roadmap phase or a batch of features. args: {phase, items: [ks-lane args...], bookSync?: true, continueOnFailure?: false}',
  phases: [
    { title: 'Lanes' },
    { title: 'Book' },
  ],
}
// Lanes run one after another in the main checkout: the items share the engine and the hook adapter,
// so each lane starts from the master the previous one merged. A lane that does not merge stops the phase
// (unless continueOnFailure), because later items usually build on it.
const A = args || {}
const PHASE = A.phase || 'PHASE-UNSET'
const WT = 'D:/key-switch-rs/key-switch-rs'
const SP = 'D:/tmp/ks-orch/' + PHASE + '/'

phase('Lanes')
const lanes = []
for (const item of (A.items || [])) {
  log(`lane ${item.name}: start`)
  let res = null
  try {
    res = await workflow('ks-lane', item)
  } catch (e) {
    res = { name: item.name, stage: 'error', merged: false, error: String(e && e.message ? e.message : e) }
  }
  lanes.push(res)
  log(`lane ${item.name}: ${res && res.merged ? 'merged ' + res.head : 'NOT merged (stage ' + (res && res.stage) + ')'}`)
  if ((!res || !res.merged) && !A.continueOnFailure) break
}

const merged = lanes.filter(l => l && l.merged)
let book = null
if (A.bookSync !== false && merged.length) {
  phase('Book')
  const brief = `ROLE: doc-writer - BOOK SYNC after ${PHASE}. Lanes merged into master: ${JSON.stringify(merged.map(l => ({ name: l.name, branch: l.branch, head: l.head, reports: l.reports })))}.
Use git log and git diff on master to see exactly what changed, and the lanes' cut.md/results.md reports for intent.
On a new branch docs/book-${PHASE} in ${WT}, update the English mdBook in book/src so it describes the CURRENT master accurately, keeping the historical audit (pinned to 7cb3521) intact:
- audit.md: the status line of every KS issue these lanes fixed ("Fixed in <sha> (<branch>)"), plus the Status column;
- config/*.md: every page whose semantics changed. Validate every changed or new RON example against the real loader before publishing it. known-issues.md drops or marks fixed pitfalls;
- architecture.md, build-and-run.md, overview.md, gaps.md: wherever behaviour, modules, threads or commands changed;
- roadmap.md: mark the merged items done with their shas.
Build with mdbook (output outside the repo) and check that no internal link or anchor is broken. Then commit, merge --no-ff into master, push, and delete the branch, per CLAUDE.md. Save ${SP}book.md.
SAFETY: the owner uses this machine. Never run key-switch-rs.exe, the daemon or swch, never install a global hook, never call SendInput or inject input, never write the registry or system settings (reads are fine). This book sync is a merge step: push master only after the merge above; never tag, release or change GitHub settings.`
  try {
    book = await agent(brief, { label: `book:${PHASE}`, phase: 'Book', agentType: 'doc-writer', model: 'opus', effort: 'high' })
  } catch (e) {
    book = null
  }
  if (!book) {
    book = await agent(`YOUR ROLE: read ${WT}/.claude/agents/doc-writer.md first and follow it exactly.\n${brief}`, { label: `book:${PHASE}-role`, phase: 'Book', model: 'opus', effort: 'high' })
  }
}

return { phase: PHASE, lanes: lanes.map(l => l && { name: l.name, stage: l.stage, merged: l.merged, head: l.head, results: l.results, reports: l.reports }), book }
