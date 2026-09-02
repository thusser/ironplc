# TwinCAT/CODESYS Dialect Work — Status

**Living document.** Update this as work progresses so it's possible to
resume from a different machine. This branch (`twincat-dev` on
`thusser/ironplc`) is the testing/integration branch for all TwinCAT-related
work in one place -- individual pieces get merged into `main` separately via
PRs, but `twincat-dev` should always reflect everything, landed or not.

## 2026-09-02: routine sync to `main`, no conflicts

20 commits, all garretfick's own bug-fixing sweep (constant-range
checks, install.sh, `--dump-vars`, discovery docs, steering-doc
reorg, etc.) -- nothing TwinCAT/OOP-specific. Clean merge, no
conflicts (`869efe16`). Full `just` CI clean (compile, coverage
≥85%, clippy, fmt, dupes).

Checked issue #1199 and thusser's PRs/assigned issues while here:
no new activity since the 2026-08-24 comment quoted below; no open
PRs, nothing assigned. Nothing pending for us right now.

## 2026-08-31: PR #1362 merged via #1449; `twincat-dev` synced to `main`

PR #1362 is **closed**, superseded by garretfick's own **PR #1449**
(merged 2026-08-28, `a860646a`). Same three commits (original
authorship preserved), rebased onto a fresh `main` with conflicts
resolved on his end -- including the exact `effect_of`/`METHOD_CALL`
gap the 2026-08-24 entry below predicted whoever rebased #1362 would
hit again. He also added four verifier tests, a disassembler test and
arm, and updated `specs/design/bytecode-instruction-set.md`. Our
METHOD-call codegen (including the void-`RETURN` stack-leak fix) is
now on `main`, out of our hands.

Synced `twincat-dev` to `main` (44 commits: our merge conflicted, since
`main` now carries #1449's own version of everything twincat-dev's
prior sync had improvised locally). Conflicts and resolutions:
- **`compile_method.rs`** (add/add): took `main`'s version wholesale --
  it's a strict superset, additionally fixing a method-local variable
  leak between sibling methods (#1439) and binding a method's own name
  to its return slot (`GetSpeed := speed` pattern), neither of which
  `twincat-dev`'s version had.
- **`compile_stmt.rs`**: same `MethodCall.receiver` match as before;
  took `main`'s wording (`self_ref.span()` instead of `call.span()` for
  the `SelfRef` diagnostic -- more precise).
- **`compile.rs` / `compile_fn.rs`**: trivial -- `main` added a
  `struct_array_vars` field to `SavedFbScope` (top-level `ARRAY OF
  <struct>`, #1415, unrelated to OOP) that our snapshot/restore code
  didn't know about yet; wired it through. Also a stray constant-pool
  population block that `main`'s optimizer refactor had relocated
  elsewhere in the same file -- dropped our copy at the old location.
- **`opcode.rs`** (whole-file rewrite conflict): `main` replaced the
  old per-opcode `pub const` declarations with a macro-generated table
  (from the #1446 container-viewer work), and it already carries
  `METHOD_CALL` fully wired through the new system. Took `main`'s
  version wholesale.
- **`end_to_end_methods.rs`** (add/add): no real conflict, just two
  extra upstream tests appended after ours (own-name assignment,
  method-local shadowing a field) -- kept both, all 7 tests unique.
- Deleted `specs/plans/2026-08-12-oop-method-declarations-static-dispatch.md`
  per `main`'s new policy (adopted since our last sync) that plans are
  deleted before merge, not carried forward.

**One bug introduced by the merge itself, caught before pushing**: the
auto-merge (no conflict reported, so no manual review) left *two*
`METHOD_CALL` arms in `verify.rs`'s `effect_of` match -- our old one
from the 2026-08-24 fix below, and #1449's own version, both landing
via clean recursive merge since neither textually overlapped the
other's insertion point. Rust caught it only as an `unreachable_patterns`
warning, not a build failure. Removed the dead (ours) arm, kept #1449's
(uses a cleaner `method_return_depth` helper). Committed separately
(`f5264b44`) since it was missed in the merge commit itself -- the fix
was made in the working tree before running CI but never `git add`ed,
so the merge commit as committed still had the duplicate; caught by
`git status` showing an unstaged diff afterward, not by CI.

Full `just` CI clean after both commits (compile, coverage ≥85%,
clippy, fmt, dupes, plan-citations).

Checked issue #1199 and open TwinCAT/OOP issues while here: no new
comment since the 2026-08-24 one quoted below. Garretfick has opened
~15 new parsing-defect issues since then (#1418-#1433 range) plus
#1467 (STRING/WSTRING method-return `Diagnostic::todo`, formalizing a
known limitation) -- all his own, none assigned to us, consistent with
"refuse codegen until parsing is correct." Nothing of ours pending
review; no open PRs.

## 2026-08-24: void-METHOD RETURN stack leak fixed; synced to `main` again

garretfick flagged a real bug on PR #1362 (posted 2026-08-21, via Claude
Code on his end): `RETURN;` inside a void `METHOD` compiled to a
value-pushing `RET` instead of `RET_VOID`, because
`compile_method.rs` set `CurrentFunctionReturn::Scalar` unconditionally
regardless of whether the method had a return type. The call site only
pops the return slot `if has_return_value`, so the leaked value
(`fb_ref`) accumulates on the VM's never-reset operand stack across scan
rounds and eventually traps `StackOverflow` far from the real cause.
Fixed with a one-liner (`has_return_value.then_some(...)`); added a
regression test that calls a void method with an early `RETURN;` in a
loop within one scan round, and confirmed it reproduces the trap on the
unfixed code. Pushed to both `twincat-dev` and PR #1362's branch; full
CI clean.

Also on issue #1199, garretfick said he's deliberately not reviewing
any codegen PRs (including #1362) until parsing defects he's found are
fixed first -- "better to refuse codegen than generate the wrong code."
So #1362 is parked on his end for now; don't keep re-poking it.

Separately pulled `main` into `twincat-dev` (15 commits, incl.
THIS^/SUPER^ parsing under the same ADR-0041, and a new
compile-time operand-stack-balance verifier in
`container/src/verify.rs`, #1394). One real merge conflict in
`compile_stmt.rs`: main had stubbed `MethodCall` codegen as
`Diagnostic::todo_with_span` (declarations-only slice); kept our real
`compile_method_call` implementation. Two follow-up fixes needed after
resolving it:
- `MethodCall.instance: Id` was renamed to `receiver: MethodReceiver`
  upstream (to support THIS^/SUPER^; the analyzer already rejects
  `SelfRef` receivers pre-codegen, tracked in #1406), and
  `Diagnostic::todo_with_span` dropped its `file!()`/`line!()` args for
  `#[track_caller]`. Updated all affected call sites.
- The new verifier's exhaustive `effect_of` match in
  `container/src/verify.rs` didn't know our `METHOD_CALL` opcode, so it
  fell into the same `UnknownOpcode` error a call to an undefined
  function would produce -- a confusing "not an assigned opcode" report
  even though the opcode *is* registered (`is_assigned` returned true
  when checked directly). Added a `METHOD_CALL` arm mirroring `CALL`,
  but since a method (unlike a `FUNCTION`) can be void, the push count
  can't use `CALL`'s fixed `RET_DEPTH` -- it's derived from whether the
  callee's bytecode ends in `RET` vs `RET_VOID`.

Note for whoever next rebases PR #1362 onto a fresh `main`: it will hit
this exact same `effect_of`/`METHOD_CALL` gap again, since the fix so
far only lives on `twincat-dev`'s own merge commit, not as something
upstream is aware needs adding when method codegen eventually lands
there.

Not rebasing/force-pushing #1362 itself right now, per garretfick's
"not reviewing codegen yet" note above -- would just be repeated churn.

## 2026-08-20: `twincat-dev` synced to `main` (v0.239.0)

Routine sync, no PR changes. Local `main` had fallen 26 commits behind;
fast-forwarded `57819df8 -> d8af5b80` (v0.239.0). Upstream had advanced
by exactly one commit since the last entry (`d8af5b80`, a CI version
bump touching only `Cargo.toml`/`Cargo.lock`/`package.json`/`docs/VERSION`
-- no source changes missed).

`twincat-dev` rebased onto the new tip per the standing playbook (old
tip archived, branch rebuilt on fresh `main`): the two twincat commits
(`a677c849` codegen, `d059fcbc` status doc) reapplied cleanly, zero
conflicts. Verified the codegen patch is byte-identical after rebase
(`git diff 82e17eb5 1ea861d0` == `git diff d8af5b80 a677c849`); only
version-bump files differ in the tree, as expected. Old tip preserved
as **`twincat-dev-archive-20260820`** (`23aac7c7`).

**Correction to an earlier session's audit**: the archive refs the
2026-08-16/17 entries mention *do* exist -- they're **tags**, not
branches (`refs/tags/twincat-dev-archive-20260816` -> `803c3865`,
`refs/tags/twincat-dev-archive-20260817` -> `f3526146`). The doc was
accurate; the earlier check had only looked at `git branch`, not
`git tag`.

Re-verified PR state against upstream while here: #1361 `CLOSED`,
#1386/#1378 `MERGED`, #1362 still `OPEN` + `MERGEABLE` with no review
decision yet. Fork `main` (thusser/ironplc) remains 133 commits behind
upstream -- expected, since the fork only hosts `twincat-dev` and opens
PRs upstream.

## 2026-08-20: per-file corpus re-check -- 48/166 pass (~29%)

Re-ran the standing per-file corpus check (`ironplcc check --dialect
twincat`, one file at a time) against all 8 real solutions under
`/home/husser/code/brotlib`, built from fresh `main` v0.239.0 + our
codegen commit (`feature/twincat-oop-method-codegen`).

**48/166 files pass clean (~29%)**, down from 57/176 (~32%) at the
2026-08-12 checkpoint. Caveat: the denominator changed (176 -> 166
files; brotlib is not a git repo, so the file-set drift can't be
diffed). First error code per failed file, vs 08-12:

| Code | 08-12 | Now | Trend |
|---|---|---|---|
| P0002 syntax | 49 | 35 | down -- parser accepts more |
| P2008 cross-file resolution | 27 | 41 | up |
| P9999 not-implemented-in-codegen | 19 | 18 | flat |
| P4038 non-constant initializer | 11 | 11 | flat |
| P4017 undeclared function | 8 | 8 primary / 36 occurrences | flat as primary, up as secondary |

Secondary codes also rose on the same files (not counted in the first
code per file): P4012 FB-not-in-scope 34 occurrences, P4007 undefined
var 29, P4008 const-without-init 18, P4017 36. This is exactly the mix
shift the 08-13 entry predicted -- more `instance.Method(args)` now
parses, but a chunk of it doesn't resolve against `EXTENDS`-inherited
members yet. Spot-checked representative failures; all genuine (e.g.
`FB_NUTATE.TcPOU` P0002 on `1/189474` in an array initializer,
`FB_CO_ABERRATION.TcPOU` P2008 on `FB_SUNPOS`/`FB_IAU2000B` types).

Per-solution pass: IAG50cm 22/50, BROTLib 20/57, AstroBROT 3/22, MONETN
1/10, HalfBROT 1/10, MONETS 1/4, MONETRoof 0/5, MONETcommon 0/8.

**Gap re-triage** (the six "Remaining gaps" items) -- **corrected
2026-08-20 after source-level verification** (the initial version of
this entry said items 1 and 2 were still present; that was wrong):

- **Item 1** (function-call case-sensitivity) -- **already fixed, long
  before this work started**: `FunctionEnvironment` stores lowercase
  keys with a dedicated test
  (`function_environment_get_when_case_insensitive_then_finds_function`,
  added in #436, 2026-02-06). The "not yet confirmed" caveat in the
  Remaining gaps section is stale and can be removed.
- **Item 2** (`^.` inside a structured/call-style initializer) --
  **already fixed** via `allow_struct_initializer_expressions`, which is
  enabled in the `twincat` dialect (`options.rs`; flag introduced in
  #1276). Verified with an isolated repro of the exact corpus pattern
  (`tonDelta : TON := (PT:= pCover^.Delta);` in an EXTENDS FB): checks
  clean, exit 0. The corpus file's failure on this line is a
  **per-file-check artifact**, not a gap: `pCover` is declared in the
  *base* class `FB_CoverState` (separate file), so single-file checking
  can't resolve it -- that's the known cross-file P2008 family, not
  initializer grammar.
- **Item 3** (`THIS^.Method()`) -- **genuinely still open, confirmed
  live in the corpus** (`FB_MonetCoverControl.TcPOU:116`:
  `THIS^._SendTelemetry();`). Isolated repro: `THIS^.field := x` parses
  but fails semantic (P4007, THIS undefined); `THIS^._CallMe()` fails
  to *parse* -- after `THIS^._Method` the grammar accepts
  `.`/`:=`/`REF`/`[`/`^`/newline but **not `(`**. This is a parse-level
  grammar + symbol-resolution gap, independent of the METHOD-call
  codegen in #1362 -- the one genuinely unblocked, corpus-confirmed
  item left. Candidate for its own small PR (parse-level only, no
  codegen, per the maintainer's no-stacking guidance).
- **Item 4** (external Beckhoff types) confirmed still present --
  `AXIS_REF`/`MC_Home` in IAG50cm and MONETN.
- **Item 6** (namespace-qualified identifiers): no real hits outside
  string literals (the `AUXILIARY.SENSOR[1].NAME`-style matches are
  quoted strings, not identifiers) -- still not in the private corpus.
- Item 5 unchanged (blocked on #1362's codegen landing).

Bottom line: nothing here changes the "wait on the maintainer" status
for #1362, but the gap list shrinks -- items 1 and 2 are closed, and
the remaining corpus-confirmed gaps are cross-file resolution
(P2008/P4012), external Beckhoff types (item 4), and the parse-level
`THIS^.Method()` (item 3).

## 2026-08-17: #1361 closed too -- superseded by garretfick's own #1386

garretfick closed [#1361](https://github.com/ironplc/ironplc/pull/1361)
himself: *"This was built on top of a change for project discovery which
I've taken a different direction. I merged something equivalent."*
Merged his own **[#1386](https://github.com/ironplc/ironplc/pull/1386)**
("Add METHOD declarations and static dispatch (ADR-0041 Phase 1)")
instead -- structurally and even textually identical to our #1361 (same
file list, same shared `call_assignment_check.rs` extraction, same
`MethodDeclaration` struct including our exact doc comments), codegen
explicitly deferred there too ("stub in compile_stmt.rs ... deferred to
follow-up slice") -- so #1362 (our codegen slice) is still the right next
piece, just needed rebasing onto his version instead of ours.

**Rebuild**: from fresh `origin/main` (now including #1386), cherry-picked
only the one real codegen commit from the old
`feature/twincat-oop-method-codegen` -- applied clean (two files
auto-merged, no manual conflict resolution). The pre-existing
`compile_stmt.rs` `MethodCall` stub #1386 added was correctly replaced by
our real implementation, not left duplicated. Full CI clean, all 4
`end_to_end_methods.rs` VM-execution tests pass unchanged. Force-pushed
over `feature/twincat-oop-method-codegen`; #1362 now `MERGEABLE`, PR
description updated to point at #1386 instead of the closed #1361.
`twincat-dev` reset to `main` + fast-forwarded again; old tip preserved
locally as `twincat-dev-archive-20260817`.

**#1361 itself needs no further action** -- fully superseded, nothing to
carry forward from it that isn't already in #1386.

## 2026-08-16: branch rebuilt from `main` again -- #1342 superseded

garretfick closed [#1342](https://github.com/ironplc/ironplc/pull/1342)
himself (not merged), superseded by his own
[#1378](https://github.com/ironplc/ironplc/pull/1378) -- rebased onto
`main` and went further than his review comments asked: the
`.plcproj` recursive-walk fallback is gone entirely, replaced by strict
manifest-chain resolution (`.sln` -> `.tsproj` -> `.plcproj`, or a bare
`.plcproj`; ambiguous/unreadable manifests are now a diagnostic, never a
silent heuristic). Merged to `main` 2026-08-16.

17 other commits also landed on `main` since the last sync (not audited
individually; notable ones: `#1332` codegen emit-macro refactor touching
`emit.rs`, `#1377` FB member-access type resolution, `#1381` compile
pipeline unification). This made #1361
(`feature/twincat-oop-method-declarations`) show a real merge conflict
against `main` for the first time (`mergeable: CONFLICTING`), since its
branch still carried the now-obsolete `.sln`/`.plcproj` split from the
old `twincat-dev`.

**Rebuild, same playbook as 2026-08-10**: identified the 7 commits on
`feature/twincat-oop-method-declarations` that are actually METHOD-work
(not the superseded `.sln` discovery commits underneath them), created
`feature/twincat-oop-method-declarations-v2` from fresh `origin/main`,
cherry-picked all 7 -- every one applied clean or auto-merged, zero
manual conflict resolution needed. Same for
`feature/twincat-oop-method-codegen` (1 commit on top). Full CI clean on
both. Force-pushed (`--force-with-lease`) over the original branch names
so PRs #1361 and #1362 updated in place rather than needing new PR
numbers -- both now show `mergeable: MERGEABLE`. Old `twincat-dev` tip
preserved locally as `twincat-dev-archive-20260816` (not pushed
anywhere). `twincat-dev` itself reset to `main` + fast-forwarded in the
rebuilt `feature/twincat-oop-method-codegen` (which already contains the
declarations work).

## 2026-08-13: duplication audit on #1361, per garretfick's general feedback

garretfick's follow-up comment on #1199 went beyond the specific
plc2plc test file he'd already flagged (fixed earlier today): *"I want
the tests to follow existing patterns to minimize duplicate code.
Claude is producing too much similar code which then becomes a
maintenance issue."* Audited the rest of the METHOD work:

- `parser/tests/methods.rs` and the analyzer rule's own tests already
  matched their closest sibling files' existing conventions (plain
  `#[test]` functions like `fb_inheritance.rs`; the `rule_ok_with!`/
  `rule_err1_with!` macros like every other rule test) -- no change
  needed there.
- Real duplication found: `rule_method_call_declared.rs`'s
  `check_assignments` and four small owner-lookup helpers were verbatim
  copies of `rule_function_block_invocation.rs`'s, with a comment
  explicitly justifying the copy as intentional ("duplicated rather than
  shared"). Extracted both into a new shared
  `analyzer/src/call_assignment_check.rs`; each rule now calls it with
  its own diagnostic label strings. Incidentally fixed an inconsistency
  in the FB-invocation rule's own mixed-args diagnostic (had different
  label text than its sibling branches; no test depended on the exact
  old text). Full CI clean, all 741 analyzer tests unchanged.

## 2026-08-13: pulled main (picks up #1360's discovery-abort fix)

A peer session working against a private TwinCAT corpus flagged that
`twincat-dev` was missing #1360 (merged to `origin/main` 2026-08-12
22:26 UTC) — confirmed directly (`278e1450` wasn't an ancestor). Effect
while missing: every whole-project check against a real solution
referencing a non-bundled vendor library (i.e. every real TwinCAT
solution) silently returned zero real diagnostics. Merged `origin/main`
into `twincat-dev` (also picks up #1339 cross-crate test-dedup and two
unrelated plan docs); no conflicts, full CI clean.

Same session's corpus re-check (with #1360 unmasking real diagnostics):
pass-rate headline unchanged at 69/173 (~40%) after today's METHOD/
EXTENDS work, but the error mix shifted -- fewer flat parse failures
(P0002 50→36), more EXTENDS-chain resolution failures (P2008 77→161)
and undeclared-function (P4017 45→65). Reads as: the parser now accepts
more `instance.Method(args)` syntax that used to be a flat parse
failure, but a chunk of it doesn't resolve against `EXTENDS`-inherited
members yet -- consistent with the known gap already noted below (derived
FB types don't get runtime storage for inherited fields, so codegen
can't support inherited-method calls yet either). Not diffed to
per-file/per-construct level.

## 2026-08-13: review feedback on #1342 and #1361

garretfick reviewed both open PRs. Addressed:

- **#1342** (`.sln` discovery): `discovery/mod.rs` had grown past the
  project's 1000-line module limit. Split into `discovery/sln.rs`
  (`.sln`/`.tsproj` resolution) and `discovery/plcproj.rs` (`.plcproj`
  parsing/merging, including the `Tc2_BuiltIns` implicit-library logic
  merged separately on `twincat-dev` in the meantime — required a real
  three-way merge, not just a rename, to avoid losing that). `mod.rs` now
  979 lines. Pure refactor, all 43 discovery tests pass unchanged.
- **#1361** (METHOD declarations): three real comments fixed — P4046.rst
  wrongly called OOP method syntax "the CODESYS/TwinCAT dialect
  extension" (it's IEC 61131-3 Edition 3, and "dialect extension" is a
  banned phrase per the glossary); `MethodDeclaration.edge_variables` got
  a doc comment explaining it's inherited for free from reusing the
  standard variable-declaration grammar (IEC 61131-3 §2.4.3), not a
  fabricated TwinCAT-specific capability; `plc2plc/src/tests/methods.rs`
  collapsed into one `#[rstest]` table matching `corpus.rs`'s existing
  shorthand. A fourth comment (on `discovery/mod.rs`) was #1342's diff
  surfacing through the PR stack, not something to fix in #1361 directly
  — already covered by the #1342 fix above.

No review yet on #1362 (codegen).

## 2026-08-12: OOP method declarations + codegen, slice 1 (ADR-0041 Phase 1)

Started ADR-0041 Phase 1 (static method/property dispatch), scoped as
two stacked PR-sized slices:

- [#1361](https://github.com/ironplc/ironplc/pull/1361) (open):
  `METHOD ... END_METHOD` declarations, `instance.Method(args)` call
  statements, and static-dispatch resolution across the `EXTENDS` chain
  (own methods first, then base, then base's base, ...), with a new
  diagnostic `P4046 MethodNotFound`.
- [#1362](https://github.com/ironplc/ironplc/pull/1362) (open, stacks on
  #1361): codegen. A method call now actually executes and mutates the
  FB instance's own persistent field — proven end-to-end via real VM
  execution (`codegen/tests/it/end_to_end_methods.rs`), not just
  opcode-level unit tests. New `METHOD_CALL` opcode reuses `FB_CALL`'s
  existing copy-in/copy-out machinery (confirmed by reading the VM:
  ADR-0021's flat variable table has no heap/pointers at all — each FB
  type shares one scratch region across instances).

Together: `ironplcc check` against real TwinCAT code using methods now
gets real diagnostics instead of a parse error, and `ironplcc compile` +
run actually executes method calls correctly.

**Not in either slice**: `THIS^`/`SUPER^`, `PROPERTY`, method calls in
expression position (statement-only for now), `.TcPOU` XML method wiring
(TwinCAT stores each method as a separate `<Method>` XML element; not
yet read), and — discovered while implementing codegen — **calling a
method reached only via `EXTENDS`** hits `Diagnostic::todo`: derived FB
types don't currently get runtime storage for inherited fields at all (a
separate, pre-existing gap in `EXTENDS` support, confirmed by checking
that `inherited_fields.rs` is used only for semantic name-duplication
checks, never to flatten storage onto the derived type). Full design and
remaining task list:
`specs/plans/2026-08-12-oop-method-declarations-static-dispatch.md`.

## 2026-08-12: pass-rate re-measured; found and fixed a discovery-abort bug

**Re-ran the per-file corpus check** (`ironplcc check --dialect twincat`
against every `.TcPOU`/`.TcGVL`/`.TcDUT`/`.TcIO` file across the 8 real
solutions, 176 files found vs. 173 at the last checkpoint): **57/176 pass
clean (~32%)**, up from 49/173 (~28%). Failure breakdown: `P0002` syntax
errors 49 (no single dominant new pattern; spot-checking 20 found a
recurring "bare identifier followed by `(` where `.`/`:=`/`REF`/`[` is
expected" shape in ~6 of them, plus qualified-enum-value initializers
like `E_TestState.IDLE` -- not triaged further), `P2008` cross-file
resolution 27 (structural, as before), `P9999` not-implemented-in-codegen
19, `P4038` non-constant initializer 11, `P4017` undeclared function 8,
small tail 5.

**Found and fixed a real regression while validating garretfick's
catch-up work** (separate from the above; affects whole-*project*
checking, not the per-file corpus numbers above, which don't hit
discovery/library resolution at all): `create_project` in
`ironplc-cli/src/cli.rs` dropped the entire built project on any
discovery-time diagnostic -- an unresolvable `.plcproj` entry, or (since
#1320) a `P6011` referenced-but-unbundled compatibility library -- before
ever running `project.semantic()`. Since IronPLC bundles only four
libraries so far, any real solution referencing a vendor library outside
that set (most of them -- visualization, motion control, IoT, ...) got
**zero analysis output**, just library-not-found noise, regardless of
real bugs in its own files. Verified directly against all 8 real
solutions here -- all 8 hit this. Fixed in
[#1360](https://github.com/ironplc/ironplc/pull/1360) (open, not yet
merged as of 2026-08-12): `create_project` now always runs the caller's
real work against whatever resolved, folding the discovery diagnostic
into the final result afterward instead of short-circuiting before it.
Verified before/after against `MONETN`: went from "28 `P6011`s, nothing
else" to "28 `P6011`s plus 22 real `P2008`/`P4012`/`P4007` diagnostics in
its own POUs."

Also closed [#1246](https://github.com/ironplc/ironplc/pull/1246)
(BOOL_TO_STRING/LREAL_TO_FMTSTR/ADR) -- all three now covered by the
library mechanism and the new `ADR()`/`POINTER TO` language feature,
both merged into `main`.

## 2026-08-11: Tc2_Math and Tc2_BuiltIns landed; MODABS PR closed

The maintainer landed the VM-intrinsics + bindings work he'd said he would
after library work landed, unblocking most of what was still open the day
before:

- **ADR-0042** ("library functions over compiler intrinsics", merged in
  #1340) formalizes the rule: library functions are the default; VM
  intrinsics (`BUILTIN` func_ids) are reserved for operations genuinely
  inexpressible in ST source.
- **`TRUNC_F64`/`MOD_F64`** VM builtins (#1345, merged) -- LREAL-preserving
  truncation/fmod, the primitives `TRUNC`/`MOD` can't express in source.
- **`Tc2_Math`** compatibility library (spec #1350, implementation #1351,
  both merged) -- `LTRUNC`, `LMOD`, `MODABS`, `FRAC`, implemented in ST,
  delegating to the two new intrinsics.
- **`Tc2_BuiltIns`** compatibility library (#1347, merged) -- `BOOL_TO_STRING`,
  **auto-activated** whenever a `.plcproj` is discovered (`REQ-CL-sources-008`,
  no manual `--library` flag needed) -- see `TWINCAT_IMPLICIT_LIBRARIES` /
  `append_implicit_references` in `discovery/mod.rs`.
- Merging `origin/main` into `twincat-dev` conflicted exactly where
  expected: `append_implicit_references` (new on `main`) needed wiring into
  the refactored `merge_plcproj_projects` (new on this branch, from the
  `.sln` discovery work) rather than the old inline `detect_twincat` body
  it was written against. Resolved by calling it at the same point (right
  before constructing the final `DiscoveredProject`) inside the refactored
  function. All 43 discovery tests pass post-merge; full CI passes.

**Verified locally**: `MODABS(-400.56, 360.0)` via `--library Tc2_Math`
matches #1218's own worked example (`319.44`), correctly gated (clean
`P4017` undeclared-function diagnostic without the flag). **Closed #1218**
as obsolete, same disposition as #1219 before it.

Per garretfick on #1199 (2026-08-11): *"I merged in Tc2_Math functions.
Remaining work is one string format and ADR. I have plans for both."*
`LREAL_TO_FMTSTR` and `ADR` (address-of operator) are still open --
`ADR`'s design doc (#1352) is up but not yet merged; plan (#1349) merged.

**Updated disposition, replacing the 2026-08-10 table below:**
- #1218 (MODABS) -- **closed** by us, superseded by `Tc2_Math`.
- #1246 (BOOL_TO_STRING/LREAL_TO_FMTSTR/ADR) -- still open. `BOOL_TO_STRING`
  now redundant (superseded by `Tc2_BuiltIns`, auto-activated). `LREAL_TO_FMTSTR`
  and `ADR` await the maintainer's in-progress work (#1352 design doc for ADR).
- #1342 (`.sln`/`.tsproj` discovery) -- still open, `MERGEABLE`, unreviewed.

## 2026-08-10: branch rebuilt from `main`

The previous `twincat-dev` had an entirely unrelated git history from
`main` (no common ancestor -- likely predates a history rewrite upstream)
and had fallen a full generation behind on some files: notably
`compiler/sources/src/discovery/mod.rs` there still predated the
multi-`.plcproj` sub-project merge (#1279) and the compatibility-library
mechanism (#1307), both long since merged into `main`.

**Reconciled against the issue tracker**: nearly everything from the old
stacked-branch list below is already in `main`, just merged under
different PR numbers than originally opened (some got closed as
superseded when their content landed through a different PR):

| Landed in `main` | | |
|---|---|---|
| #1207 pragma-skipping | #1208 recursive discovery | #1216 LSP multi-workspace merge |
| #1224 AND_THEN | #1239 multi-file merge determinism | #1241 reserved keywords as enum values |
| #1279 multi-`.plcproj` merge | #1280 UDINT<->DWORD widening | #1285 partial-resolution revert fix |
| #1288 DAP loop | #1301 OOP foundation (superseding #1247/#1287) | #1307 compatibility-library mechanism |

**Still genuinely open, unlanded:**
- [#1218](https://github.com/ironplc/ironplc/pull/1218) (MODABS) and
  [#1246](https://github.com/ironplc/ironplc/pull/1246)
  (BOOL_TO_STRING/LREAL_TO_FMTSTR/ADR) -- both blocked on the same
  VM-intrinsics design question asked on
  [#1199](https://github.com/ironplc/ironplc/issues/1199), unanswered as
  of 2026-08-10. See the architecture-change section below.
- [#1342](https://github.com/ironplc/ironplc/pull/1342) (`.sln` ->
  `.tsproj` -> `PrjFilePath` discovery, closes #1292) -- open, merged
  into `twincat-dev` directly since it's this session's own work.

So this rebuild is cheap: reset `twincat-dev` to `main`, then fast-forward
in the one branch with real unlanded code
(`feature/twincat-sln-tsproj-discovery`). The old `twincat-dev` tip is
preserved locally as `old-twincat-dev-archive` if anything from its
history needs to be dug up later (it is **not** pushed anywhere).

## Context

- Filed as [ironplc/ironplc#1199](https://github.com/ironplc/ironplc/issues/1199)
  ("TwinCAT vendor dialect support").
- Motivation: use IronPLC as a diagnostics backend for (1) an IntelliJ
  plugin for TwinCAT Structured Text, and (2) linting TwinCAT projects in CI
  without a Windows/TcXaeShell build agent.
- Baseline measurement from the issue: of 158 real `.TcPOU`/`.TcGVL`/`.TcDUT`
  files across 8 real TwinCAT projects, only 17 (~11%) parsed clean with
  `ironplcc check --dialect codesys` before this work started. Last
  measured checkpoint (2026-08-02, against a fork build with all PRs of
  the time applied): 49/173 (~28%), with the OOP `EXTENDS`/`IMPLEMENTS`
  gate (`P9004`, 46 files) as the single largest remaining bucket at that
  point -- since resolved by #1301's OOP foundation landing. Re-measured
  on 2026-08-20: 48/166 (~29%) per-file against fresh `main` v0.239.0 --
  see the 2026-08-20 corpus re-check entry above.

## Architecture change: compiler-builtin registration rejected, replaced by compatibility libraries

Registering vendor constants/stdlib functions as compiler built-ins behind
a dialect flag (`allow_math_constants`, `allow_extended_math_functions`)
was rejected by the maintainer during review of PI (#1219) and
LTRUNC/LMOD/MODABS (#1217/#1218), on architectural grounds: dialect
(syntax) and library availability (which constants/functions exist) are
orthogonal, and a per-constant compiler flag doesn't scale.

The maintainer instead built a **compatibility library** mechanism, merged
as [#1307](https://github.com/ironplc/ironplc/pull/1307) (2026-08-04):
vendor libraries are reproduced as `.st` files shipped with the compiler
(`compiler/sources/resources/libs/<Name>/<version>/...`) and brought into
a compilation via a new `--library` flag, independent of `--dialect`.
`Tc2_System` (containing `PI`) is the first library shipped this way.

**Verified locally against `main` on 2026-08-10:**

```sh
ironplcc compile --dialect twincat --library Tc2_System \
  --output prog.iplc uses_pi.st
ironplcvm run prog.iplc --scans 1 --dump-vars -
```

- `PI` resolves in statement context (`circumference := 2.0 * PI * 10.0` ->
  `62.83185307179586`, matching the `tests/e2e/library/uses_pi.st` fixture).
- `PI` also resolves as a `VAR` initializer now (`d2r : LREAL := PI/180.0;`
  -> `0.017453292519943295`) -- this was the specific gap the old
  `allow_math_constants` approach never closed.
- Omitting `--library` produces a clean `P4038` diagnostic, not a bare
  undeclared-identifier error or silent misresolution.

**Disposition of the built-in-registration PRs, as of 2026-08-10:**
- #1219 (PI) -- **closed**, superseded by #1307.
- #1217 (LTRUNC/LMOD) -- **closed by the maintainer** (not merged), same
  objection.
- #1218 (MODABS) -- **open, CHANGES_REQUESTED.** Blocked on VM-intrinsic
  work the maintainer said would start after the library mechanism landed
  (LTRUNC/LMOD/MODABS need byte-code-level truncation, not just an
  ST-body library function). Asked on #1199 whether that intrinsic work
  is in progress and whether there's a tracking issue -- **unanswered as
  of 2026-08-10.**
- #1246 (BOOL_TO_STRING/LREAL_TO_FMTSTR/ADR) -- **open, not yet
  reviewed.** `BOOL_TO_STRING` is framed as core IEC 61131-3, likely
  unaffected. `LREAL_TO_FMTSTR` should migrate to the library mechanism
  like PI did. `ADR` doesn't fit the library-as-ST-body model at all
  (it's an address-of operator) -- asked on #1199 whether it should be a
  VM intrinsic or stay a `__`-prefixed compiler intrinsic like the
  existing `__SYSTEM_UP_TIME` -- **also unanswered as of 2026-08-10.**

**Implication for any remaining/future work**: before adding a new
`--allow-x` flag that registers a specific vendor constant or stdlib
function as a compiler built-in, check whether it should instead be a
`.st` compatibility library file under the `--library` mechanism. Pure
grammar/syntax additions (new keywords, new expression shapes) are
unaffected by this change.

## Remaining gaps (from the 2026-07-20 re-scan; items 1 and 2 re-verified 2026-08-20)

These were identified during the original private-corpus + 5-external-repo
survey and had not been re-triaged as of the last rebuild. Re-verify each
directly against real files before estimating cost -- the standing lesson
from this project (see "Key design decisions" below) is that file-count
rankings and construct names are not reliable cost estimates on their own.

1. **Undeclared function call, actually a case-sensitivity bug** (e.g.
   declared `SOME_FUNCTION`, called `some_function`) -- function-call
   resolution (`FunctionEnvironment`) not yet confirmed case-insensitive,
   unlike `TypeName`/`Id` lookups which already are.
   **CLOSED 2026-08-20**: `FunctionEnvironment` stores lowercase keys
   with a dedicated case-insensitive test since #436 (2026-02-06); the
   "not yet confirmed" caveat was stale.
2. **`^.` (deref + member access) inside a structured/call-style `VAR`
   initializer** -- e.g. `tonDelta : TON := (PT := pDevice^.Delta);`.
   Plain `:=` initializers already handle `^.` fine via `expression()`;
   this is specifically the structured/call-style initializer position.
   **CLOSED 2026-08-20**: supported via
   `allow_struct_initializer_expressions` (#1276), enabled in the
   `twincat` dialect; verified with an isolated repro of the exact
   corpus pattern (checks clean). Corpus failures on this line were a
   per-file-check artifact (base-class var in another file -> P2008
   cross-file family, see item 4), not initializer grammar.
3. **`THIS^.Method()`** -- calling a method via an explicit `THIS^`
   pointer-dereference. Unclear whether it needs its own grammar support
   or composes with the existing qualified-call-parsing work.
   **PARSING CLOSED 2026-08-24** by upstream #1403 (garretfick's own
   "THIS^ and SUPER^ parsing", ADR-0041 Phase 1), picked up here via
   today's `main` sync. Verified directly: `THIS^._SendTelemetry();`
   inside a FUNCTION_BLOCK now parses clean (no `P0002`), stopping only
   at semantic resolution (`P9999` "not yet resolved by IronPLC", from
   `rule_method_call_declared.rs`). What remains isn't parse-level
   anymore, isn't independent, and isn't ours to pick up: garretfick
   opened **#1406** 2026-08-22 ("Reject SUPER^ where there's no base
   type, and THIS^ outside a function block") as the deliberate
   next small PR after #1403, and his 2026-08-24 comment on #1199 says
   he's actively working parsing correctness right now. Don't start
   this -- it's mid-flight under him, and duplicating it would repeat
   the exact "too much similar code" complaint he already raised.
4. **`P2008` remaining pieces**: genuinely external Beckhoff-library types
   with no source in the corpus (`MC_Home`, `AXIS_REF`, etc. --
   Motion/System libraries, not fixable without stub/declaration-only
   registration -- a much larger effort) and one same-project resolution
   gap that only reproduces in a private corpus not available here.
   **Re-confirmed present 2026-08-20** (`AXIS_REF`/`MC_Home` in IAG50cm
   and MONETN).
5. **Full OOP dispatch** (`METHOD`/`PROPERTY` bodies, access modifiers,
   inheritance *dispatch*, `THIS^`/`SUPER^`) -- #1301 landed the
   `EXTENDS`/`IMPLEMENTS`/`INTERFACE` foundation and field-inheritance
   resolution; what's still missing is real `METHOD` body
   parsing/dispatch/codegen. Big, multi-PR effort.
6. **Namespace-qualified identifiers** (`SysFile.ACCESS_MODE`,
   `EXTENDS TcUnit.FB_TestSuite`, `GVL.MaxCount` as an array bound) --
   not in the private test corpus at all (single-namespace project), but
   the single biggest gap found across 5 external public TwinCAT repos
   checked (82/491 files, ~17%). Worth revisiting for any future
   multi-namespace/library-style project.
   **Re-checked 2026-08-20**: still no real hits in the private corpus
   (matches were quoted strings, not identifiers).

## Key design decisions (apply to future work in this area too)

- **Dialect placement**: CODESYS/TwinCAT-shared vendor extensions go on the
  existing `Rusty`/`Codesys` dialects, not a new `BeckhoffTwinCAT` dialect --
  unless/until a construct is found that's genuinely TwinCAT-only and
  wouldn't make sense on plain CODESYS. So far nothing has required that split.
- **Keyword gating**: use the codebase's actual **demotion** pattern
  (`xform_demote_*.rs` -- always lex as the specific token, demote to
  `Identifier` when the flag is off), not a "promotion" pattern.
- **`VendorExtension` trait usage**: implement it unconditionally on a type
  even when that type isn't *always* an extension (e.g.
  `FunctionBlockDeclaration`); let `rule_unsupported_extension`'s visitor
  decide *when* to flag it (e.g. only when `extends`/`implements` is
  present). Reserve unconditional flagging for types that only exist when
  the extension is used (e.g. `InterfaceDeclaration`).
- **New `LibraryElementKind` variants are a bigger deal than new tokens.**
  Adding a token (like `Pragma`) only touches 1-2 exhaustive matches (LSP
  semantic highlighting). Adding a declaration variant (like
  `InterfaceDeclaration`) ripples through toposort, type/symbol
  registration, and needs a deliberate decision for codegen/renderer/MCP
  tools -- though it's usually smaller than it looks, since most of those
  sites use `if let`/wildcards, not exhaustive matches. `cargo build`
  surfaces the real exhaustive-match list precisely; don't try to
  enumerate it by hand in a plan.
- **Verify against real files before implementing**, not just before
  finishing. Checking a private local checkout of a real TwinCAT codebase
  *during* plan-writing, before any code is written, is cheaper than
  finding a gap after implementation -- this is how the `.sln`/`.tsproj`
  stale-duplicate bug (#1292/#1342) was confirmed real before writing any
  code, and how the `.TcIO` interface-file discovery was originally found.
- **A survey's file-count ranking is not a cost estimate.** `PI` looked
  like the cheapest, highest-leverage item on paper (18 files, "pure
  registration"). Testing the actual real-file *pattern* against the
  parser -- not just grepping for the construct's name -- revealed the
  true blocker (and eventual full redesign) was something else entirely.
- **"Normalize away early" for AST changes with a wide blast radius.** When
  a change would otherwise touch a type used in many places, check whether
  the codebase already has a "parsed-but-unresolved placeholder,
  normalized by a dedicated early pass" precedent to follow instead of
  changing the shared type directly.
- **Check ADRs before assuming a runtime-evaluated fix is viable.**
  ADR-0024 forced the initializer-expression design into "must fully
  constant-fold at compile time" rather than "defer to runtime" -- only
  discoverable from the ADR, not from the parser/AST alone.
- **In this `peg`-based grammar, "try A, fall back to B" only works if A
  *fails outright* on the fallback cases.** If A can *partially* match and
  return successfully, PEG's ordered choice locks in that match and never
  tries B. Parse via the more general rule unconditionally and dispatch on
  the *shape* of the result instead. Always run the full workspace test
  suite after such a change, not just tests for the new construct.
- **When a grammar rule silently fails to match and it's not obvious why,
  use `cargo build -p ironplc-cli --features trace`** (the `peg` crate's
  built-in tracing) before guessing -- it prints every rule attempt/match/
  fail to stderr with source position.
- **Dependency-graph edge direction conventions must be checked against a
  working example, not assumed from one arm's code.** Compare a new
  toposort visitor arm's edge direction against an existing,
  actually-exercised arm for the same relationship, not an
  existing-but-rarely-exercised one.

## How to resume from another computer

```sh
git clone git@github.com:thusser/ironplc.git
cd ironplc
git checkout twincat-dev
cd compiler && just   # should pass end-to-end
```

Read this file, then any linked plan docs in `specs/plans/` for full
design detail on specific pieces of work.
