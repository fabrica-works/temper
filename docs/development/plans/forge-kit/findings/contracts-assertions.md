Main 4c0895d676c9c80ebc5e95c1f03d686f076e0b0c; original contracts has 60 direct assertion executions, including table iterations, excluding inherited Harness assertions (retained).

| Original meaningful assertion(s) | Focused destination in scratch |
|---|---|
| stored Repository=0 | repository_measure_has_no_payload |
| stored bare branch=3 | branch_measure_counts_its_name |
| stored comment+echo=35 | stored_measure_counts_comment_and_echo_stamps |
| stored review=51 | stored_measure_counts_review_stamps |
| stored cached label=70 | stored_measure_counts_cached_label_width |
| stored cached pull=92 | stored_measure_counts_cached_pull_reviewers |
| answer Items=131 | items_measure_uses_fixed_summary_width |
| answer Item=100 | item_measure_uses_fixed_comment_width |
| answer Pull=22 | pull_measure_uses_fixed_reviewer_width |
| answer Job=5 | job_measure_counts_step_and_log |
| four exact-byte acceptances | answer_bytes_at_bound_are_admitted: Items131/Item100/Pull22/Job5 |
| four byte-minus-one refusals | answer_bytes_past_bound_are_refused: 130/99/21/4 |
| four row predicate refusals | answer_rows_past_bound_do_not_fit: branches/labels/reviewers/protection5 |
| four row admission refusals | row_answers_past_bound_are_refused, room83; all five-row payloads fit byte room |
| scalar Answer::Done rows0 | scalar_answers_need_no_row_room |
| measured_read initial read empty, all twelve invocations | scheduling_reads_emit_no_terminal; byte fixtures at cap/cap-1 and four five-row fixtures at original LIMITS.answer_bytes4096 |
| key kit/a exact bytes | keys_frame_the_namespace_and_opaque_identity, unchanged |
| invalid empty namespace,65536 namespace,empty identity,cap6 | invalid_key_frames_are_refused, unchanged |
| largest namespace key len65539,prefix[1,255,255],last a | the_largest_namespace_keeps_its_identity, same assertions, MAX_KEY length and NAMESPACE_BYTES allocation |
| identity bytes,repeat equal,different identity unequal,different namespace unequal | opaque_identities_are_stable_and_distinct, unchanged |
| bind configured succeeds,historical namespace recognized | binding_replaces_the_namespace_before_restore, unchanged |
| bind unconfigured fails,config None | binding_without_configuration_changes_nothing, unchanged |
| bind with op_bytes3 fails,namespace kit retained | binding_past_op_bytes_preserves_the_namespace, unchanged |
| effect reviewer16,pull6,merge3 | effect_measure_uses_fixed_widths, named independent sizes/identities; table retained |
| NEW four-row predicates | answer_rows_at_bound_fit, four shapes |
| NEW four-row admissions | row_answers_at_bound_are_admitted, four shapes |

Independent bytes: branch3; stamps3+2*16=35; review35+16=51; label51+16+3=70; pull70+3+3+2*8=92.
Answer Items128+3=131; Item96+3+1=100; Pull3+3+2*8=22; Job3+2=5.
Rows4: branches64/labels64/reviewers38/protection67; rows5:80/80/46/83. Named ROW_BYTE_ROOM83 admits every payload by bytes.
Namespace: u16 maximum65535, one past65536; version1+length2+namespace65535+identity1=65539; kit repeats21845.
Original total: keys18 + stored6 + effect3 + answer(measure4 + scheduling12 + terminals12 + row predicates4 + scalar1)=60. New total68 (+8 successes).
Current helpers are private; source sweep finds measured_read only in contracts and measure_row in keep plus its one contracts caller. After02.3 resolve the actual fixture module, do not invent or require a new API.
Count sweep: only skein/02-client.md final Done when explicitly consumes114/40new. Its 02.4 proof114 is historical; earlier81/84/90 are before this supplement, not targets to rewrite. Other grep hits are unrelated sizes/timings.
README change proposed only: add the supplement row/dependency, increment supplementary-session inventory, make final repin await it; keep original17 and skein01.s1 scope intact.
