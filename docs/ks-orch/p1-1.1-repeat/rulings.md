# Orchestrator rulings for lane p1-1.1-repeat (2026-10-16)

- **Q1, the KS-29 corner** (a press after a lost key-up reads as an auto-repeat and, for a non-repeating
  binding, is swallowed without firing; the next press works): ACCEPTED as documented, as the cut
  recommends (CHANGELOG Fixed entry, known-issues row, scenario comment). The root fix is roadmap item 2.3
  (state reset on WTS_SESSION_UNLOCK / desktop switch) in Phase 2.
- The critic notes for items 1.6 and 2.3, and the CHANGELOG.md:242 reflow, are for the phase book sync,
  not for this lane's rework.
