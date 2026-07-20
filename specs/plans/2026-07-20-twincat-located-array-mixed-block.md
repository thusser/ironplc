# Plan: `AT`-Located `ARRAY` Variable in a Mixed `VAR` Block

## Goal

An `AT`-located variable with an `ARRAY` type (e.g. `outputs AT %Q* :
ARRAY[0..9] OF BOOL;`) already parses fine in its own **dedicated**
`VAR ... END_VAR` block, but fails with a `P0002` syntax error when
mixed alongside plain variables in the same block (the
`--allow-mixed-located-var-declarations` shape landed earlier this
session):

```
FUNCTION_BLOCK FB_Example
VAR
    bEnabled : BOOL;
    outputs AT %Q* : ARRAY[0..9] OF BOOL;  // currently a P0002 parse error
END_VAR
END_FUNCTION_BLOCK
```

## Verification against real code

Confirmed directly with two repros:

- **Dedicated block** (`outputs AT %Q* : ARRAY[0..9] OF BOOL;` alone in
  its own `VAR ... END_VAR`) — already parses cleanly today, no changes
  needed. This path goes through `incompl_located_var_decl()` ->
  `var_spec()`, and `var_spec()` already includes
  `array_specification()` as one of its alternatives.
- **Mixed block** (the same declaration alongside an ordinary plain
  variable in one block) — produces `P0002` at the `ARRAY` token. This
  path goes through `located_var1_init_decl()` (added by the
  mixed-located-var-declarations feature), whose init rule is
  `simple_or_enumerated_or_subrange_ambiguous_struct_spec_init()` — which
  has no array alternative at all.

So this is exactly the same shape as the survey's framing: "the
located-declaration path and the array-type path don't compose" — but
narrower than it first sounds: only the *mixed-block* located-declaration
path is missing array support; the dedicated-block path never had this
gap.

## Design

Add an `array_spec_init()` alternative to `located_var1_init_decl()`,
tried first (an `ARRAY` token makes this unambiguous with the other
alternatives, which all start with a type name):

```rust
rule located_var1_init_decl() -> Vec<UntypedVarDecl> = name:variable_name() _ loc:(location() / incompl_location()) _ tok(TokenType::Colon) _ init:(arr:array_spec_init() { InitialValueAssignmentKind::Array(arr) } / simple_or_enumerated_or_subrange_ambiguous_struct_spec_init()) {
  vec![UntypedVarDecl {
    name,
    location: Some(loc),
    initializer: init,
  }]
}
```

This mirrors how the dedicated-block path already combines both shapes
in `located_var_spec_init()` (`arr:array_spec_init() { ... } /
simple:simple_spec_init() { simple }`), and how the plain (non-located)
mixed-block path already has `array_var_init_decl()` as a sibling
alternative in `var_init_decl()`.

No other changes expected: `UntypedVarDecl`'s `location: Some(loc)` is
already what `rule_mixed_located_var_declarations.rs` keys off for its
mixed-block gating, and `VarDeclarations::flat_map`'s location-handling
was already fixed as part of the earlier mixed-located-var-declarations
work.

## Non-goals

- Any change to the dedicated-block path — already works.
- Array *initializers* on a located array variable (e.g. `outputs AT
  %Q* : ARRAY[0..9] OF BOOL := [10(FALSE)];`) beyond what
  `array_spec_init()` already supports generically — no real file in
  the survey needs anything beyond a bare type spec, and
  `array_spec_init()`'s existing initializer support is reused as-is,
  not extended.

## File Map

| File | Change |
|------|--------|
| `compiler/parser/src/parser.rs` | `located_var1_init_decl()`: add the `array_spec_init()` alternative |

## Testing Strategy

- Parser test: a located `ARRAY` variable mixed with a plain variable in
  the same block parses, with the correct `InitialValueAssignmentKind::Array`
  shape and `location` set.
- Regression: the existing dedicated-block located-array case still
  parses (already covered by existing tests, if any — confirm during
  implementation).
- Regression: an all-plain mixed block (no located variables at all)
  and an all-located block (no plain variables) both still parse
  unaffected.
- plc2plc round-trip test for the new shape.
- End-to-end: verify via the CLI that the original repro now parses
  clean under `--dialect=codesys`.

## Tasks

- [x] Write plan (this document)
- [x] Grammar fix in `located_var1_init_decl()`
- [x] Tests from Testing Strategy
- [x] Verify end-to-end via CLI
- [x] Run full CI pipeline (`cd compiler && just`)
- [ ] Push branch to fork
- [ ] Merge into `twincat-dev`, update `twincat-status.md`, push

## Implementation Notes

- No changes needed beyond the single grammar-rule fix: `UntypedVarDecl`'s
  location handling and the mixed-block semantic gating
  (`rule_mixed_located_var_declarations.rs`) were both already
  shape-agnostic (they key off the `location` field, not the
  `initializer` variant), and `visit_initial_value_assignment_kind` in
  the plc2plc renderer already dispatches `Array` generically — neither
  needed touching.
- Verified end-to-end via the CLI both ways: the mixed located-array
  repro parses clean under `--dialect=codesys`, and correctly produces
  `P4038` under the default dialect (confirming the mixed-block
  semantic gate applies to the array case exactly like every other
  located-variable shape).
