# Supplement jig-17.s2: a busy watch is opened again, and closed once answered

Added by ruling 319 (the designer's answer of 2026-10-18T05:52:56Z,
ruling 316). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan"; jig's `client/domain/client.md`, 8.1 ("Counted until the engine
has ended it", "Dropped is not ended", "Bounded") and 8.2 (`me`).

Scope: jig's client, `crates/jig-client-domain`, as client.md 8.1 and
8.2 say at jig `544a57091`; nothing else.
Needs: nothing.

Shape:
- **Busy is dropped, not ended:** `watches.rs:153`'s `WatchEnd::Busy`
  joins `Dropped | Stopping` (`:148`): opened again after the backoff,
  its handles behind.
- **`me` ends only with the sign-in or its version:** for the watch
  whose `own` is set, every end but `SignedOut` and `NotServed` is
  opened again after the backoff, the link offline until its next
  delivery (`:172-174` already sets offline when nothing ends it).
- **A closing waits for its opening's answer:** `domain.rs:244-247`
  (`unwatch`) and `watches.rs:132-135` (`refuse_watch`) send
  `Down::Close` at once for a named watch. A watch whose opening has no
  answer yet (no `WatchEvent::Opened`, no delivery, no terminal) is
  marked closing and sends nothing; `Opened` then sends its closing,
  and a terminal removes it, as `ended` does for a closing watch
  (`:143-146`).
- **One kept for `me`:** `domain.rs:181`'s `self.watches.len() ==
  env.limits.watches` refuses the application's watch at one fewer
  while the table holds no `own` watch.

Test shape: step tests in `crates/jig-client-domain/src/tests/watches.rs`,
each under 20 ms: `a_watch_refused_busy_is_opened_again_after_a_backoff`
(the row `(WatchEnd::Busy, Why::Busy)` of
`a_refused_watch_ends_each_of_its_handles_once`, `:76`, leaves its
table, `ROWS` 4: its expected value changes by the design, named in the
commit body); `me_refused_busy_is_opened_again_and_the_link_is_offline`;
`a_watch_closed_before_its_opening_is_answered_closes_once_opened`;
`a_watch_closed_then_refused_sends_no_closing`;
`the_application_s_watches_leave_one_for_me`.
Proof: land's gate.
Size: aim for at most 80 production lines; past it, the report says
why.
Don't: change the link's exchanges, the wire, or the engine's rooms;
change `Ended`'s reasons for an application's watch other than busy.
A further test whose expected value changes with these rules is
refreshed and listed in the commit body.
