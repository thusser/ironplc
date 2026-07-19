# Plan: Implicit `PI` Math Constant

> **STATUS: paused mid-implementation, superseded by a bigger finding.**
> While implementing this, testing against the actual real-file pattern
> (`d2r : LREAL := PI/180.0;`) surfaced that this is a **syntax error today,
> independent of whether `PI` resolves**: `VAR` initializers in IronPLC only
> accept a single literal constant (`d2r : LREAL := 3.14;` parses), not any
> expression — confirmed both arithmetic (`3.14 / 180.0`) and a bare
> identifier reference (`SOME_CONST`) fail to parse as a `VAR` initializer.
> Statement-context usage (`x := PI/180.0;`) already works fine — expressions
> are unrestricted there.
>
> Re-checking the real files: of the ~18 `PI`-as-bare-identifier files, only
> one (`ATAN2.TcPOU`) plus a few lines in `FB_NUTATE.TcPOU`/`FB_IAU2000B.TcPOU`
> use `PI` in statement context. The overwhelming majority use the
> `VAR ... := PI/180.0;` initializer pattern. **So registering `PI` alone
> would fix ~2 files, not 18** — the real blocker for the rest is "`VAR`
> initializers only accept literal constants, not expressions," a bigger and
> previously-unidentified gap that's arguably higher-leverage than `PI`
> itself (it would unblock *any* computed-constant initializer, not just
> `PI`-based ones). See `specs/plans/2026-07-19-twincat-var-initializer-expressions.md`
> for that investigation (started next, per user decision to pivot
> immediately rather than land this first).
>
> The design below (the `VarDecl`/`GlobalVarDeclarations` injection
> mechanism, the flag placement reasoning) is still believed correct and
> should be picked back up once the initializer-expression gap is
> understood/fixed — `PI` registration is still needed even then, just not
> sufficient on its own. No code was committed for this plan beyond the plan
> document itself; the branch (`feature/twincat-pi-constant`) was reverted to
> a clean state rather than left with partial/failing work.

## Goal

Register `PI` as an implicit, built-in `LREAL` global constant so that real
CODESYS/TwinCAT code using it as a bare identifier (e.g.
`d2r : LREAL := PI/180.0;`) resolves instead of failing with an undeclared
identifier error.

Per a fresh re-scan of the same real project set behind issue #1199 (run
after the pragma-skipping and `EXTENDS`/`IMPLEMENTS`/`INTERFACE` work
landed), `PI` used as a bare identifier is now the single largest remaining
blocker at 18 files — bigger than `AT %I*`/`AT %Q*` (14), `STRING(n)`/inline
FB-constructor syntax (13), explicit enum-value assignment (11), and
`REFERENCE TO` (10). It's also confirmed to be the cheapest of these: pure
registration, no grammar or parser change.

## Verified against real project files

Checked `/home/husser/code/brotlib` (same TwinCAT codebase used for prior
plans):

- `PI` is used as a bare identifier in expressions across ~18 files, e.g.:
  `d2r : LREAL := PI/180.0;` (`FB_TelescopeControl.TcPOU`,
  `CO_REFRACT_FORWARD.TcPOU`, `FB_HADEC2ALTAZ.TcPOU`, and many more),
  `ATAN2 := ATAN(y/x) + PI;` (`ATAN2.TcPOU`), `2.0*PI` (`FB_NUTATE.TcPOU`).
- **No GVL file in the codebase declares `PI` itself** (`grep -rn "^\s*PI\s*:"
  --include="*.TcGVL"` — zero hits). This rules out the alternative
  explanation that this is really the GVL/cross-file resolution gap
  (`P2008`, tracked separately, structural, out of scope here) — `PI` is
  genuinely expected to be provided implicitly by the compiler/runtime, the
  same way `TRUE`/`FALSE` don't need declaring.
- Every observed usage is in an `LREAL` context (variable type or arithmetic
  with other `LREAL`s). No `REAL`-typed usage found.

## Design

### Mechanism: synthesize a real `VarDecl`, not a special-cased symbol

`SymbolKind::Constant` already exists in `symbol_environment.rs` but is
currently `#[allow(unused)]` — this feature was anticipated but never wired
up.

The existing precedent for an implicit global (`__SYSTEM_UP_TIME`,
`__SYSTEM_UP_LTIME`) uses a weaker mechanism: it registers a bare symbol
name in `symbol_environment` during `analyzer::stages::resolve_types` (so
expressions referencing it type-check), and **separately** re-synthesizes an
actual `VarDecl` in `codegen::compile::compile()` (so it gets memory
allocated) — two separate injection points, because the symbol was never
represented as a real AST node in the analyzed `Library`.

`PI` doesn't need that duplication. Because it's a genuine compile-time
constant (not a runtime-read system value like uptime), the correct model is
a real `VarDecl` — exactly what a user's own
`VAR_GLOBAL CONSTANT PI : LREAL := 3.14159265358979; END_VAR` would produce
— injected as one `LibraryElementKind::GlobalVarDeclarations(vec![pi_decl])`
element into `Library.elements` **once, in `analyzer::stages::resolve_types`**,
before any transform runs. Confirmed by reading `codegen/src/compile.rs`
(~line 235): codegen already generically collects *every* top-level
`GlobalVarDeclarations` element from the `Library` it's given (that's how
`__SYSTEM_UP_TIME`'s codegen-side synthesis gets memory-allocated) — so a
`VarDecl` injected once during analysis flows through symbol resolution,
type checking, *and* codegen automatically, with no codegen changes needed.

Confirmed via `rule_var_decl_const_initialized.rs`: that rule only checks
that a variable *declared with* the `CONSTANT` qualifier itself has an
initializer — it says nothing about whether *other* variables' initializers
must be literal constants. So `d2r : LREAL := PI/180.0;` (a plain `VAR`, not
`VAR CONSTANT`) needs no special constant-folding support; it's just an
ordinary initializer expression referencing another (constant) global
variable, which already works generically once `PI` resolves as a normal
`LREAL` global.

### The `VarDecl` to synthesize

```rust
VarDecl {
    identifier: VariableIdentifier::new_symbol("PI"),
    var_type: VariableType::Global,
    qualifier: DeclarationQualifier::Constant,
    initializer: InitialValueAssignmentKind::simple(
        "LREAL",
        ConstantKind::RealLiteral(RealLiteral {
            value: std::f64::consts::PI, // 3.141592653589793
            data_type: None,
        }),
    ),
}
```

Built with existing DSL helpers (`InitialValueAssignmentKind::simple`,
already used elsewhere) — no new `VarDecl` constructor needed.

### Injection point

In `compiler/analyzer/src/stages.rs::resolve_types`, right after
`library = library.extend(...)` (before `xform_resolve_constant_expressions`
runs), gated by a new flag:

```rust
if options.allow_math_constants {
    library.elements.push(LibraryElementKind::GlobalVarDeclarations(vec![
        pi_var_decl(),
    ]));
}
```

Placed alongside (not replacing) the existing `__SYSTEM_UP_TIME`
`symbol_environment` registration block, following the same "inject
implicit globals early" pattern.

### Dialect flag: `allow_math_constants`

New flag, enabled on `[Rusty, Codesys]` — same placement as
`allow_pragmas`/`allow_oop_extensions`. Named for the general concept
("implicit math constants library") rather than `allow_pi_constant`
specifically, since `PI` is very likely not the last constant of this kind
(commonly paired with e.g. `E` in CODESYS constant libraries), even though
this PR only implements `PI`. Unlike `allow_system_uptime_global`
(`Rusty`-only, described as "an IronPLC/RuSTy runtime convention rather than
a CODESYS feature"), `PI` is genuinely CODESYS/TwinCAT-provided, so it
belongs on both `Rusty` and `Codesys`.

## Non-goals

- No other math constants (`E`, etc.) — not found in the survey, would need
  their own verification against real usage first.
- No general "named constant expression folding" mechanism — `PI` works via
  ordinary variable resolution, not compile-time substitution. If a future
  construct genuinely needs `PI` folded into a literal (e.g. as an array
  bound or `STRING` length, which requires `xform_resolve_constant_expressions`
  to evaluate it), that would need separate work — no evidence from the
  real files that this is needed.
- No `REAL`-typed `PI` — real usage is exclusively `LREAL`. A user assigning
  `PI` directly to a `REAL` variable would still need whatever
  narrowing/conversion rule already governs `LREAL → REAL` today (unchanged
  by this PR).

## File Map

| File | Change |
|------|--------|
| `compiler/parser/src/options.rs` | New `allow_math_constants` flag (`[Rusty, Codesys]`); update descriptor-count tests |
| `compiler/analyzer/src/stages.rs` | Synthesize and inject the `PI` `VarDecl` in `resolve_types`, gated by the flag |
| `compiler/ironplc-cli/src/lsp.rs` | Wire `allowMathConstants` into `extract_compiler_options` (per steering guide checklist) |
| `compiler/mcp/src/tools/list_options.rs` | Descriptor-count test update |
| `docs/explanation/enabling-dialects-and-features.rst`, `docs/reference/compiler/ironplcc.rst`, `specs/steering/syntax-support-guide.md` | Document `--allow-math-constants` |

## Testing Strategy

- Unit test in `stages.rs` (or a focused test module): `resolve_types` with
  `allow_math_constants` off leaves `Library.elements` unchanged; with it on,
  a `GlobalVarDeclarations` element containing `PI` (type `LREAL`, value
  `std::f64::consts::PI`) is present.
- Semantic/integration test: a program declaring
  `d2r : LREAL := PI/180.0;` resolves without a "not declared" diagnostic
  under `allow_math_constants`, and still fails under the default dialect.
- End-to-end execution test (per syntax-support-guide checklist, since this
  produces executable code): compile and run a program computing e.g.
  `deg := 180.0; rad := deg * (PI/180.0);` and assert the resulting `rad`
  value is correct (~3.14159...) — exercises codegen's generic
  `GlobalVarDeclarations` collection actually allocating and initializing
  `PI`'s memory correctly, not just that semantic analysis accepts it.
- Regression: existing `__SYSTEM_UP_TIME` tests and standard programs
  unaffected when the flag is off.

## Tasks

- [x] Write plan
- [ ] `allow_math_constants` flag in `options.rs` (+ descriptor-count test
      updates, same pattern as the previous two PRs)
- [ ] Synthesize + inject `PI` `VarDecl` in `stages.rs::resolve_types`
- [ ] LSP `extract_compiler_options` wiring
- [ ] Unit test: injection happens/doesn't happen based on the flag
- [ ] Semantic test: `PI` resolves in an expression under the flag, fails
      without it
- [ ] End-to-end execution test: compiled program produces the correct
      numeric result using `PI`
- [ ] Update docs (`enabling-dialects-and-features.rst`, `ironplcc.rst`,
      `syntax-support-guide.md`)
- [ ] Run full CI pipeline (`cd compiler && just`)
- [ ] Push branch to fork (no PR against `ironplc/ironplc` without explicit
      go-ahead, per standing instruction)
