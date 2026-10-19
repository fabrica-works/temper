# Who drafted each file (experiment 7)

The director's decision 94 (10-18 23Z) moved drafting from Opus
sub-agents to Codex gpt-6-astra at xhigh (`codex-astra` below). When it
arrived, fourteen Opus drafters (one per session file, and later.md)
were already running: they were let finish rather than their spend
thrown away; three were lost in the account's outage of 10-18 23:20Z
and redrafted by Codex. The director's rust-25 (10-19) stopped new
Codex runs at xhigh: the README, decisions.md and the last fixes are the
planner's own (Opus) or Sonnet sub-agents'. "Reviewed by" names the
checks each file had before landing; every file was also read against
its siblings by the planner, from the drafters' reports.

| File | Drafter | Reviewed by |
|---|---|---|
| README.md | planner | Sonnet (consistency, literal) |
| decisions.md | planner, from the drafters' reports and the designer's answers | planner |
| code-organization.md | Sonnet | Sonnet fixer (cross-session names) |
| later.md | opus, revised by the planner (the split) | planner |
| skein/01-given-keys.md | opus | Sonnet (consistency, literal) |
| jig/01-repin.md | opus (rehearsed in a scratch worktree) | Sonnet (consistency, literal) |
| jig/02-named-send.md | opus | Sonnet (consistency, literal) |
| jig/03-turn-read.md | codex-astra (its Opus drafter lost) | Sonnet (consistency, literal) |
| jig/04-test-cost.md | opus, redrafted by codex-astra after the user's ruling on question 20, then revised by it (profile cause C) | Sonnet (consistency, literal) |
| jig/05-deadline-spans.md | codex-astra, fixed by it after question 34 | Sonnet (consistency, literal) |
| temper/01-repin.md | opus | Sonnet (consistency) |
| temper/02-tools-table.md | opus, split and revised by codex-astra (466 lines before) | Sonnet fixer (cross-session names) |
| temper/03-tools-root.md | opus, split, revised and fixed by codex-astra (465 lines before) | Sonnet fixer (cross-session names) |
| temper/04-waits.md | opus, revised by codex-astra | Sonnet fixer (cross-session names) |
| temper/11-tools-forge.md | codex-astra (split from temper 03), revised by it | Sonnet fixer (cross-session names) |
| temper/12-tools-help.md | codex-astra (split from temper 02), revised by it | Sonnet fixer (cross-session names) |
| findings/next/05-ctl.md | codex-astra (its Opus drafter lost) | none: the next plan's input |
| findings/next/06-client-stories.md | opus | none |
| findings/next/07-forge-host.md | opus | none |
| findings/next/08-real-loop.md | codex-astra (its Opus drafter lost) | none |
| findings/next/09-real-loop-stories.md | opus, split by codex-astra; its fix stopped at capacity | none |
| findings/next/10-process.md | opus, revised and fixed by codex-astra | none |
| findings/next/13-real-loop-landing.md | codex-astra (split from temper 09) | none |
| findings/jig-test-profile.md | codex-astra | planner |
| findings/base-times.md | planner | planner |
