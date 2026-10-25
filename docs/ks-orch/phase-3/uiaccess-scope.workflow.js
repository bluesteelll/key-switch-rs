export const meta = {
  name: 'ks-uiaccess-scope',
  description: 'Owner chose uiAccess deployment (open question 3): documented requirements and effects, the codebase impact on the current master, then a revision of the Phase 3 plan and the launch lane spec',
  phases: [
    { title: 'Evidence' },
    { title: 'Revise' },
  ],
}
const WT = 'D:/key-switch-rs/key-switch-rs'
const OUT = 'D:/tmp/ks-orch/phase-3/uiaccess/'
const SAFETY = 'SAFETY: the owner uses this machine. Never run key-switch-rs.exe, the daemon or swch, never install a global hook, never call SendInput or inject input, never write the registry, never create, import or change certificates, never change system settings (reads are fine). Read-only on the repository: no edit, commit, checkout or cargo command in it.'
const HDR = `CONTEXT: key-switch-rs, a Windows keyboard remapper (Rust). Today the release daemon embeds a requireAdministrator manifest (build.rs + app.manifest) so that its WH_KEYBOARD_LL hook also sees keys typed into elevated windows. On 2026-10-25 the owner decided roadmap open question 3: move to a uiAccess="true" deployment (a signed executable installed in a secure location, running without administrator rights). Master is the current origin/master (git rev-parse master; the plan was scoped at 252daef, and 1c1f3a0 and the hand-off chore changed only book/ and docs/). Work tree ${WT}. Output directory ${OUT} (create it). The Phase 3 plan draft is D:/tmp/ks-orch/phase-3/plan_draft.md; the launch lane spec is D:/tmp/ks-orch/p3-3.3-launch-de-elevate/spec.md; the scope of item 3.3 is D:/tmp/ks-orch/phase-3/scope/3.3-launch.md.
${SAFETY}`

const R = { type: 'object', properties: { report: { type: 'string' }, summary: { type: 'string' } }, required: ['report', 'summary'] }
const run = async (label, ph, type, schema, body, effort) => {
  const o = { label, phase: ph, agentType: type, schema, model: 'opus', effort }
  let res = null
  try { res = await agent(`${HDR}\n${body}`, o) } catch (e) { res = null }
  if (res) return res
  log(`${label}: typed agent ${type} gave nothing; retrying with .claude/agents/${type}.md as the role`)
  const plain = Object.assign({}, o, { label: `${label}-role` })
  delete plain.agentType
  return agent(`${HDR}\nYOUR ROLE: read ${WT}/.claude/agents/${type}.md first and follow it exactly.\n${body}`, plain)
}

phase('Evidence')
const [research, impact] = await parallel([
  () => run('research:uiaccess', 'Evidence', 'researcher', R,
`ROLE: researcher - the documented facts of a uiAccess deployment, for the architect of the deployment lane. Use Microsoft Learn first (the application manifest reference, "User Account Control" and UIPI pages, the "Only elevate UIAccess applications that are installed in secure locations" policy, SendInput, SetWindowsHookEx, LowLevelKeyboardProc), then Raymond Chen's The Old New Thing, docs.rs for the windows 0.62 crate, and how AutoHotkey ships its "UI Access" executables (its documentation and installer source on GitHub). Cite every claim; mark anything you could not confirm as UNVERIFIED. Questions:
1. Requirements: the manifest form (level and uiAccess attributes, whether level must be asInvoker, whether requireAdministrator with uiAccess="true" is valid and what it does); the Authenticode signature (must the chain end in a root in the machine's Trusted Root store; does a self-signed certificate that the owner imports there work; is a timestamp needed); the secure locations; what happens at start when a requirement is missing (the error), and whether a uiAccess executable starts without a UAC prompt from a non-elevated parent.
2. What it grants: the integrity level the process runs at (for a standard user and for an administrator in Admin Approval Mode); whether a WH_KEYBOARD_LL hook in it receives keys typed into elevated windows; SendInput and posted messages into elevated windows; SetWinEventHook; anything still out of reach (the secure desktop, Ctrl+Alt+Del, UAC prompts).
3. Child processes: the token, integrity level and uiAccess flag that a program started by a uiAccess process gets through CreateProcess and through ShellExecuteEx; whether a launched program can end up with more than medium integrity; how ShellExecuteEx with the "runas" verb behaves from such a process (the UAC prompt).
4. Signing options for one person's open-source project: a self-signed certificate imported by the owner on their own machine (the exact manual steps the owner would take, which the agents never take), a commercial OV certificate, Microsoft's cloud signing service (Trusted Signing / Artifact Signing) and its eligibility and price, SignPath's free plan for open source; signtool usage; SmartScreen effects.
5. Developer workflow: how debug builds and cargo test binaries avoid the uiAccess manifest; how projects keep one manifest for development and one for release; how AutoHotkey installs and signs its UIA executable.
6. Anything that changes the design of a low-level keyboard hook daemon: raw input with RIDEV_INPUTSINK, the foreground WinEvent hook, OpenProcess/QueryFullProcessImageNameW on an elevated foreground process from a medium-integrity process (PROCESS_QUERY_LIMITED_INFORMATION), WTSRegisterSessionNotification, named pipes between a medium daemon and a medium client.
Write ${OUT}research.md with one section per question, a table of requirements, edge cases, and the open points. Do not design the lane. Return the report path and a summary.`, 'high'),
  () => run('impact:uiaccess', 'Evidence', 'project-analyst', R,
`ROLE: project-analyst - the codebase impact of moving the release daemon from requireAdministrator to asInvoker + uiAccess="true", on the current master. Map, with file:line:
1. Every place that assumes or documents that the daemon runs elevated: build.rs and app.manifest (and how only the release daemon gets the manifest), swch open (how it starts the daemon today), the single-instance object, the IPC pipe and its DACL (KS-26, KS-37, KS-40, item 4.1), the config location next to the executable (src/config/loader.rs; under Program Files a standard user cannot write it, which pulls roadmap item 4.4 "Config location" forward), the log folder, the registry reads, the foreground cache's process queries, Launch/OpenUrl (KS-09, KS-34), the watchdog's raw input, the control window, any known issue, gap, audit line or book page that says "elevated" or "administrator" (grep book/src, README.md, CHANGELOG.md, default_config.ron).
2. What breaks or changes for the owner's daily use: where the executables must live, how a release build is produced, signed and installed, how config.ron is found and edited, autostart (item 4.5), the console window (item 4.2 background mode), and what a debug build does.
3. Roadmap items that change: 3.3 (KS-09 may be solved by the deployment itself, or only partly; elevated: true becomes a UAC prompt through ShellExecuteEx runas per the owner's answer), 4.1, 4.2, 4.4, 4.5, the release pipeline and the code-signing row of "Project infrastructure", open question 3.
4. A proposal: which work forms the deployment lane(s), what each lane's acceptance is (automated where possible; the uiAccess start itself is a manual check, since agents never sign, install or run the daemon), the order relative to the other Phase 3 lanes in plan_draft.md, and which Phase 4 items to pull into Phase 3 because the deployment needs them. Note every manual step only the owner can take (creating or importing a certificate, installing under Program Files).
Write ${OUT}impact.md. Return the report path and a summary.`, 'xhigh'),
])
if (!research) log('uiaccess research missing')
if (!impact) log('uiaccess impact missing')

phase('Revise')
const REV = { type: 'object', properties: {
  report: { type: 'string' },
  lanes: { type: 'array', items: { type: 'object', properties: { order: { type: 'integer' }, name: { type: 'string' }, branch: { type: 'string' }, title: { type: 'string' }, spec_path: { type: 'string' }, size: { type: 'string' } }, required: ['order', 'name', 'branch', 'title', 'spec_path', 'size'] } },
  owner_questions: { type: 'array', items: { type: 'object', properties: { question: { type: 'string' }, options: { type: 'array', items: { type: 'string' } }, recommendation: { type: 'string' }, evidence: { type: 'string' } }, required: ['question', 'options', 'recommendation', 'evidence'] } },
  owner_steps: { type: 'array', items: { type: 'string' } },
  summary: { type: 'string' } }, required: ['report', 'lanes', 'owner_questions', 'owner_steps', 'summary'] }
const rev = await run('revise:plan', 'Revise', 'project-analyst', REV,
`ROLE: project-analyst - REVISE the Phase 3 plan for the owner's uiAccess decision. Inputs: ${OUT}research.md${research ? '' : ' (MISSING)'}, ${OUT}impact.md${impact ? '' : ' (MISSING)'}, D:/tmp/ks-orch/phase-3/plan_draft.md and its lane specs D:/tmp/ks-orch/p3-*/spec.md. Owner answers already given on 2026-10-25 (binding): KS-19 release like an AutoHotkey remap (plan Q1 option b); PressKey("Ctrl+S") yes; known issue 66 fixed for Tap and Hold, DoubleTap documented; known issue 73: lock the winner for the whole press; deployment: uiAccess; elevated: true shows a UAC prompt; elevated: only on Launch, not OpenUrl; without cwd: the user's profile folder; AltGr: right Alt on an AltGr layout stops counting as Ctrl+Alt in Phase 3 (Breaking; fold it into p3-3.1-held-modifiers item 6 or its own lane); no v0.2.0 tag, v0.3.0 only after Checkpoint 3 and the owner's go; the lanes run back to back, each merged on ACCEPT. Verify the inputs' key claims against the tree and the cited documentation where they look doubtful.
Write ${OUT}plan_revision.md: (1) what the uiAccess decision changes in the plan (lanes added, changed, reordered; Phase 4 items pulled forward and why; KS-09/KS-34 status under the new deployment); (2) the revised lane order of the whole phase with reasons; (3) for every new or changed lane, its spec written to D:/tmp/ks-orch/<lane name>/spec.md in the format of the existing specs (rewrite D:/tmp/ks-orch/p3-3.3-launch-de-elevate/spec.md in place if the lane keeps its name, or give the new name; the spec must name the research report as evidence and fold in the owner answers above); (4) the owner's manual steps (certificate, install), in order, which agents never take; (5) the remaining owner questions, values and scope only (for example which signing route), each with options, a recommendation and evidence; (6) the revised Checkpoint 3 rows and manual checks for the deployment. Return the report path, the ordered lanes, the owner questions, the owner steps and a summary.`, 'xhigh')

return { research: research && research.report, impact: impact && impact.report, revision: rev }
