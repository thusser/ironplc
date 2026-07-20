# TwinCAT/CODESYS Dialect Work — Status

**Living document.** Update this as work progresses so it's possible to
resume from a different machine. This branch (`twincat-dev` on
`thusser/ironplc`) exists to collect all TwinCAT-related feature branches in
one place and track overall progress — it is not itself intended as a PR;
individual feature branches are merged into `main` separately (or this
branch is rebased/split into PRs once things are ready).

## Context

- Filed as [ironplc/ironplc#1199](https://github.com/ironplc/ironplc/issues/1199)
  ("TwinCAT vendor dialect support"), still awaiting maintainer response as
  of 2026-07-18.
- Motivation: use IronPLC as a diagnostics backend for (1) an IntelliJ
  plugin for TwinCAT Structured Text, and (2) linting TwinCAT projects in CI
  without a Windows/TcXaeShell build agent.
- Baseline measurement from the issue: of 158 real `.TcPOU`/`.TcGVL`/`.TcDUT`
  files across 8 real TwinCAT projects, only 17 (~11%) parsed clean with
  `ironplcc check --dialect codesys` before this work started.
- Original ranked list of blockers from the issue (most-files-blocked
  first): pragma headers (done), `EXTENDS`/`IMPLEMENTS`/interfaces (done),
  `REFERENCE TO`, `AT%I*`, `PI`. **Superseded 2026-07-19 by a fresh re-scan**
  of the same real project set, run after pragma-skipping and
  `EXTENDS`/`IMPLEMENTS`/`INTERFACE` landed:

  | Count | Construct | Status |
  |---|---|---|
  | 18 | `PI` used as a bare identifier | Done, both statement and `VAR`-initializer context — see "Done" below |
  | 14 | `AT %I*`/`AT %Q*` shorthand | Done — see "Done" below (turned out to be a block-structure gap, not an address-syntax gap; 22 files hit it directly) |
  | 13 | `STRING(n)`/`WSTRING(n)` sized-string (~7) + inline FB-constructor-call syntax (~5-6) | Done — see "Done" below. Split apart during implementation: the two sub-issues turned out to be unrelated (a delimiter-choice grammar gap vs. a genuinely new call-style initializer syntax) |
  | 11 | Explicit enum-value assignment (`off := 0,`) | Done — see "Done" below |
  | 10 | `REFERENCE TO` | Done — see "Done" #10 below |
  | 7 | `ABSTRACT` | Done — see "Done" #12 below |
  | 24 | `P9004` (EXTENDS/IMPLEMENTS/INTERFACE) | Expected — this is the "recognized but not yet supported" diagnostic working as designed, not a new gap |
  | 10 | `P2008` GVL/cross-file resolution | Structural (multi-file project support), deliberately out of scope for quick wins |
  | 9 | small semantic tail (undeclared builtins, type mismatches) | `LTRUNC`/`LMOD` done (see "Done" #13 below); rest not investigated |
  | 24 | long tail, 1-2 files each | Not investigated |
  | — | **Namespace-qualified identifiers** (`SysFile.ACCESS_MODE`, `GVL.MaxCount` as an array bound, `EXTENDS TcUnit.FB_TestSuite`) | **New, not in the original private-corpus-based survey at all** — found 2026-07-19 by cross-checking 5 external TwinCAT repos (below). Hit **82 of 491 files (~17%)** there — bigger than any other single remaining item. Doesn't appear in the private test corpus (single-namespace project), so it's lower priority for that corpus's own work specifically, but worth keeping in mind for any future library-style/multi-namespace TwinCAT project. Not investigated beyond confirming the parse failure. |

  **`PI` turned out to be much bigger than "pure registration"** — see
  "Done" below. This is the single most important correction to
  carry forward: don't trust a survey's file-count ranking as a cost
  estimate without testing the actual real-file pattern against the parser
  first.
- Real TwinCAT project files were checked locally, using a private local
  checkout of a real TwinCAT codebase, to validate assumptions before
  implementing. This was essential: it caught that TwinCAT interfaces
  live in a separate `.TcIO` file extension, which the original plan had
  missed entirely.
- **Cross-repo validation (2026-07-19)**: before committing to the next
  item, cloned 5 additional public TwinCAT repos via `gh` code search
  (`Beckhoff/TF6310_Samples` — official samples, 146 files;
  `tcunit/TcUnit` — widely-used testing framework, 94 files;
  `OpenCommissioning/OC_TwinCAT_Core` — a substantial open framework, 91
  files; `fisothemes/FisoThemes-Common-Library-for-TwinCAT`, 150 files;
  `craigmcchesney/SxrVacPlc` — a real facility-control project, 10 files;
  491 files total) to check whether private-corpus-derived findings generalize
  or are one team's idiosyncratic style. Findings:
  - Pragmas (57 files), `EXTENDS`/`IMPLEMENTS` (87 files), and mixed
    `AT`-located blocks (11 files) all appear here too — the three landed
    fixes aren't overfit to the private corpus.
  - `PI` doesn't appear at all (these repos don't happen to use that particular math
    constant) — expected, not every construct needs to be universal.
  - `STRING(n)`/`WSTRING(n)` (parens instead of brackets), inline
    FB-constructor-call, and per-member explicit enum values are all
    confirmed real and independently reproduced as parse failures — not
    private-corpus-specific quirks.
  - Beckhoff's own docs
    ([infosys.beckhoff.com](https://infosys.beckhoff.com/content/1033/tc3_plc_intro/2529504395.html))
    describe per-member explicit enum values directly (`Red := 2, Green,
    Blue := 10`), explicitly calling out *custom enum base types*
    (DWORD/LINT instead of INT) as "extension beyond IEC 61131-3" — less
    clear whether plain per-member value assignment itself needs its own
    flag or is closer to universally-supported CODESYS-core syntax (like
    pragmas). Check the actual IEC 61131-3:2013 grammar text before
    deciding, per the standing "verify before assuming vendor-extension
    vs. standards-gap" lesson.
  - Surfaced the new namespace-qualified-identifier gap above, entirely by
    accident of checking a different kind of codebase (library-style
    repos use namespacing constantly; a single-project codebase like
    the private corpus never needs it).

## Branches

All pushed to `thusser/ironplc` (the user's fork). Upstream PRs against
`ironplc/ironplc` are opened only when explicitly requested (not by
default) -- as of 2026-07-20:

- [ironplc/ironplc#1207](https://github.com/ironplc/ironplc/pull/1207)
  (pragma-skipping) -- open. Two maintainer review comments, both replied
  to: a semantic-highlighting classification question (`COMMENT_INDEX` vs.
  `KEYWORD_INDEX` for the `Pragma` token -- explained the reasoning,
  offered to switch if still preferred) and a pragma-placement
  correctness concern (whether pragmas should be restricted to
  "instruction position" per TwinCAT docs, excluding expressions).
  The second one was verified directly against a real TcXaeShell
  instance rather than assumed from documentation: pragmas placed
  inside an arithmetic expression, a function call's argument list, an
  array subscript (both read and assignment-target position), and a
  struct field-access chain all compiled clean in real TwinCAT --
  contradicting the docs' stricter framing. Reported this back; no
  placement restriction implemented, since the current (permissive)
  behavior already matches real compiler behavior. Maintainer's follow-up
  was positive but not final ("I may be OK with this as is... I would
  then track that as an open issue") -- no further action requested yet.
- [ironplc/ironplc#1208](https://github.com/ironplc/ironplc/pull/1208)
  (recursive project discovery) -- **merged.**
- Internal PR #20 (`feature/twincat-reference-to-no-explicit-deref` into
  `feature/twincat-abstract-instantiation`, on the fork) opened for the
  latest stack addition, per the usual per-branch workflow -- not an
  upstream PR.

**Lesson worth carrying forward**: when a maintainer review comment cites
documented behavior that conflicts with the implementation, verify
against the real toolchain before changing code -- documentation
describes conventions the actual compiler doesn't always enforce. This
same TcXaeShell-verification approach also confirmed the empty-`CASE`
-branch fix below (see "Done" #19) was a real gap, not a
misunderstanding, before any code was written.

**As of 2026-07-19, the branches are a linear stack, not independent
branches merged via merge commits.** Each feature branch is based directly
on the tip of the previous one, in this order:

```
main
 └─ feature/twincat-pragma-skipping
     └─ feature/twincat-extends-implements-interface
         └─ feature/twincat-pi-constant
             └─ feature/twincat-var-initializer-expressions
                 └─ feature/twincat-mixed-located-var-declarations
                     └─ feature/twincat-qualified-method-call-parsing
                         └─ feature/twincat-sized-string-and-inline-fb-ctor
                             └─ feature/twincat-explicit-enum-values
                                 └─ feature/twincat-recursive-project-discovery
                                     └─ feature/twincat-reference-pointer-to
                                         └─ feature/twincat-plcproj-resolution-warnings
                                             └─ feature/twincat-abstract-keyword
                                                 └─ feature/twincat-ltrunc-lmod
                                                     └─ feature/twincat-extends-field-inheritance
                                                         └─ feature/twincat-located-array-mixed-block
                                                             └─ feature/twincat-lsp-multi-workspace-folder
                                                                 └─ feature/twincat-modabs
                                                                     └─ feature/twincat-and-then-operator
                                                                         └─ feature/twincat-empty-case-branch
                                                                             └─ feature/twincat-extends-duplicate-field
                                                                                 └─ feature/twincat-abstract-instantiation
                                                                                     └─ feature/twincat-reference-to-no-explicit-deref
                                                                                         └─ twincat-dev  (= stack tip + this status doc, on top)
```

| Branch | Status | Plan |
|---|---|---|
| `feature/twincat-pragma-skipping` | Done, based on `main` | [2026-07-18-twincat-pragma-skipping.md](2026-07-18-twincat-pragma-skipping.md) |
| `feature/twincat-extends-implements-interface` | Done, based on `feature/twincat-pragma-skipping` | [2026-07-18-twincat-extends-implements-interface.md](2026-07-18-twincat-extends-implements-interface.md) |
| `feature/twincat-pi-constant` | Done, based on `feature/twincat-extends-implements-interface` | [2026-07-19-twincat-pi-constant.md](2026-07-19-twincat-pi-constant.md) |
| `feature/twincat-var-initializer-expressions` | Done, based on `feature/twincat-pi-constant` | [2026-07-19-twincat-var-initializer-expressions.md](2026-07-19-twincat-var-initializer-expressions.md) |
| `feature/twincat-mixed-located-var-declarations` | Done, based on `feature/twincat-var-initializer-expressions` | [2026-07-19-twincat-mixed-located-var-declarations.md](2026-07-19-twincat-mixed-located-var-declarations.md) |
| `feature/twincat-qualified-method-call-parsing` | Done, based on `feature/twincat-mixed-located-var-declarations` | [2026-07-19-twincat-qualified-method-call-parsing.md](2026-07-19-twincat-qualified-method-call-parsing.md) |
| `feature/twincat-sized-string-and-inline-fb-ctor` | Done, based on `feature/twincat-qualified-method-call-parsing` | [2026-07-19-twincat-sized-string-and-inline-fb-ctor.md](2026-07-19-twincat-sized-string-and-inline-fb-ctor.md) |
| `feature/twincat-explicit-enum-values` | Done, based on `feature/twincat-sized-string-and-inline-fb-ctor` | [2026-07-19-twincat-explicit-enum-values.md](2026-07-19-twincat-explicit-enum-values.md) |
| `feature/twincat-recursive-project-discovery` | Done, based on `feature/twincat-explicit-enum-values` | [2026-07-19-twincat-recursive-project-discovery.md](2026-07-19-twincat-recursive-project-discovery.md) |
| `feature/twincat-reference-pointer-to` | Done, based on `feature/twincat-recursive-project-discovery` | [2026-07-20-twincat-reference-pointer-to.md](2026-07-20-twincat-reference-pointer-to.md) |
| `feature/twincat-plcproj-resolution-warnings` | Done, based on `feature/twincat-reference-pointer-to` | [2026-07-20-twincat-plcproj-resolution-warnings.md](2026-07-20-twincat-plcproj-resolution-warnings.md) |
| `feature/twincat-abstract-keyword` | Done, based on `feature/twincat-plcproj-resolution-warnings` | [2026-07-20-twincat-abstract-keyword.md](2026-07-20-twincat-abstract-keyword.md) |
| `feature/twincat-ltrunc-lmod` | Done, based on `feature/twincat-abstract-keyword` | [2026-07-20-twincat-ltrunc-lmod.md](2026-07-20-twincat-ltrunc-lmod.md) |
| `feature/twincat-extends-field-inheritance` | Done, based on `feature/twincat-ltrunc-lmod` | [2026-07-20-twincat-extends-field-inheritance.md](2026-07-20-twincat-extends-field-inheritance.md) |
| `feature/twincat-located-array-mixed-block` | Done, based on `feature/twincat-extends-field-inheritance` | [2026-07-20-twincat-located-array-mixed-block.md](2026-07-20-twincat-located-array-mixed-block.md) |
| `feature/twincat-lsp-multi-workspace-folder` | Done, based on `feature/twincat-located-array-mixed-block` | [2026-07-20-twincat-lsp-multi-workspace-folder.md](2026-07-20-twincat-lsp-multi-workspace-folder.md) |
| `feature/twincat-modabs` | Done, based on `feature/twincat-lsp-multi-workspace-folder` | [2026-07-20-twincat-modabs.md](2026-07-20-twincat-modabs.md) |
| `feature/twincat-and-then-operator` | Done, based on `feature/twincat-modabs` | [2026-07-20-twincat-and-then-operator.md](2026-07-20-twincat-and-then-operator.md) |
| `feature/twincat-empty-case-branch` | Done, based on `feature/twincat-and-then-operator` | [2026-07-20-twincat-empty-case-branch.md](2026-07-20-twincat-empty-case-branch.md) |
| `feature/twincat-extends-duplicate-field` | Done, based on `feature/twincat-empty-case-branch` | [2026-07-20-twincat-extends-duplicate-field.md](2026-07-20-twincat-extends-duplicate-field.md) |
| `feature/twincat-abstract-instantiation` | Done, based on `feature/twincat-extends-duplicate-field` | [2026-07-20-twincat-abstract-instantiation.md](2026-07-20-twincat-abstract-instantiation.md) |
| `feature/twincat-reference-to-no-explicit-deref` | Done, based on `feature/twincat-abstract-instantiation` | [2026-07-20-twincat-reference-to-no-explicit-deref.md](2026-07-20-twincat-reference-to-no-explicit-deref.md) |
| `twincat-dev` | `= feature/twincat-reference-to-no-explicit-deref` tip + one commit adding this file. `cd compiler && just` passes | this file |

**Why stacked, not independent-branches-merged-via-merge-commits (the
original approach):** the first three feature branches were each
independently cut from bare `main`, so each one that also touched
`compiler/parser/src/options.rs`/`lsp.rs`/`list_options.rs` (any PR adding
a new `--allow-x` dialect flag does) collided with the others on merge into
`twincat-dev`. That got resolved by hand each time (see git history before
2026-07-19 for the gory details), but it meant re-deriving the same
resolution repeatedly and — worse — the fourth branch
(`var-initializer-expressions`) ended up carrying the *entire* `twincat-dev`
history (merge commits + this status doc) once it was cut from `twincat-dev`
directly to dodge the conflict, making it unsuitable for an eventual
individual PR. Fixed on 2026-07-19 by rebasing all four branches into a
linear stack instead, so:

- Each branch's diff against its *immediate parent* is just its own
  feature — clean and small, suitable for an individual PR.
- `twincat-status.md` (this file) lives only on `twincat-dev`, one commit
  on top of the stack — never inside any feature branch's own history.
- Conflicts (same shape as before: `options.rs`/`lsp.rs`/`list_options.rs`
  three-way clashes whenever a later branch's flag was added while an
  earlier branch's own flag was already present) still occur during the
  rebase itself, but only once per branch, and the resolution from the
  first time this happened is reusable (see "Rebase conflict resolution"
  below).

**PR sequencing when ready to go upstream:** open PRs strictly in stack
order (pragma-skipping first, then extends-implements-interface, then
pi-constant, then var-initializer-expressions), each only after the
previous one has merged into upstream `main` — then rebase the next branch
onto the new `main` before opening its PR. Since the stack is already
linear and each branch's diff against its parent is clean, this should be
close to conflict-free against upstream (assuming no unrelated upstream
changes touched the same files in the meantime).

### Rebase conflict resolution reference

Rebasing `feature/twincat-extends-implements-interface` onto
`feature/twincat-pragma-skipping`, then `feature/twincat-pi-constant` onto
that, each hit the same conflict shape in
`compiler/parser/src/options.rs`/`compiler/ironplc-cli/src/lsp.rs`/
`compiler/parser/src/tests.rs`/`compiler/mcp/src/tools/list_options.rs`/
three docs pages — each new branch's own commit adds a flag to the same
macro table/test file the previous branch(es) already modified. Resolution
pattern (same both times): keep both sets of additions (never drop one
side), then manually recompute the `FEATURE_DESCRIPTORS`/`rusty_features`/
`codesys_features` count assertions and the `list_options.rs`
`Vec::with_capacity`/`resp.flags.len()` count (git/rebase does NOT flag
`list_options.rs` as conflicting — it silently merges to a stale value,
caught only by the test failing). Current combined counts after all four
branches: **19 total, 19 Rusty, 18 Codesys**. If a future branch adds
another flag, bump all three by one again and re-run
`cargo test --workspace` (not just the new feature's tests) to catch both
the silent `list_options.rs` drift and any unrelated regression the
underlying grammar/AST change might cause (see `var-initializer-expressions`'s
own Implementation Notes for an example: switching an initializer's grammar
production silently broke every *negative-literal* initializer in the
codebase, caught only by the full suite).

## Done

### 1. Pragma-skipping (`--allow-pragmas`)

`{attribute '...'}` treated as opaque trivia (parsed and discarded, like a
comment). Confirmed via web search that this is genuine CODESYS-core syntax
(documented by 3S/CODESYS itself and by Schneider Electric's Machine
Expert), not a TwinCAT-only invention — so it's placed on the existing
`Codesys`/`Rusty` dialects rather than a new `BeckhoffTwinCAT` dialect.

### 2. `EXTENDS`/`IMPLEMENTS` + minimal `INTERFACE` (`--allow-oop-extensions`)

- `FUNCTION_BLOCK ... EXTENDS base IMPLEMENTS i1, i2` parses; fields stored
  as metadata only (no inheritance/dispatch checking).
- `INTERFACE name (EXTENDS base_list)? END_INTERFACE` parses; the interface
  name registers as a known type (via a placeholder
  `IntermediateType::Structure { fields: vec![] }` representation — chosen
  to avoid rippling into `IntermediateType`'s size/alignment/codegen
  matches, since interfaces have no runtime representation yet).
- New `P9004`/`VendorExtension`/`ExtensionOrigin` mechanism: these
  constructs parse cleanly but emit a "recognized but not yet supported"
  diagnostic (blocks codegen) rather than silently pretending
  inheritance/interface dispatch works.
- New `.TcIO` file type + `<Itf>` XML element recognition in
  `twincat_parser.rs` — this was a real gap found only by checking real
  project files, not anticipated by the original plan.
- Deliberately out of scope: `METHOD`/`PROPERTY` bodies, access modifiers,
  `THIS^`/`SUPER^`, inheritance semantics. Confirmed these aren't currently
  blocking parsing because `twincat_parser.rs` already silently ignores
  `<Method>`/`<Property>` XML child elements.

### 3. Implicit `PI` math constant (`--allow-math-constants`)

`PI` registers as a built-in `LREAL` global constant (`std::f64::consts::PI`),
injected as a real `VarDecl` during `resolve_types` so ordinary symbol
resolution, type checking, and codegen's generic top-level `VAR_GLOBAL`
collection all just work with no codegen changes. Resolves anywhere an
ordinary expression is legal — e.g. `x := PI/180.0;`, matching real usage in
`ATAN2.TcPOU`.

As of `feature/twincat-var-initializer-expressions` (below), **also**
resolves as a `VAR` initializer (`d2r : LREAL := PI/180.0;`, the dominant
real usage pattern — 16 of the 18 surveyed files use it that way) when
`--allow-constant-initializer-expressions` is enabled alongside
`--allow-math-constants`.

**Originally hit the same conflict shape as the pragma/OOP merge** when
first landed (since this branch was cut from bare `main`, not from the
prior feature) — since resolved by rebasing onto
`feature/twincat-extends-implements-interface` as part of restructuring all
four branches into a linear stack; see "Branches" above and "Rebase
conflict resolution reference" for the current state of that fix.

### 4. Constant expressions in `VAR` initializers (`--allow-constant-initializer-expressions`)

`VAR` initializers can now be a constant *expression* (arithmetic between
literals and/or references to declared `CONSTANT`s), not just a bare
literal — e.g. `d2r : LREAL := PI/180.0;`. Combined with `PI` (above), this
fully unblocks the dominant real-world pattern from the survey.

- New `InitialValueAssignmentKind::SimpleExpr` placeholder variant, always
  normalized back to the ordinary `Simple` shape (or diagnosed) by a new
  `xform_fold_initializer_expressions` pass before any other code sees it
  — following the same "normalize away early" precedent as
  `IntegerRef::Constant`/`LateBound`. The real blast radius was 6 files
  (found via `cargo build`'s non-exhaustive-match errors), not the 21
  files a naive grep suggested.
- Two non-obvious regressions surfaced only by the *full* test suite, not
  by testing the new feature in isolation — both are the kind of thing
  worth remembering for any future grammar change that routes an existing
  literal-only position through the general `expression()` rule:
  1. "Try `constant()`, fall back to `expression()`" doesn't work in PEG —
     `constant()` greedily matches a *prefix* (e.g. just `3.14` out of
     `3.14/180.0`) and "succeeds", so the fallback is never reached; fixed
     by always parsing via `expression()` and dispatching on the result
     shape instead.
  2. That change broke every negative-literal initializer in the codebase
     (`x : INT := -123;`) because `expression()` wraps a leading `-` as a
     separate unary operator rather than using `constant()`'s built-in
     signed-literal parsing — fixed by collapsing `UnaryOp(Neg, Const(c))`
     back to a directly-negated `Const`, unconditionally (not gated by the
     new flag, since it isn't new capability).
- Full details, including the enum-initializer disambiguation fix (a
  fallible `{? }` grammar action was needed, not a plain fallback), are in
  [2026-07-19-twincat-var-initializer-expressions.md](2026-07-19-twincat-var-initializer-expressions.md)'s
  Implementation Notes.

### 5. Mixed located/plain `VAR` declarations (`--allow-mixed-located-var-declarations`)

An `AT`-located variable (e.g. `tempSensor AT%I*: INT;`) can now appear
inside an otherwise plain `VAR`/`VAR_INPUT`/`VAR_OUTPUT` block, instead of
requiring its own dedicated block — closes the "`AT %I*`/`AT %Q*`
shorthand" survey item (14 files originally estimated, 22 actually hit it
directly; ~240 total `AT %I*`/`AT %Q*` occurrences, overwhelmingly the
bare wildcard form with no size prefix).

- Turned out to be a **block-structure** gap, not an address-syntax gap —
  `AT %I*`/`AT %IX0.0` already parsed fine unconditionally, just only
  inside their own dedicated `located_var_declarations()`/
  `incompl_located_var_declarations()` block type. Confirmed by directly
  reproducing the failure (`P0002` at the first plain declaration
  following a located one in the same block) before designing anything.
- The DSL already supported per-variable locations
  (`VariableIdentifier::Direct`) regardless of context — this was purely a
  parser gap, closed with one new per-declaration grammar rule
  (`located_var1_init_decl()`), not a new AST shape.
- Since the resulting `VarDecl` shape is byte-identical to what the
  pre-existing, always-allowed dedicated-block rules already produce, a
  new `in_mixed_var_block` marker field was needed purely to distinguish
  "used the new mixed-block path" from "used the old dedicated-block
  path" for semantic gating — and that marker had to be computed at the
  *whole-block* level (does the block contain both a `Symbol` and a
  `Direct` identifier?), not per-declaration, or an all-located
  block using the new per-declaration syntax gets incorrectly flagged
  too (caught by a regression test, not by inspection).
- Two implementation bugs surfaced only by actually running the parsed
  output through a debug probe, not by re-reading the grammar: (1)
  `VarDeclarations::flat_map` — the function `var_declarations()` actually
  calls — ignored the new location field entirely and always produced
  `Symbol`, because it built `VarDecl` directly rather than going through
  `into_var_decl()`; (2) `VAR_INPUT`'s separate aggregation path
  (`VarDeclarations::with`) needed its own whole-block mixed-check, since
  it keeps declarations grouped per semicolon-separated line rather than
  flattening them the way `VAR`/`VAR_OUTPUT` do.
- Full details in
  [2026-07-19-twincat-mixed-located-var-declarations.md](2026-07-19-twincat-mixed-located-var-declarations.md)'s
  Implementation Notes.

### 6. Qualified method-call statements parse, recognized-but-unsupported (`P9004`)

`fbComm.Publish('a', 'b');` — calling something through a member instance
— now parses under every dialect (no new flag; there's no keyword to gate
the way `EXTENDS` has one). Reported directly from the user's IntelliJ-plugin
project, not found via the private-corpus/cross-repo survey process — a good
example of the "verify before assuming" habit paying off in the other
direction: an external, independently-confirmed bug report, checked
against real project code before committing to a design.

- Checked the reporting codebase's actual declaration: `fbComm : I_Comm;`
  — an **interface type**, so this is genuine polymorphic method dispatch,
  not a call on a known concrete type. Building that for real (parsing
  `METHOD` bodies — currently rejected outright — plus a vtable mechanism
  and indirect-call codegen) is the same "Full OOP" bucket already listed
  below as a big, multi-PR effort, not a single scoped feature.
- Scoped to match the project's own stated motivation (a linter/diagnostics
  backend, not full execution): parse the qualified call, flag it via the
  existing `VendorExtension`/`P9004` mechanism (same as `EXTENDS`/
  `IMPLEMENTS`/`INTERFACE`), and stop there. `FbCall` gained an
  `Option<Id>` qualifier field — 2 construction sites total, no
  exhaustive-match ripple (it's a struct, not an enum).
- A *second* semantic rule (`rule_function_block_invocation.rs`) needed a
  guard too, found only by running the compiler against a real repro, not
  by reading the grammar: it looked up the qualified call's `var_name`
  (the method name, e.g. "Publish") as if it were a declared variable,
  and — not finding one — reported a second, misleading
  `P4012 Function block invocation is not a variable in scope` alongside
  the correct `P9004`. Needed an early return when `qualifier.is_some()`.
- Found and fixed an unrelated, pre-existing plc2plc bug while writing the
  round-trip test: `visit_fb_call` never wrote a trailing `;` for *any*
  FB call, qualified or not — presumably never caught because no prior
  test round-tripped a standalone FB-call statement.
- Confirmed a separate, still-open, pre-existing gap along the way:
  declaring a variable of an `INTERFACE` type at all (`fbComm : I_Comm;`)
  already produces its own diagnostic (`P2008`) regardless of this
  feature — from the earlier `INTERFACE` work, unrelated to and unaffected
  by this change.
- Full details in
  [2026-07-19-twincat-qualified-method-call-parsing.md](2026-07-19-twincat-qualified-method-call-parsing.md)'s
  Implementation Notes.

### 7. `STRING(n)`/`WSTRING(n)` parentheses + FB call-style instance init

Two unrelated sub-issues bundled in the original survey item, split apart
during implementation:

- **`STRING(n)`/`WSTRING(n)`**: parenthesis-delimited string length (e.g.
  `hostName : STRING(255);`) now accepted everywhere the bracket form
  (`STRING[255]`) already was (`VAR` declarations, `FUNCTION` return
  types) — no new dialect flag, matching the pre-existing, already-
  unconditional `string_type_declaration__parenthesis()` precedent for
  `TYPE`-alias declarations. Pure grammar addition (a shared
  `string_length_spec()` rule trying brackets then parens); no DSL or
  codegen changes needed since `StringSpecification`/`StringInitializer`
  never stored a bracket/paren marker in the first place. The plc2plc
  renderer always normalizes back to the bracket form (confirmed
  intentional, not a round-trip bug).
- **FB call-style instance init**: `comm : FB_Comm(retries := 3, THIS);`
  — passing an initialization parameter list directly after the type
  name, instead of the standard `:= (member := value, ...)` named-struct
  form — now parses under every dialect (no new flag; unambiguous syntax,
  same "no keyword to gate" reasoning as qualified method calls above).
  Reuses the exact same positional-or-named param shape as an ordinary FB
  call (`ParamAssignmentKind`), confirmed necessary by real test-corpus usage
  mixing both (`FB_CoverControl(comm := comm)` and
  `FB_CoverIdleState(THIS)`). `FunctionBlockInitialValueAssignment`
  gained a `call_params: Option<Vec<ParamAssignmentKind>>` field (5
  construction sites updated). Not wired into codegen — matches the
  pre-existing status of the standard `:=` form's own `init` field, which
  was *also* never read by codegen.
- **Found `fb_name_decl()` was pre-existing dead code**, unrelated to
  this feature but discovered while implementing it (via `peg`'s
  `trace`/`debug` cargo feature — `cargo build -p ironplc-cli --features
  trace` prints every grammar rule attempt to stderr, essential for
  debugging *why* a rule silently fails to match rather than just *that*
  it does). Root cause: the shared `commasep_oneplus()` combinator
  requires a spurious trailing comma after an already-complete
  comma-separated list, so it never actually matches real input. Every
  function block instance declaration in the codebase was already being
  handled by a different fallback path instead. **Deliberately did not
  fix `commasep_oneplus()`** — that fallback path's `LateResolvedType`
  deferral is load-bearing (it's *how* the parser avoids prematurely
  deciding whether a bare type name is an FB, struct, or enum before the
  type environment is built), so fixing the shared combinator would have
  changed parsing behavior for every bare FB declaration in the codebase.
  Added a new, narrowly-scoped rule instead
  (`fb_call_style_var_decl()`, requiring the call-style parens
  unconditionally, which makes it inherently unambiguous).
- **Found and fixed a second, unrelated pre-existing bug**: once the
  grammar fix made it possible to eagerly construct
  `InitialValueAssignmentKind::FunctionBlock` for a real, user-declared
  (not stdlib) FB type for the first time,
  `xform_toposort_declarations.rs` turned out to add the dependency edge
  in the wrong direction for that variant — backwards relative to the
  `Structure`/`LateResolvedType` arms and `EXTENDS`'s own edge in
  `visit_interface_declaration` (confirmed by reading all three side by
  side). This produced a spurious `P2011 Parent type is not declared`
  whenever the referenced FB type happened to need topological
  reordering. Never surfaced before because eager `FunctionBlock`
  construction was previously only reachable via `VAR_GLOBAL`
  declarations and `TYPE`-alias declarations, and no existing test
  exercised a forward reference through either of those paths. Fixed by
  flipping both edges (the dedicated visitor method and the inline match
  arm) to the correct direction; verified with the full workspace test
  suite plus two new targeted regression tests.
- Full details in
  [2026-07-19-twincat-sized-string-and-inline-fb-ctor.md](2026-07-19-twincat-sized-string-and-inline-fb-ctor.md)'s
  Implementation Notes.

### 8. Explicit enum-value assignment + enum base-type suffix

`E_ModeLanguage : (Deutsch := 1, English := 2);` (all members explicit,
matching the private test corpus) and `E_AssertionType : (Type_UNDEFINED := 0,
Type_ANY, Type_BOOL) BYTE;` (only the first member explicit + a base-type
suffix, matching `tcunit/TcUnit` and other cross-repo files) both now
parse, resolve to the correct runtime ordinals, and size correctly — no
new dialect flag (same "no keyword to gate, and the parser has no
mechanism to check `CompilerOptions` for non-keyword grammar changes
anyway" reasoning as `STRING(n)` parens and qualified calls).

- Unlabeled members continue from the previous resolved value + 1
  (ordinary C-style enum semantics), confirmed against Beckhoff's own
  documented example (`Red := 2, Green, Blue := 10` -> Green = 3) since
  no real file in the survey happened to contain a disambiguating gap.
- New shared helper, `ironplc_analyzer::resolve_ordinal_values()`, used
  by *both* the analyzer's sizing (`intermediates/enumeration.rs`) and
  codegen's ordinal map (`codegen::compile_enum::build_enum_ordinal_map`)
  so they can't disagree on what a member's actual value is. Sizing now
  uses the resolved max value, not raw member count — verified necessary
  with a real test (`(A := 300, B)`, 2 members, needs 2 bytes not 1).
- The base-type suffix (`) BYTE;`) was a new finding, not in the original
  survey — found while checking real files for a continuation-semantics
  disambiguating example. Included rather than filed separately because
  the file that originally motivated this whole survey item
  (`E_AssertionType.TcDUT`) uses it — without support, that exact file
  would still fail to parse, just further down.
- Found *another* pre-existing `fb_name_decl()`-shaped dead-code
  situation: a tuple-returning grammar helper
  (`enumerated_spec_init__with_values()`) became fully unused once its
  only caller was rewritten to thread the new base-type field through
  directly; deleted it. A *different*, already-known-dead rule
  (`enumerated_spec_init()`, unreferenced before this branch too) was
  left in place, just fixed to compile.
- Found and fixed two pre-existing, unrelated plc2plc renderer bugs while
  adding rendering for the new `explicit_value` field: there was no
  dedicated `visit_enumerated_value` override at all, so (1) a qualified
  enum value reference (`COLOR#RED`) silently rendered as `COLOR RED`
  (the `#` was dropped) regardless of this feature, and (2) the new
  `explicit_value` field itself — being `#[recurse(ignore)]` — would
  have been silently dropped by the default recursive visitor without an
  override, losing `:= 1`/`:= 2` entirely on any round-trip.
- Full details in
  [2026-07-19-twincat-explicit-enum-values.md](2026-07-19-twincat-explicit-enum-values.md)'s
  Implementation Notes.

### 9. Recursive project discovery

Not a grammar/dialect feature like the others — a fix to
`compiler/sources/src/discovery/mod.rs`'s `discover()`, which only
listed a directory's *immediate* children when searching for a
`.plcproj` (TwinCAT) or falling back to enumerating supported files.
Real TwinCAT layouts are Visual-Studio-style nested solutions — in a
private test corpus of real TwinCAT projects, the actual `.plcproj`
lives 2-3 levels below the directory a user would naturally point the
tool at
(`TestProject/TestProject/TestProjectRuntime/TestProjectRuntime.plcproj`,
not `TestProject/TestProjectRuntime.plcproj`), so `ironplcc check TestProject` reported "no
content" even though the project was right there, just deeper.

- Found via targeted verification (not part of the original survey):
  confirmed the exact failure mode against the real corpus, confirmed no
  recursive-walk mechanism existed anywhere in the discovery pipeline (no
  `walkdir` dependency, exactly 2 non-recursive `read_dir` calls total),
  and confirmed the fix by re-running the same real-corpus check before
  and after.
- Also verified, as a related but *separate* question (see the "Next"
  list's `P2008` item above for the full writeup): whole-project checking
  once files ARE discovered, already resolves cross-file FB/struct/enum
  type references correctly today — this fix is what was actually
  standing between the tool and being usable against real TwinCAT
  checkouts at all, not a symptom of deeper missing multi-file support.
- New shared `walk_files()` helper skips hidden directories (`.git`,
  `.idea` — both actually present in the corpus) to avoid wasteful/risky
  traversal, and doesn't follow symlinks (rules out cycles); used by both
  `detect_twincat()` and `detect_fallback()`.
- Recursion surfaces a real ambiguity the single-level code already had
  but could paper over: the corpus has **two** `.plcproj` files in the
  same directory (`TestProjectRuntime.plcproj` and `TestRuntime.plcproj`, an
  apparent stale rename artifact). Resolved with a simple, deterministic
  tie-break (sort candidates, take the first) rather than building
  disambiguation heuristics — no evidence more than this one coincidental
  case exists, and the simple tie-break happens to pick the "obviously
  correct" one here anyway.
- Found and fixed a latent, easy-to-miss correctness bug this exposed:
  `.plcproj`'s `<Compile Include="...">` paths are always relative to
  the `.plcproj` file's *own* directory, but the code was resolving them
  against the original directory passed to `discover()` — harmless
  before (the two were always the same path when `.plcproj` had to live
  directly in that directory) but silently wrong once nesting is
  possible. Caught by writing a test for a nested `.plcproj` referencing
  a file in its own subdirectory, before assuming the straightforward
  wiring was correct.
- Full details in
  [2026-07-19-twincat-recursive-project-discovery.md](2026-07-19-twincat-recursive-project-discovery.md)'s
  Implementation Notes.

### 10. `REFERENCE TO`/`POINTER TO` as alternate spellings of `REF_TO`

Prompted by an independent re-classification of a larger real-world
failure set (97 files, from a separate project using IronPLC as a
diagnostics backend, not this repo's own survey) — see "Re-scan
2026-07-20" below for the full breakdown and how it changed the
prioritization for what's still open.

- Neither `REFERENCE TO` nor `POINTER TO` was recognized at all before
  (parsed the first word as a plain identifier, then choked on the
  following `TO`). Both now produce the exact same
  `ReferenceTarget`/reference-initializer shape `REF_TO` already does,
  under the existing `--allow-ref-to` flag — no new flag, since these
  are alternate spellings of the same reference-type concept, not a
  separate feature.
- Verified against real files: 10 files use `REFERENCE TO`/`POINTER TO`
  exclusively targeting function-block types. Confirmed the deref-syntax
  split IEC 61131-3:2013 predicts actually holds in the corpus
  (`REFERENCE TO` accessed bare, `POINTER TO` accessed with `^`), and
  confirmed IronPLC doesn't enforce either convention today — `x.field`
  and `x^.field` both parse and analyze identically regardless of
  declared reference kind — so unifying all three spellings onto one DSL
  shape carries no semantic risk.
- Full details in
  [2026-07-20-twincat-reference-pointer-to.md](2026-07-20-twincat-reference-pointer-to.md).

### 11. `.plcproj` file-resolution no longer aborts the whole project on one bad entry

`parse_plcproj()` used to treat *any* single unresolvable
`<Compile Include="...">` entry as fatal for the entire project — it
returned `Err` on the first one found, before even attempting to
resolve the rest. One bad reference (a stale entry, a case-sensitivity
mismatch, a genuinely missing file) meant **zero** files from that
project ever got checked, even perfectly valid ones.

- Confirmed directly with a synthetic repro (a project with one valid
  file, one file with a real syntax error, and a `.plcproj` referencing
  a missing file): before the fix, the whole check aborted with a single
  `P6004` diagnostic and nothing else; after, the missing-file warning
  and the real syntax error both print, and the valid file passes
  silently — confirming the fix doesn't swallow genuine per-file
  diagnostics alongside the new warning.
- `DiscoveredProject` gained a `warnings: Vec<Diagnostic>` field;
  `parse_plcproj` now collects each unresolvable entry as a warning and
  `continue`s instead of aborting. The `.plcproj` file itself being
  unreadable or malformed XML are still hard errors (nothing to resolve
  at all in either case) — only the per-entry resolution failure changed.
  `enumerate_files`/`create_project` in the CLI surface these warnings
  via the existing `handle_diagnostics` helper, non-fatally.
- Deliberately not a case-insensitive filesystem fallback — that would
  silently paper over a real project-file mismatch instead of surfacing
  it as a (non-fatal) warning.
- Prompted by the same 97-file re-classification as item #10 above; see
  "Re-scan 2026-07-20" below for the original finding (27 files in a
  different project's corpus had never actually been checked at all
  because of this).
- Full details in
  [2026-07-20-twincat-plcproj-resolution-warnings.md](2026-07-20-twincat-plcproj-resolution-warnings.md).

### 12. `ABSTRACT` function block keyword

`FUNCTION_BLOCK ABSTRACT <name> ...` didn't parse at all — `ABSTRACT`
wasn't a registered keyword, so it was consumed as the function block's
own *name* by `derived_function_block_name()` (a bare identifier rule),
which is why the *real* name that follows, and any `EXTENDS`/`IMPLEMENTS`
clause after it, then failed to parse further down. One root cause, not
two: the original survey counted "plain `ABSTRACT`" and "`ABSTRACT` +
`EXTENDS`/`IMPLEMENTS`" as separate 7-file buckets, but both fail for the
exact same reason and are fixed by the same change.

- New `Abstract` token, demoted to `Identifier` under the existing
  `allow_oop_extensions` flag alongside `EXTENDS`/`IMPLEMENTS`/
  `INTERFACE`/`END_INTERFACE` — not a new flag, since every real-world
  `ABSTRACT` usage found co-occurs with `EXTENDS`/`IMPLEMENTS`.
- New `is_abstract: bool` field on `FunctionBlockDeclaration`, flagged by
  the existing `rule_unsupported_extension.rs`/`P9004` mechanism the
  exact same way `extends`/`implements` already are — recognized, not yet
  semantically enforced (no check that an abstract function block is
  never directly instantiated).
- Verified end-to-end via the CLI (`--dialect=codesys`): a function block
  combining all three (`ABSTRACT ... EXTENDS ... IMPLEMENTS ...`) now
  parses correctly and produces exactly one `P9004` for the FB (plus a
  separate one for the `INTERFACE` it references), confirming the single
  fix resolves both halves of the original survey bucket.
- Full details in
  [2026-07-20-twincat-abstract-keyword.md](2026-07-20-twincat-abstract-keyword.md).

### 13. `LTRUNC`/`LMOD` extended math functions (Beckhoff `Tc2_Math` library)

Both were undeclared function calls (`P4017`). The originating survey
assumed they were plain missing entries in the generic, always-on
`TRUNC`/`MOD` table — **that assumption didn't survive checking
Beckhoff's own `Tc2_Math` documentation directly before implementing**,
the same kind of correction already made this session for the
EXTENDS/IMPLEMENTS bucket and for `PI`.

- `LTRUNC` (`FUNCTION LTRUNC : LREAL`, param `lr_in : LREAL`) truncates
  the fractional part like `TRUNC`, but returns `LREAL` rather than
  `ANY_INT` — not clamped to an integer type's value range.
- `LMOD` (`FUNCTION LMOD : LREAL`, params `lr_Value`/`lr_Arg : LREAL`) is
  a floating-point modulo (Beckhoff's own example:
  `LMOD(400.56, 360) = 40.56`), unlike the integer-oriented `MOD`.
- Both are `Tc2_Math` library functions — a specific, named Beckhoff PLC
  library a TwinCAT project must reference — not core IEC 61131-3, and
  not generic over `ANY_REAL`/`ANY_NUM` the way `TRUNC`/`MOD` are (they
  operate on `LREAL` only). Registered via a new
  `allow_extended_math_functions` flag, gated the same way `SIZEOF`
  already is (conditional registration in `stages.rs`), not folded into
  the always-on core arithmetic tables.
- `NCError_TO_STRING`, in the same 5-file survey bucket, is a
  project-local function, not a stdlib gap — explicitly out of scope
  here.
- `CompilerOptions`'s `define_compiler_options!` macro auto-generates the
  CLI/LSP/MCP wiring from one table entry, so adding the flag itself
  caused no exhaustive-match ripple — but 3 hardcoded flag-count
  assertions in `options.rs` and 2 more in `mcp/src/tools/list_options.rs`
  still needed manual bumps, found by running the full test suite (not by
  grep). Same silent-drift hazard already documented in "Rebase conflict
  resolution reference" above for the stack's earlier branches.
- Full details in
  [2026-07-20-twincat-ltrunc-lmod.md](2026-07-20-twincat-ltrunc-lmod.md).

### 14. `EXTENDS` field inheritance for function blocks

Unqualified references to a field declared only on a base class (via
`EXTENDS`) previously failed with `P4007 Undefined variable` — scoping
(`rule_use_declared_symbolic_var.rs`) and expression type resolution
(`xform_resolve_expr_types.rs`) only ever considered a function block's
own fields, never its ancestor chain. This was explicitly deferred
earlier (see the re-scan below) pending a design discussion; that
discussion happened 2026-07-20 and settled one central question before
implementation started.

- New shared `collect_inherited_fields()` helper (in a new
  `intermediates/inherited_fields.rs`), transitive across multi-level
  `EXTENDS` chains, with derived-class fields shadowing a same-named
  ancestor field. Wired into both consumers that need "which fields are
  visible here" per function block.
- Also found and fixed an adjacent gap while implementing this:
  `xform_toposort_declarations.rs`'s `visit_interface_declaration`
  already added a toposort edge for its own `EXTENDS` (interface
  extends interface), but the parallel `visit_function_block_declaration`
  added **no edge at all** for its own `EXTENDS` — meaning an `EXTENDS`
  cycle between function blocks wasn't caught by anything, and there was
  no ordering guarantee that a base class is processed before its
  derived class. Added the missing edge, in parity with the interface
  case — gives cycle detection (via the existing `RecursiveCycle`
  diagnostic) and correct ordering for free.
- **Explicit design decision, made before implementation**: since field
  inheritance is now fully resolved, a plain `EXTENDS` (no `IMPLEMENTS`,
  not `ABSTRACT`) no longer produces `P9004` — there's nothing left
  unsupported for that specific shape. `IMPLEMENTS` (interface dispatch)
  and `ABSTRACT` (instantiation-legality enforcement) still flag, alone
  or combined with `EXTENDS`. Without this change, the 25 files that
  motivated this work would still show `P9004` even after the underlying
  field-resolution bug was fixed.
- Deliberately out of scope, unaffected by this change: method dispatch,
  `METHOD`/`PROPERTY` bodies (still rejected outright at parse time),
  `THIS^`/`SUPER^`, qualified *external* field access on an FB instance
  (`instance.field` reading an ordinary, non-inherited field from
  outside — a separate, pre-existing gap: `xform_resolve_expr_types.rs`'s
  `resolve_struct_type` only recognizes `IntermediateType::Structure`,
  not `IntermediateType::FunctionBlock`, so this doesn't work even with
  zero inheritance involved).
- Full details in
  [2026-07-20-twincat-extends-field-inheritance.md](2026-07-20-twincat-extends-field-inheritance.md).

### 15. `AT`-located `ARRAY` variable in a mixed `VAR` block

`outputs AT %Q* : ARRAY[0..9] OF BOOL;` already parsed fine alone in its
own dedicated `VAR ... END_VAR` block, but failed with a `P0002` syntax
error when mixed alongside plain variables in the same block. This
bucket was in the original 97-file re-scan table but hadn't been
triaged into the numbered "Next" priority list at all until now.

- Root cause, confirmed directly with two repros: the dedicated-block
  path (`incompl_located_var_decl()` -> `var_spec()`) already included
  `array_specification()`, but the mixed-block path
  (`located_var1_init_decl()`, added by the earlier
  mixed-located-var-declarations feature) had no `ARRAY` alternative in
  its init rule at all.
- Fix: add `array_spec_init()` as an alternative to
  `located_var1_init_decl()`, mirroring the dedicated-block path's own
  combined shape (`located_var_spec_init()`). No other changes needed --
  `UntypedVarDecl`'s location handling, the mixed-block semantic gate,
  and the plc2plc renderer were all already shape-agnostic.
- Full details in
  [2026-07-20-twincat-located-array-mixed-block.md](2026-07-20-twincat-located-array-mixed-block.md).

### 16. LSP: merge multiple workspace folders into one compilation unit

`ironplcc lsp` previously only ever analyzed the *first* LSP workspace
folder a client sent on `initialize` (`lsp.rs`'s `folders.first()`),
silently dropping the rest -- a multi-sub-project TwinCAT solution
needs all of them loaded together for cross-project type resolution to
work. Not from the private-corpus survey at all -- motivated by an
external editor plugin using `ironplcc` as an LSP diagnostics backend.

- Drafted as a plan first, then implemented in the same pass once
  priority was confirmed (see the plan doc's own "Status" line).
- One extra finding beyond the original plan, caught before writing any
  code: `SourceProject::initialize_from_directory` clears sources
  before discovering, so naively looping over all folders and calling
  the existing single-directory `initialize()` on each would *not* have
  merged them -- it would have made the *last* folder win, not all of
  them combined. The fix needed a real accumulating multi-directory
  path (clear once, discover-and-add per directory), not just a loop
  over the existing method.
- New `SourceProject::initialize_from_directories()` and a matching
  `Project::initialize_many()` trait method (default delegates to
  single-directory `initialize` for 0/1 directories, errors for more
  than one unless overridden -- deliberately not a "call `initialize`
  per directory" default, since that would silently reproduce the same
  clearing bug for any future implementor whose `initialize` clears
  state per call).
- Mirrors a pattern that already existed and worked: the CLI's
  multi-argument `ironplcc check dir1 dir2 ...` (`cli.rs`'s
  `create_project`) already merges multiple directories into one
  project the same way.
- Verified end-to-end, not just that files get merged into a list: a
  test wires two real temp directories (one declaring a function block,
  the other referencing it by type name) through the real
  `LspProject::initialize_many` -> `semantic_all()` path, confirming the
  cross-folder type reference resolves with zero diagnostics once both
  folders are loaded together.
- Full details in
  [2026-07-20-twincat-lsp-multi-workspace-folder.md](2026-07-20-twincat-lsp-multi-workspace-folder.md).

### 17. `MODABS` stdlib function (Beckhoff `Tc2_Math` library)

`MODABS(x, y)` was undeclared (`P4017`) -- the single biggest
remaining one-fix win in a fresh re-scan pass (see "Re-scan 2026-07-20,
second pass" below). Same family as `LTRUNC`/`LMOD`: a Beckhoff
`Tc2_Math` library function, `LREAL`-only, not core IEC 61131-3.

- Verified against Beckhoff's own docs before implementing: `MODABS` is
  like `LMOD` but always returns an unsigned/non-negative modulo result
  (`MODABS(-400.56, 360) = 319.44`, where `LMOD` with the same
  arguments would return `-40.56`) -- used in NC-axis contexts where
  modulo values are conventionally unsigned.
- Added to the existing `get_extended_math_functions()` table, gated by
  the already-landed `allow_extended_math_functions` flag -- no new
  flag, no new registration mechanism, just one more signature in the
  already-iterated `Vec`.
- Full details in
  [2026-07-20-twincat-modabs.md](2026-07-20-twincat-modabs.md).

### 18. `AND_THEN` short-circuit boolean operator

`AND_THEN` was an unrecognized token (parse error) -- a genuine
Beckhoff/CODESYS extension, verified against Beckhoff's own docs: a
short-circuit `AND` that only evaluates its right operand when the left
is `TRUE`, commonly used to guard a dereference
(`ptr <> 0 AND_THEN ptr^ = 99`).

- Kept as a distinct `CompareOp::AndThen` variant rather than folding
  into `CompareOp::And` (unlike the `REFERENCE TO`/`POINTER TO` ->
  `REF_TO` unification) -- the short-circuit vs. eager evaluation
  difference is real and externally-visible in TwinCAT/CODESYS itself,
  so normalizing it away on render would not be behavior-preserving for
  a real downstream toolchain even though IronPLC's own analysis
  doesn't model the difference.
- `ironplcc check` fully supports `AND_THEN` -- parsing, type-checking,
  round-tripping with its spelling preserved. Codegen doesn't implement
  short-circuit (conditional-branch) evaluation for any boolean operator
  today, so rather than silently emit eager (behaviorally wrong, unsafe)
  bytecode, compiling an `AND_THEN` expression now fails explicitly with
  `P9999` instead of miscompiling.
- `cargo build`'s exhaustiveness checking found most affected match
  sites automatically, but two already had a wildcard catch-all arm
  (compiling *without* error) that would have silently routed
  `AndThen` through the wrong branch if not found and fixed by hand --
  a reminder that exhaustiveness-driven discovery only catches match
  sites without a wildcard, worth a direct grep as a second pass.
- Full details in
  [2026-07-20-twincat-and-then-operator.md](2026-07-20-twincat-and-then-operator.md).

### 19. `CASE` branch with no statements

A `CASE` branch whose body has zero statements (a label that falls
through to nothing -- just a comment, or a genuine placeholder) failed
to parse, with the error confusingly located at the *next* case label
rather than the actual empty branch.

- The original survey framing ("bare-integer `CASE` label") was
  misleading -- tested directly first and found a bare integer label
  parses fine on its own, including with a comment and the real
  statement on the next line. The actual failure only appears when a
  branch has *no statement at all* before the next label.
- Root cause: `case_element()`'s statement portion is `statement_list()`,
  which is `statements_or_empty()+` -- a one-or-more repetition. An
  empty branch has nothing for the `+` to match, so `case_element()`
  fails entirely and backtracks; the parser then tries (and fails) to
  interpret the next label's bare integer as a statement for the empty
  previous branch, producing the confusingly-located error.
- Verified as a real gap against actual TcXaeShell before implementing:
  three synthetic files (a normal populated `CASE`, an empty branch
  followed by another label, an empty branch as the last one before
  `END_CASE`) all compiled clean in real TwinCAT.
- New `statement_list_or_empty()` fallback rule, used only by
  `case_element()`. The plc2plc renderer already had explicit handling
  for an empty statement group -- someone had anticipated this shape on
  the render side already, even though the parser couldn't produce it
  until this fix.
- Deliberately not extended to `CASE`'s own `ELSE` clause or to
  `IF`/`FOR`/`WHILE`/`REPEAT` bodies, which share the same underlying
  `statement_list()`-requires-one-or-more gap -- not part of the
  verified survey item, no evidence any real file needs it yet.
- Full details in
  [2026-07-20-twincat-empty-case-branch.md](2026-07-20-twincat-empty-case-branch.md).

### 20. Reject a derived FB redeclaring a base-class field via `EXTENDS` (`P4039`)

A `FUNCTION_BLOCK ... EXTENDS Base` that redeclares a field already
present in `Base` (same name, any type) now produces an error instead of
silently accepting it. Verified against a real TcXaeShell compile first
(`C0097: Duplicate definition`) — this and the next two items came from a
batch of full, directly-importable `.TcPOU`/`.TcDUT` test files generated
for empirical testing of already-"Done" `EXTENDS`/`ABSTRACT`/
`REFERENCE TO` support, not from a fresh survey item.

- Reused the existing `collect_inherited_fields()` helper (from "Done"
  #14's field-inheritance work) unmodified as the basis for the new
  check — no new traversal needed.
- Full details in
  [2026-07-20-twincat-extends-duplicate-field.md](2026-07-20-twincat-extends-duplicate-field.md).

### 21. Reject direct instantiation of an `ABSTRACT` function block (`P4040`)

A `VAR` declaring an instance of an `ABSTRACT`-qualified function block
now produces an error; instantiating a concrete subclass of it still
works. Verified against a real TcXaeShell compile (`C0434`) as part of
the same test batch as #20 above.

- Deliberately scoped to avoid threading `is_abstract` through
  `IntermediateType::FunctionBlock` (10+ construction sites across
  `type_environment.rs`/`intermediate_type.rs`/
  `xform_resolve_type_decl_environment.rs`) — works directly off the raw
  AST instead (a `HashSet<TypeName>` of abstract FB names, checked
  against each `VarDecl`'s already-resolved function-block initializer),
  avoiding a much larger, unrelated ripple for a narrowly-scoped check.
- Full details in
  [2026-07-20-twincat-abstract-instantiation.md](2026-07-20-twincat-abstract-instantiation.md).

### 22. Reject explicit `^` dereference on a `REFERENCE TO` variable (`P2037`)

`REFERENCE TO` is the genuine IEC 61131-3:2013 reference type: it
auto-dereferences, so `r^` is now rejected (only bare `r` is allowed at
the access site); `REF_TO`/`POINTER TO` are unaffected and still require
`^`. Verified against a real TcXaeShell compile (`C0032`/`C0064`) as part
of the same test batch as #20/#21 above — a genuine gap in "Done" #10,
which had unified all three spellings into one AST shape with no way to
tell them apart at the access site.

- New `ReferenceKeyword` enum (`RefTo`/`Reference`/`Pointer`) tracks
  which spelling declared a reference-typed `VAR`, added to
  `ReferenceInitializer`. Scoped to just the verified case (a direct
  `VAR ... : REFERENCE TO ...;` declaration) — `TYPE` aliases and inline
  array-element `REFERENCE TO` types are left untouched.
- Surfaced a real, pre-existing tension with plc2plc's renderer, which
  already normalizes `REFERENCE TO`/`POINTER TO`/`REF_TO` all to
  `REF_TO` on render (a deliberate prior choice, unrelated to this fix):
  round-tripping a `REFERENCE TO` variable through plc2plc silently
  produces `REF_TO` text, where `^` is allowed again. A pre-existing
  plc2plc round-trip test asserted full AST equality across a
  render-then-reparse cycle and broke on the new field for exactly this
  reason; fixed by asserting render idempotency instead of AST equality,
  matching the style already used by neighboring "normalizes to REF_TO"
  tests. Not otherwise resolved — flagged as a known, low-priority
  follow-up (no evidence yet that real `REFERENCE TO` code round-trips
  through plc2plc today).
- Full details in
  [2026-07-20-twincat-reference-to-no-explicit-deref.md](2026-07-20-twincat-reference-to-no-explicit-deref.md).

### Re-scan 2026-07-20: 97-file re-classification (from a separate project's corpus)

A different project using IronPLC as a diagnostics backend independently
re-ran the same kind of whole-project survey against its own private
TwinCAT corpus, using `ironplcc check` per-project (not per-file) once
the recursive-discovery fix (above) made that possible. Two findings
worth carrying forward here, even though the analysis itself lives
outside this repo:

- **A `.plcproj` file-resolution gap — done, see "Done" #11 below.**
  `ironplcc` used to treat *any* single unreadable `<Compile Include="...">`
  entry as fatal for the whole project (aborts before checking anything
  else) — confirmed via two real cases (a case-sensitivity mismatch
  between the `.plcproj` entry and the actual filename on a
  case-sensitive filesystem, and a genuinely missing referenced file).
  27 files across 2 sub-projects in that corpus had never actually been
  checked at all because of this, not because they passed or failed on
  their own merits. Fixed by recording each unresolvable entry as a
  warning and skipping it rather than aborting — not a case-insensitive
  fallback (that would silently paper over a real project-file mismatch
  instead of surfacing it).
- **Re-classification of the (then-)97 verified-still-failing files**,
  ranked by impact:

  | Count | Code | Construct | Status |
  |---|---|---|---|
  | 25 | P9004 | Plain `EXTENDS`/`IMPLEMENTS` | **Done — see "Done" #14 above.** Turned out to need real semantic work (inheritance-aware field resolution), not just lifting the flag -- fixed after an explicit design discussion settled the P9004-gating question. |
  | 19 | P2008 | Cross-file type resolution | Mixed: some are genuinely external Beckhoff-library types with no source in the corpus at all (`MC_Home`, `AXIS_REF`, etc. — Motion/System libraries); at least one is a case-sensitivity resolution bug (`FB_EventLog` vs `FB_Eventlog`); at least one more fails to resolve despite matching declared case in the same sub-project, not yet root-caused. |
  | 10 | P0002 | `REFERENCE TO`/`POINTER TO <FB-type>` | **Done — see "Done" #10 above.** |
  | 27 | P6004 | `.plcproj` file-resolution abort (not one of the 97 — these were never even checked) | **Done — see "Done" #11 above.** |
  | 7 | P0002 | `FUNCTION_BLOCK ABSTRACT ... EXTENDS/IMPLEMENTS` | **Done — see "Done" #12 above.** |
  | 5 | P4017 | Undeclared function call | `LTRUNC`/`LMOD` **done — see "Done" #13 above.** `EXPT`-adjacent math variants not yet investigated; `NCError_TO_STRING` is project-local, not a stdlib gap. |
  | 3 | P0002 | `AT%Q*`-located variable with an `ARRAY` type | **Done — see "Done" #15 above.** |
  | 2 | P0002 | `^.` (deref + member access) inside a `VAR` initializer | Not yet investigated. |
  | 23 | P0002 | Misc, scattered | No shared construct; some likely cascading from an earlier unparsed line in the same file. |
  | 3 | P4012/P4007/P4035 | Singles | Not dug into individually. |

### Re-scan 2026-07-20, second pass (from the same separate project's corpus)

A follow-up pass over the same private TwinCAT corpus, after the fixes
above landed, surfaced a fresh, smaller batch of still-failing files.
Ranked by file count as given, though count doesn't necessarily track
cost (see the standing lesson above):

| Count | Bucket | Status |
|---|---|---|
| 6 | `MODABS` undeclared | **Done — see "Done" #17 above.** |
| 3 | `AND_THEN` operator | **Done — see "Done" #18 above.** |
| 3 | `THIS^.Method()` | Calling a method via an explicit `THIS^` pointer-dereference. Newly found this pass; not yet investigated against the qualified-call-parsing work already landed (see "Done" #6) -- unclear yet whether `THIS^` needs its own grammar support or composes with the existing qualified-call path. |
| 3 | Bare-integer `CASE` label starting a branch (e.g. `5: // comment`) | **Done — see "Done" #19 above.** Turned out to be about empty branch bodies, not bare integer labels. |
| 2 | `^.` (deref + member access) inside a structured/call-style `VAR` initializer | A concrete real shape: `tonDelta : TON := (PT := pDevice^.Delta);` -- narrower than originally framed (not a plain `:=` simple initializer, which already routes through `expression()` and would parse `^.` fine; specifically the *structured/call-style* initializer position, e.g. `ref_to_var_init_decl()`/similar, which may use a narrower initializer grammar). Investigation was in progress when superseded by this second-pass table; picking back up needs confirming exactly which initializer rule the real shape goes through. |
| 1 | Undeclared function call, actually a case-sensitivity bug (e.g. declared `SOME_FUNCTION`, called `some_function`) | **Distinct from the `FB_EventLog`/`FB_Eventlog` type-resolution case already re-verified as a non-issue** (see the P2008 sub-item write-up above) -- that one was about `TypeName`/`Id` (already case-insensitive by design). This is about *function call* resolution instead, a different lookup path (`FunctionEnvironment`) that hasn't been checked for the same case-insensitivity guarantee. Worth verifying directly before assuming it's broken, then scoping as its own small, general fix (not a one-off) if it is. |

## Key design decisions (apply to future work in this area too)

- **Dialect placement**: CODESYS/TwinCAT-shared vendor extensions go on the
  existing `Rusty`/`Codesys` dialects, not a new `BeckhoffTwinCAT` dialect —
  unless/until a construct is found that's genuinely TwinCAT-only and
  wouldn't make sense on plain CODESYS. So far nothing has required that split.
- **Keyword gating**: use the codebase's actual **demotion** pattern
  (`xform_demote_*.rs` — always lex as the specific token, demote to
  `Identifier` when the flag is off), not the "promotion" pattern originally
  proposed in `specs/design/beckhoff-twincat-dialect.md` (that doc predates
  the demotion convention).
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
  tools — though it's usually smaller than it looks, since most of those
  sites use `if let`/wildcards, not exhaustive matches. `cargo build`
  surfaces the real exhaustive-match list precisely; don't try to
  enumerate it by hand in a plan.
- **Verify against real files before implementing**, not just before
  finishing. The `.TcIO` discovery came from checking a private local
  checkout of a real TwinCAT codebase *during* plan-writing, before any
  code was written — cheaper than finding it after implementation.
- **A survey's file-count ranking is not a cost estimate.** `PI` looked like
  the cheapest, highest-leverage item on paper (18 files, "pure
  registration"). Testing the actual real-file *pattern* against the parser
  — not just grepping for the construct's name — revealed the true blocker
  was something else entirely. Do this check *before* writing "cheap" into
  a plan's goal statement, not after implementation starts.
- **"Normalize away early" for AST changes with a wide blast radius.** When
  a change would otherwise touch a type used in N places (here: 21 files
  touch `SimpleInitializer`), check whether the codebase already has a
  "parsed-but-unresolved placeholder, normalized by a dedicated early pass"
  precedent to follow instead (here: `IntegerRef::Constant`/`LateBound`).
  Introducing a new placeholder variant that a single pass always resolves
  back to the existing shape keeps the blast radius near zero, instead of
  changing the shared type itself.
- **Check ADRs before assuming a runtime-evaluated fix is viable.**
  ADR-0024 forced the initializer-expression design into "must fully
  constant-fold at compile time" rather than "defer to runtime" — this
  wasn't discoverable from the parser/AST alone, only from the ADR
  explaining *why* the init-template mechanism was chosen over a bytecode
  prologue.
- **In this `peg`-based grammar, "try A, fall back to B" only works if A
  *fails outright* on the fallback cases.** If A can *partially* match and
  return successfully (e.g. `constant()` matching just the literal prefix
  of `3.14/180.0`), PEG's ordered choice locks in that match and never
  tries B, even though a later required token then fails to parse. The fix
  is to parse via the more general rule (B) unconditionally and dispatch on
  the *shape* of the result, not to chain "specific then general"
  alternatives hoping the specific one fails cleanly when it shouldn't
  match. Cost extra: this can silently change how *other*, unrelated
  constructs parse (here: negative literals wrapped in `UnaryOp` instead of
  parsing directly to a negative `Const`) — always run the full workspace
  test suite after such a change, not just tests for the new construct.
- **When a grammar rule silently fails to match and it's not obvious why,
  use `cargo build -p ironplc-cli --features trace` (the `peg` crate's
  built-in tracing) before guessing.** It prints every rule attempt/match/
  fail to stderr with source position, which is how the pre-existing
  `commasep_oneplus()`/`fb_name_decl()` dead-code bug (see "Done" #7) was
  actually found — reading the grammar alone made the rule look correct.
- **Dependency-graph edge direction conventions must be checked against
  a working example, not assumed from one arm's code.** Before adding a
  new toposort visitor arm for a variant that hadn't been eagerly
  constructed before, compare its edge direction against an existing,
  actually-exercised arm for the same "referenced type must be ordered
  before referencing POU" relationship (here: `Structure`/
  `LateResolvedType` and `visit_interface_declaration`'s `EXTENDS` edge)
  rather than trusting that an existing-but-rarely-exercised arm (the
  `FunctionBlock` one) already had it right.

## Next

**Re-sorted 2026-07-20, simplest first**, after directly verifying (not
guessing) the complexity of each item. Item 1 (`.plcproj`
file-resolution), the former item 1 (`ABSTRACT` keyword), the former
item 1 (`LTRUNC`/`LMOD`), the former item 2 (`EXTENDS`/`IMPLEMENTS`
field-inheritance resolution), the LSP multi-workspace-folder item,
`MODABS`, `AND_THEN`, and the bare-integer-`CASE`-label item (which
turned out to be about empty branch bodies, not bare integer labels)
are now all done — see "Done" #11, #12, #13, #14, #16, #17, #18, and #19
above. The `AT %Q*`-located-`ARRAY` bucket (also done, see "Done" #15)
wasn't in this numbered list at all until it was triaged and fixed in
the same pass. **Items 1-3 below are from the second-pass re-scan**
(see that section above) and have not yet had their complexity directly
verified the way earlier items were before implementation -- treat
their "likely simple"/"not yet verified" framing as a starting estimate,
not a confirmed one.

**Former item 1, `located_var_declarations()` (complete addresses) not
reachable from `FUNCTION_BLOCK`/`FUNCTION`, turned out to already be a
non-issue on re-verification (2026-07-20) — nothing to implement:**
`FUNCTION_BLOCK` already fully supports complete-address `AT`-located
variables (e.g. `sensor AT %IX0.0 : BOOL;`) today, unconditionally, under
every dialect — a side effect of the earlier "mixed located/plain `VAR`
declarations" fix (`var_init_decl()` -> `located_var1_init_decl()`,
which handles both `location()` and `incompl_location()`) that nobody had
cross-checked against this separately-tracked item. Confirmed directly:
`FUNCTION_BLOCK FB_Example VAR sensor AT %IX0.0 : BOOL; END_VAR
END_FUNCTION_BLOCK` parses cleanly with no flags (an all-located block is
standard syntax, unaffected by the mixed-block gating). `FUNCTION`
genuinely has zero located-variable support (confirmed via a parse-error
repro), but that's very likely *correct* — functions are stateless per
IEC 61131-3, so direct hardware I/O binding doesn't make sense for
something with no persistent state between calls, and no real
CODESYS/TwinCAT PLC is known to allow it either. Not implementing
anything here.

1. **Undeclared function call, actually a case-sensitivity bug** (e.g.
   declared `SOME_FUNCTION`, called `some_function`) — **Not yet
   verified.** Distinct from the `FB_EventLog`/`FB_Eventlog`
   type-resolution case already re-verified as a non-issue (`TypeName`/
   `Id` are already case-insensitive) -- this is about *function call*
   resolution (`FunctionEnvironment`), a different lookup path not yet
   checked for the same guarantee. Verify directly before assuming it's
   broken; if it is, scope as its own small, general fix (case-
   insensitive function lookup), not a one-off.
2. **`^.` (deref + member access) inside a structured/call-style `VAR`
   initializer** — **Partially investigated, narrower than originally
   framed.** Real shape: `tonDelta : TON := (PT := pDevice^.Delta);` --
   not a plain `:=` simple initializer (which already routes through
   `expression()` and parses `^.` fine), but specifically the
   *structured/call-style* initializer position. Investigation was
   in progress (checking `ref_to_var_init_decl()` and similar rules)
   when superseded by a fresh re-scan; needs confirming exactly which
   initializer rule the real shape goes through before estimating cost.
3. **`THIS^.Method()`** — **Not yet investigated.** Calling a method via
   an explicit `THIS^` pointer-dereference. Newly found; unclear whether
   it needs its own grammar support or composes with the existing
   qualified-call-parsing work (see "Done" #6) once investigated. Note
   `THIS^`/`SUPER^` are also listed under item 5's (Full OOP) scope --
   this item is scoped narrower (just the `THIS^` deref-then-call
   parsing shape, not full dispatch semantics), so investigate whether
   it can land independently before assuming it's blocked on Full OOP.
4. **`P2008` remaining pieces, split apart** — re-assessed 2026-07-20
   (was a single mixed bucket; not structural, once files are discovered
   together, cross-file type resolution for ordinary FB/struct/enum
   references already works correctly today —
   `run_semantic_analysis` in `compiler/project/src/project.rs` already
   feeds every discovered file into one shared `analyze(&all_libraries,
   ...)` call, not per-file isolation):
   1. **Case-sensitivity resolution bug** (`FB_EventLog` vs
      `FB_Eventlog`) — **Re-verified 2026-07-20, already resolved, no
      code needed.** Checked both plausible causes directly before
      assuming a fix was needed: (a) same-project, both files parsed,
      type referenced with different case than declared — already
      resolves cleanly, since `TypeName`/`Id` implement case-insensitive
      `Hash`/`Eq` by design (per the IEC 61131-3 spec) and always have;
      (b) a `.plcproj`-referenced filename not matching the actual
      file's case on disk — reproduced directly, and the just-landed
      `.plcproj` resolution-warnings fix (see "Done" #11) already
      surfaces this correctly: a clear `P6004` pointing at the exact
      missing filename, plus the resulting `P2008` for whatever
      references that now-undeclared type, with the rest of the project
      still checked. A case-insensitive filesystem fallback was already
      explicitly ruled out as a non-goal when #11 landed (would silently
      paper over a real project-file mismatch), so there's nothing
      further to build here.
   2. **Genuinely external Beckhoff-library types with no source in the
      corpus** (`MC_Home`, `AXIS_REF`, etc. — Motion/System libraries) —
      **Not fixable from this side.** No source to resolve against;
      would need stub/declaration-only registration of the relevant
      library types (a different, much larger effort: modeling an
      external library's public surface, not a bug fix) if ever pursued.
   3. **One same-project resolution gap, not yet root-caused** — matches
      declared case in the same sub-project but still fails to resolve.
      **Blocked** — only reproduces in the private corpus, which isn't
      accessible for direct investigation; needs a synthetic repro
      before it can be estimated or fixed at all.

   **All three sub-items are now either resolved, structurally
   unfixable, or blocked on a repro that isn't available** — this bucket
   has no further actionable work right now.
5. **Full OOP** (`METHOD`/`PROPERTY` bodies, access modifiers, inheritance
   *dispatch*, `THIS^`/`SUPER^`) — **Complex.** Big, multi-PR effort.
   Field-inheritance resolution is done (see "Done" #14) — what's still
   missing is `METHOD` body parsing itself (currently rejected outright)
   and real dispatch/codegen. Phase 1+ of
   `specs/design/beckhoff-twincat-dialect.md`, but that doc needs
   reconciling with the actual demotion-pattern/dialect-flag decisions made
   above before starting. Qualified *call-site* parsing (`instance.Method(...)`)
   is also already done (see "Done" #6).
6. **Namespace-qualified identifiers** (`SysFile.ACCESS_MODE`,
   `EXTENDS TcUnit.FB_TestSuite`, `GVL.MaxCount` as an array bound) —
   **Complex.** Not in the private test corpus at all, so **not
   prioritized ahead of the items ranked above** per explicit direction,
   but flagged here because it's the single biggest gap found across the
   5 external repos checked (82/491 files, ~17%) — bigger than any item
   still on this list. Revisit if a future project needs
   multi-namespace/library support, or once the items ranked above are
   exhausted. Not investigated beyond confirming the parse failure (`.`
   unexpected after an identifier used as a type name, a constant
   reference, or an `EXTENDS` target).

## How to resume from another computer

```sh
git clone git@github.com:thusser/ironplc.git
cd ironplc
git checkout twincat-dev
cd compiler && just   # should pass end-to-end
```

Read this file, then the plan docs linked in "Branches" above for full
design detail and the "Implementation Notes" sections in each for
decisions resolved during coding (not just planning).
