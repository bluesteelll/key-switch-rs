export const meta = {
  name: 'ks-lane',
  description: 'key-switch-rs feature lane: cut -> critique -> implement -> pack -> test || review -> triage -> fix -> delta re-verify -> results -> merge; one branch per feature, merged into master',
  whenToUse: 'One roadmap item or feature end to end. args: {name, branch, title, spec, base?, wt?, merge?, stopAfter?, testerModel?, resumeNote?}',
  phases: [
    { title: 'Cut' },
    { title: 'Critique' },
    { title: 'Implement' },
    { title: 'Verify' },
    { title: 'Fix' },
    { title: 'Results' },
    { title: 'Merge' },
  ],
}
// args: name (lane id, also the reports dir), branch, title, spec (the item text: what, why, acceptance),
// base (default 'master'), wt (default the main checkout), merge (default true), stopAfter ('cut'|'critique'|'results'),
// testerModel (default 'sonnet'), resumeNote (implement only - keeps cut/critique cached on a resume).
const A = args || {}
const NAME = A.name || 'NAME-UNSET'
const BR = A.branch || 'BRANCH-UNSET'
const TITLE = A.title || NAME
const SPEC = A.spec || 'SPEC-UNSET'
const BASE = A.base || 'master'
const WT = A.wt || 'D:/key-switch-rs/key-switch-rs'
const SP = 'D:/tmp/ks-orch/' + NAME + '/'

// Model routing and reasoning effort per role (owner 2026-10-14, mirrors boyko-engine; docs/WORKFLOW.md#model-routing).
// Every role on Opus except the tester (Sonnet); xhigh for the roles that think hardest, high for the rest, nothing below high.
// Mirrors the .claude/agents/*.md frontmatter, explicit here so a workflow call never inherits the session's settings.
// The one per-call override is the context-pack scout (project-analyst at effort high) in verify().
const MODEL = { architect: 'opus', 'architecture-critic': 'opus', developer: 'opus', 'code-reviewer': 'opus', 'results-analyst': 'opus', 'project-analyst': 'opus', tester: A.testerModel || 'sonnet', researcher: 'opus', 'doc-writer': 'opus' }
const EFFORT = { architect: 'xhigh', 'architecture-critic': 'xhigh', developer: 'xhigh', 'code-reviewer': 'xhigh', 'results-analyst': 'xhigh', 'project-analyst': 'xhigh', tester: 'high', researcher: 'high', 'doc-writer': 'high' }

const HDR = `LANE ${NAME}: ${TITLE}
Branch ${BR} (from ${BASE}) in the work tree ${WT}. Lane reports directory: ${SP} (create it if missing; every report of this lane is saved there, and briefs refer to reports by path).
Binding rules: ${WT}/CLAUDE.md (quality gate, safety, git flow, conventions, owner decisions) and ${WT}/docs/WORKFLOW.md. Knowledge: book/src/audit.md (KS issues), book/src/roadmap.md (items and acceptance), book/src/architecture.md, book/src/config/*.md.
SAFETY: the owner uses this machine. Never run key-switch-rs.exe or the daemon, never install a global hook, never call SendInput or inject input, never write the registry or system settings (reads are fine). A test that needs any of that is #[ignore = "manual: <what a human checks>"]. Never push, tag, release or change GitHub settings unless this brief is the lane's MERGE STEP.
SPEC:
${SPEC}`

const S = (verdicts, extra) => ({
  type: 'object',
  properties: Object.assign({ verdict: { type: 'string', enum: verdicts }, report: { type: 'string' }, head: { type: 'string' }, summary: { type: 'string' } }, extra || {}),
  required: ['verdict', 'report', 'head', 'summary'].concat(Object.keys(extra || {})),
})
const RATED = { critical: { type: 'integer' }, important: { type: 'integer' } }

const run = async (label, ph, type, schema, body, opts) => {
  const o = Object.assign({ label, phase: ph, agentType: type, schema, model: MODEL[type], effort: EFFORT[type] }, opts || {})
  let res = null
  let err = null
  try {
    res = await agent(`${HDR}\n${body}`, o)
  } catch (e) {
    err = e
  }
  if (res) return res
  // The typed agent is not registered in this session (the roles landed after it started) or died:
  // run the same role from its definition file, same model and effort.
  log(`${label}: typed agent ${type} gave nothing (${err ? err.message : 'null'}); retrying with .claude/agents/${type}.md as the role`)
  const plain = Object.assign({}, o, { label: `${label}-role` })
  delete plain.agentType
  return agent(`${HDR}\nYOUR ROLE: read ${WT}/.claude/agents/${type}.md first and follow it exactly.\n${body}`, plain)
}
const brief = (r) => r ? { verdict: r.verdict, report: r.report, head: r.head, critical: r.critical, important: r.important, confirmed: r.confirmed, summary: r.summary } : null
const out = (stage, extra) => Object.assign({ name: NAME, branch: BR, stage, merged: false, reports: SP }, extra || {})

phase('Cut')
let cut = await run('cut', 'Cut', 'architect', S(['CUT READY', 'CUT BLOCKED']),
  `ROLE: architect - THE CUT. No product code and no cargo builds (read, grep, git log/show only). Write ${SP}cut.md:
(1) every spec item re-located on the current ${BASE} tree (file:line now);
(2) the touch set with a reason per file;
(3) the design - data structures, Win32 APIs with documentation links, threading (which thread owns which hook, window, timer or COM apartment), config/RON changes and their compatibility;
(4) commits in order, per commit the red-first test(s) with the predicted failure on ${BASE} and the gates;
(5) how each acceptance criterion is proven (a test name, or a manual check a human must do);
(6) CHANGELOG / README / default_config.ron / book impact;
(7) risks against the R1-R10 areas of the code-reviewer role;
(8) questions only the orchestrator can decide, each with your recommendation - if one blocks the work, the verdict is CUT BLOCKED.`)
if (!cut || cut.verdict !== 'CUT READY') return out('cut', { cut: brief(cut) })
if (A.stopAfter === 'cut') return out('cut', { cut: brief(cut) })

phase('Critique')
let crit = await run('critique', 'Critique', 'architecture-critic', S(['APPROVED', 'CHANGES_REQUESTED'], RATED),
  `ROLE: architecture-critic. Read ${SP}cut.md and critique it against the spec, the audit/roadmap entries it cites and the tree: a wrong re-location, a red-first that cannot fail, an acceptance criterion without a proof, hot-path cost in the hook callback, hook/window/timer/COM ownership hazards, handle leaks, sentinel discipline, an undeclared config-compatibility break, missing docs. Each finding cites lines, is CONFIRMED or PLAUSIBLE and is rated Critical / Important / Minor. Save ${SP}critique.md.`)
if (!crit) return out('critique', { cut: brief(cut) })
let critFile = `${SP}critique.md`
if (crit.critical > 0) {
  // One revision of the cut while it is cheap; the developer resolves whatever remains.
  cut = await run('cut-v2', 'Critique', 'architect', S(['CUT READY', 'CUT BLOCKED']),
    `ROLE: architect - revise THE CUT. Read ${SP}cut.md and ${SP}critique.md. Resolve every Critical and Important remark in cut.md, or refute it with evidence in a "Critique response" section at the end. Save the revised ${SP}cut.md.`)
  if (!cut || cut.verdict !== 'CUT READY') return out('cut-v2', { cut: brief(cut), crit: brief(crit) })
  crit = await run('critique-v2', 'Critique', 'architecture-critic', S(['APPROVED', 'CHANGES_REQUESTED'], RATED),
    `ROLE: architecture-critic, second pass. Read the revised ${SP}cut.md, including its "Critique response", and check that every earlier Critical/Important remark of ${SP}critique.md is resolved or validly refuted; then look for new problems the revision introduced. Save ${SP}critique_v2.md.`)
  if (!crit) return out('critique-v2', { cut: brief(cut) })
  critFile = `${SP}critique_v2.md`
}
if (A.stopAfter === 'critique') return out('critique', { cut: brief(cut), crit: brief(crit) })

phase('Implement')
const impl = await run('implement', 'Implement', 'developer', S(['DONE', 'STOPPED']),
  `${A.resumeNote ? 'RESUME NOTE: ' + A.resumeNote + '\n' : ''}ROLE: developer. Read ${SP}cut.md and ${critFile} first; if either is missing, the verdict is STOPPED.
In ${WT}: git checkout ${BASE}, then git checkout -b ${BR} (or switch to ${BR} if it already exists - a resume).
Implement per cut.md, commit by commit, red-first: write the regression test, show that it fails, and put the failing line in the commit message body; then fix.
Resolve every Critical and Important critique remark in the code, or refute it with evidence in an addendum at the end of cut.md.
Per commit: cargo fmt; cargo clippy --all-targets --locked -- -D warnings; the tests your diff touches. The full test run is the tester's.
Update CHANGELOG.md (Unreleased). If config semantics change, also update README.md and src/config/default_config.ron.
Every commit message ends with a blank line and "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>".
Leave the tree clean on ${BR}. Save ${SP}impl.md: the commits, tests added, deviations and manual checks.`)
if (!impl || impl.verdict !== 'DONE') return out('implement', { cut: brief(cut), crit: brief(crit), impl: brief(impl) })

const verify = async (round, since) => {
  const ph = round === 1 ? 'Verify' : 'Fix'
  let packLine = ''
  const p = await run(`pack-r${round}`, ph, 'project-analyst', S(['DONE', 'STOPPED']),
    `ROLE: context-pack scout for round ${round}. You are read-only on the repo and write only the pack. Write ${SP}pack_r${round}.md, at most 15k tokens, for the tester and the reviewer:
- git log --oneline and git diff --stat ${since}..${BR};
- per changed file, the line ranges that matter and why;
- the red-first tests and acceptance proofs from ${round === 1 ? SP + 'cut.md and ' + SP + 'impl.md' : SP + 'fix_r' + (round - 1) + '.md'}, with their exact commands and expected results;
- the open points from ${round === 1 ? SP + 'impl.md' : SP + 'triage_r' + (round - 1) + '.md'}.
Facts with file:line only, no opinions.`, { effort: 'high' })
  if (p && p.verdict === 'DONE') packLine = `START from the context pack ${SP}pack_r${round}.md, written for this round; open other files only for what it does not answer.\n`
  const [test, review] = await parallel([
    () => run(`test-r${round}`, ph, 'tester', S(['GREEN', 'RED']),
      `${packLine}ROLE: tester (round ${round}). Work on ${BR} in ${WT}; quote HEAD and confirm the tree is clean. ` + (round === 1
        ? `(1) The quality gate on HEAD: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, and the full cargo test --locked.
(2) Every red-first named in cut.md and impl.md must be red on ${BASE} and green on HEAD. Check ${BASE} in a scratch git worktree under ${SP}wt with its OWN CARGO_TARGET_DIR under ${SP}target-wt, and remove it afterwards with git worktree remove.
(3) Make at least two mutations of your own, in the scratch worktree and never in ${WT}, each of which a named test must catch. Report each mutation and the test that caught it.
(4) For every acceptance criterion, name the test that proves it and run it. List the criteria that can only be checked manually.`
        : `DELTA round - the fix since ${since} (${SP}fix_r${round - 1}.md).
(1) cargo clippy -D warnings, and the tests the fix touches.
(2) Every CONFIRMED item of ${SP}triage_r${round - 1}.md: its red-first is red on ${since} and green on HEAD.
(3) At least two NEW mutations aimed at the fix and at the tests the previous round added, each caught by a named test.
(4) The full cargo test --locked once on HEAD.`)
        + ` Never leave uncommitted changes in ${WT}: mutations and red-first checks run only in the scratch worktrees. A test you add to close a gap is committed on ${BR} as its own commit whose subject starts with 'Tests:', touching test code only (#[cfg(test)] modules, src/engine/scenarios.rs, tests/), with the Co-Authored-By trailer; name it in your report. Save ${SP}test_r${round}.md.`),
    () => run(`review-r${round}`, ph, 'code-reviewer', S(['APPROVED', 'CHANGES_REQUESTED'], RATED),
      `${packLine}ROLE: code-reviewer (round ${round}). Review ${round === 1 ? 'git diff ' + BASE + '...' + BR + ' and the addendum at the end of ' + SP + 'cut.md' : 'git diff ' + since + '..' + BR + ', and whether it closes the CONFIRMED items of ' + SP + 'triage_r' + (round - 1) + '.md'} against the spec, ${SP}cut.md and the R1-R10 risk areas. Read code through git show and git diff, because the tester may be mutating a scratch tree.
Above all, check:
- behaviour regressions for other binding kinds and triggers;
- the suppression invariant (a key-up is suppressed if and only if its key-down was);
- the hook-callback budget (no blocking work in the callback);
- hook, window, timer and COM ownership;
- sentinel discipline;
- a // SAFETY: comment on every unsafe block;
- Win32 handle and error handling;
- tests that can actually fail;
- docs and CHANGELOG.
Report only defects you can point at a line for, each CONFIRMED or PLAUSIBLE, rated Critical / Important / Minor. Save ${SP}review_r${round}.md.`),
  ])
  return { test, review }
}

phase('Verify')
let r = await verify(1, BASE)
const rounds = [{ test: brief(r.test), review: brief(r.review) }]
const isClean = (x) => x.test && x.test.verdict === 'GREEN' && x.review && (x.review.critical + x.review.important) === 0
for (let round = 2; round <= 3 && !isClean(r); round++) {
  phase('Fix')
  const triage = await run(`triage-r${round - 1}`, 'Fix', 'code-reviewer', S(['DONE'], { confirmed: { type: 'integer' } }),
    `ROLE: adversarial verifier of findings. You are read-only: no edits and no commits; scratch probes only under ${SP}.
For EVERY red item in ${SP}test_r${round - 1}.md and every Critical/Important remark in ${SP}review_r${round - 1}.md, try to REFUTE it against ${BR} HEAD and the spec. A finding survives only if you could not refute it.
Per item write one line: '<id> CONFIRMED | REFUTED | DEFERRED-OUT-OF-SCOPE: <reason>'. Mark Minor items as NOTED.
Save ${SP}triage_r${round - 1}.md. confirmed = the number of CONFIRMED items.`)
  if (!triage) return out('triage', { rounds })
  if (triage.confirmed === 0) { rounds.push({ triage: brief(triage), note: 'nothing confirmed' }); break }
  const since = (r.review && r.review.head) || (r.test && r.test.head) || BR
  const fix = await run(`fix-r${round - 1}`, 'Fix', 'developer', S(['DONE', 'STOPPED']),
    `ROLE: developer. On ${BR} in ${WT}, fix every CONFIRMED item of ${SP}triage_r${round - 1}.md as follow-up commits (never amend), red-first; re-run the gates the fix touches. Save ${SP}fix_r${round - 1}.md.`)
  if (!fix || fix.verdict !== 'DONE') return out('fix', { rounds, triage: brief(triage), fix: brief(fix) })
  r = await verify(round, since)
  rounds.push({ triage: brief(triage), fix: brief(fix), test: brief(r.test), review: brief(r.review) })
}

phase('Results')
const results = await run('results', 'Results', 'results-analyst', S(['ACCEPT', 'REWORK', 'RETHINK']),
  `ROLE: results-analyst. Read the spec, ${SP}cut.md, the critique report(s), ${SP}impl.md and every test_r*/review_r*/triage_r*/fix_r* report in ${SP}; then read the actual diff git diff ${BASE}...${BR}.
Verdicts:
- ACCEPT: every acceptance criterion is proven by a test or listed as a manual check, the last tester round is GREEN, and no CONFIRMED Critical/Important item is open.
- REWORK: say exactly what is wrong and which stage it goes back to.
- RETHINK: the design is wrong.
Save ${SP}results.md.`)
if (!results || results.verdict !== 'ACCEPT' || A.stopAfter === 'results' || A.merge === false) {
  return out('results', { cut: brief(cut), crit: brief(crit), impl: brief(impl), rounds, results: brief(results) })
}

phase('Merge')
const merge = await run('merge', 'Merge', 'developer', S(['MERGED', 'NOT_MERGED']),
  `ROLE: developer - MERGE STEP. In ${WT}:
1. Confirm ${BR} is clean, and run the full quality gate on it.
2. git checkout ${BASE}, then git merge --no-ff ${BR} with the message "Merge branch '${BR}': ${TITLE}", a blank line, and the Co-Authored-By trailer. Resolve any conflicts; for CHANGELOG/README keep both sides.
3. Run the full quality gate on ${BASE}.
4. git push origin ${BASE}, then git branch -d ${BR}.
head = the merged ${BASE} sha. Save ${SP}merge.md.`)
return out('merge', { merged: !!merge && merge.verdict === 'MERGED', head: merge && merge.head, cut: brief(cut), crit: brief(crit), impl: brief(impl), rounds, results: brief(results), merge: brief(merge) })
