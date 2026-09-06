<!--
AUTHOR: M. SZUL
AI MODEL: GPT-5.6 Sol
TIMESTAMP: 2026-09-06 Europe/London
PURPOSE: Record the verified Esperanto morphology single-affix milestone and the exact remaining gates. This file documents evidence only; it does not claim that local uncommitted morphology code was pushed by this commit.
REVISION: 2026-09-06 — implementation published as 105b0c6 on docs/darkstar-headscale-hotspot-plan; evidence note updated.
-->

# Esperanto morphology — single-affix milestone 572/572

## Repository state note

The active development branch is `docs/darkstar-headscale-hotspot-plan`.

The implementation and harness changes are published in implementation commit
`105b0c6` (`feat(darkstar-embed): complete single-affix Esperanto morphology conformance`).
The documentation record was imported from `1cac9c54f7d6e723af47f08135054e878f6de96a` and is
being updated in a separate documentation commit.

## Proven single-affix gate

Final conformance result:

- TOTAL: 572
- PASS: 572
- RECOVERY_FAIL: 0
- BUDGET_EXCEEDED: 0
- Analyzer construction: 317 ms
- Full single-affix runtime: 106043 ms

Verification reported with that result:

- `cargo test -p darkstar-embed eo_morph` — 5 passed
- `cargo clippy -p darkstar-embed --bin eo-morph-conformance -- -D warnings` — passed
- `git diff --check` — passed

The validation used the existing temporary local `cbms-writing` override only during execution;
the tracked dependency remains pinned to
`bad82f2e5f11973ceedf067d368f6983b093ea3d` and no local path dependency was retained.

This closes the single-affix conformance milestone only. It is not yet a declaration that the complete morphology rule engine is finished.

## Implemented behavior represented by this milestone

The local `eo_morph` work now includes the following verified behavior:

- duplicate `.dic` root entries retain all flag variants rather than overwriting earlier variants;
- direct root lookup is indexed;
- deterministic PFX ADD and SFX ADD candidate indexes are built at load;
- deterministic ADD-length inventories are available for candidate discovery;
- affix flag to rule-ID indexing is available for forward diagnostics;
- candidate matching remains UTF-8 safe;
- rule ordering/provenance remains deterministic;
- `NEEDAFFIX` bare roots remain rejected;
- legal zero-surface `STRIP=0, ADD=0` affix transitions can satisfy `NEEDAFFIX` when the affix rule is otherwise legal;
- zero-surface transitions do not emit an artificial empty morpheme;
- rule flag and condition checks remain in force;
- a focused regression test covers the zero-surface `NEEDAFFIX` case.

## Why the final 11 failures disappeared

Before the final parser fix, the in-process single-affix run produced:

- TOTAL: 572
- PASS: 561
- RECOVERY_FAIL: 11
- BUDGET_EXCEEDED: 0

All 11 failures had the same mechanism: the dictionary root required `X=NEEDAFFIX`, while a legal affix rule had `STRIP=0` and `ADD=0`. Reverse analysis skipped the zero-surface transition, reached the root as if it were bare, and correctly rejected the root for unsatisfied `NEEDAFFIX`.

The correction was to represent the legal zero-surface grammatical transition in reverse analysis without inventing an empty morpheme and without making bare `NEEDAFFIX` roots valid.

After that fix, the exact same single-affix gate reached 572/572.

## Performance findings

Earlier diagnostic isolation found eight cases that exceeded a 2-second per-process budget. All eight completed successfully when given more time, so they were not semantic parser failures.

Measured examples included long legal paths with thousands to tens of thousands of unique reverse states and many candidate transitions. Completed cases reported zero repeated reverse states. The observed performance issue is therefore candidate/state-space explosion, not repeated-state recomputation.

Memoization is not currently justified by the measured state-reuse evidence.

Production runtime optimization is still required later, after the morphology acceptance contract is fully frozen.

## Remaining morphology-rule-engine gates

The following remain open and must be proven before declaring `MORPH_RULE_ENGINE_COMPLETE`:

1. continuation conformance;
2. isolated legal PFX/SFX cross-product conformance;
3. negative conformance for illegal rule combinations;
4. exhaustive `NEEDAFFIX` classification;
5. full `eo_frek.txt` regression over 33,349 forms;
6. final CODE ANALYSER review of the complete morphology-rule-engine milestone.

`eo_frek.txt` is a regression corpus, not a definition of all productive Esperanto morphology.

## Full-system sequence after morphology

The project goal remains a fully usable system, not an experiment. After the morphology rule engine is proven:

1. ambiguity resolver;
2. canonical production morpheme inventory;
3. authoritative ordinary CBMS IDs for canonical morphemes;
4. production lossless NWRD vocabulary growth;
5. semantic initialization/transfer for new rows under an explicitly verified method;
6. input normalization/correction layer where required by the real artifacts;
7. language-to-Esperanto frontend;
8. CBMS-to-Esperanto-to-English output path;
9. full productive unseen-word and end-to-end tests.

No ordinary training, weight mutation, Book growth, NWRD growth, or commit/push claim is implied by this evidence note.
