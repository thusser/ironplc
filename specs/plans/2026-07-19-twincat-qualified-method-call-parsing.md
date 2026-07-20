# Plan: Parse Qualified Method-Call Statements (Recognized, Not Yet Supported)

**Status: implemented and landed on this branch.** `fbComm.Publish('a', 'b');`
now parses under every dialect and is flagged `P9004`, matching the design
below. One pre-existing, unrelated renderer bug was found and fixed along
the way — see "Implementation Notes" at the end of this file.

## Goal

Allow `fbComm.Publish('a', 'b');` — a qualified call statement invoking
something through a member instance — to *parse* (so the rest of the file
is checked normally), without implementing real method/interface dispatch.
Mark the call itself as a recognized-but-unsupported vendor extension
(`P9004`), exactly like `EXTENDS`/`IMPLEMENTS`/`INTERFACE` already are.

```
FUNCTION_BLOCK FB_Example
VAR
    fbComm : I_Comm;
END_VAR
    fbComm.Publish('telescope', 'dome', 'SENSOR.VALUE', '42');  // currently a syntax error
END_FUNCTION_BLOCK
```

## How this was reported and verified

Reported from the user's IntelliJ-plugin project as "qualified method-call
statements are broken generically," traced by the user to `fb_invocation()`'s
`fb_name()` grammar rule in `parser.rs` (`rule fb_name() -> Id = i:identifier() { i }`
— a single bare identifier only, no dotted path).

Independently reproduced with a minimal synthetic case, isolated from the
unrelated `METHOD`-declaration parsing gap (which fails separately and
first if the callee's `METHOD` body is included in the same test):

- `fbComm(a := 'x');` (direct FB invocation, no qualifier) — parses today.
- `fbComm.Publish(a := 'x');` (qualified) — fails with `P0002` exactly at
  the `(`, because the parser reads `fbComm.Publish` as a structured
  variable access (field `Publish` of `fbComm`) and then has no statement
  form for a call following it.

Checked what `fbComm`'s real type actually is in the reporting codebase
(a private local checkout of a real TwinCAT project):
**`fbComm : I_Comm;`** — an **interface
type**, not a concrete function block. `Publish` is a method on that
interface, dispatched polymorphically to whatever concrete FB is assigned
to `fbComm` at runtime (confirmed a `THIS^.fbComm := comm;` assignment
elsewhere). This is genuine interface-based virtual dispatch, not just
"call a method on a known concrete type." 34 call sites across the
codebase use this exact pattern (`fbComm.Publish(...)`, `fbClient.Execute()`),
and 66 files contain `METHOD` declarations that would need to parse for
real dispatch to ever work end-to-end.

## Why this is scoped to parsing only, not real dispatch

Building working interface/method dispatch requires: parsing `METHOD`/
`END_METHOD` bodies (currently rejected outright by the parser — confirmed
directly), a vtable-like mechanism per interface, and new codegen for
indirect calls through it. That's the same "Full OOP" bucket already
flagged in `twincat-status.md` as a big, multi-PR effort, not a single
scoped feature.

The project's own stated motivation for this whole line of work (from
issue #1199) is a **linter/diagnostics backend**, not full program
execution. For that use case, "parses cleanly, semantic rule flags the
call as not-yet-supported" is the useful outcome — it unblocks checking
everything else in the file (variable declarations, other statements,
type references) instead of the whole file failing at the first qualified
call. This exactly mirrors how `EXTENDS`/`IMPLEMENTS`/`INTERFACE` were
handled: parsed and represented, flagged via `VendorExtension` → `P9004`,
real semantics deferred.

## Design

### DSL: add an optional qualifier to `FbCall`

```rust
// compiler/dsl/src/textual.rs
pub struct FbCall {
    /// Present for a qualified call (`instance.Method(...)`) -- the
    /// receiver's name. `None` for an ordinary direct FB invocation
    /// (`instance(...)`).
    pub qualifier: Option<Id>,
    pub var_name: Id,
    pub params: Vec<ParamAssignmentKind>,
    pub position: SourceSpan,
}
```

Only 2 construction sites exist (`StmtKind::fb_call_mapped()` in
`textual.rs`, and the grammar action in `parser.rs`) — both updated to set
`qualifier: None` except the new grammar path. No exhaustive-match ripple
(`FbCall` is a struct, not an enum); code reading `fb_call.var_name` for
the ordinary case needs no changes.

Scoped to a single qualifier level (`instance.Method(...)`), matching
every real occurrence found (`fbComm.Publish(...)`, `fbClient.Execute()`)
— no evidence of deeper paths (`a.b.c()`) in the survey.

### Grammar: one new optional prefix on `fb_invocation()`

```
rule fb_invocation() -> StmtKind =
  qualifier:(q:fb_name() tok(TokenType::Period) { q })? name:fb_name() _
  tok(TokenType::LeftParen) _
  params:param_assignment() ** (_ tok(TokenType::Comma) _) _
  end:tok(TokenType::RightParen) {
    StmtKind::FbCall(FbCall { qualifier, var_name: name, params, position: ... })
  }
```

No ordering hazard (unlike the earlier `constant()`/`expression()`
initializer case): `assignment_statement()` — tried before
`subprogram_control_statement()` in the top-level `statement()` rule —
requires a mandatory `:=` (or `^ :=`) immediately after `variable()`
parses the same dotted path; for a call statement the next token is `(`
instead, so `assignment_statement()` fails cleanly and completely (a hard
token mismatch, not a greedy partial match), and PEG backtracks fully to
try `fb_invocation()` next. Positional and named parameters already work
via the existing `param_assignment()` rule — no changes needed there
(confirmed `fbComm.Publish('a', 'b', ...)`-style all-positional calls
already parse once the qualifier prefix is accepted).

### Semantic: flag via the existing `VendorExtension` mechanism

```rust
// compiler/dsl/src/textual.rs
impl VendorExtension for FbCall {
    fn extension_name(&self) -> &'static str { "Qualified method-call statement" }
    fn extension_origins(&self) -> &'static [ExtensionOrigin] {
        &[ExtensionOrigin::BeckhoffCodesys]
    }
    fn extension_span(&self) -> SourceSpan { self.position.clone() }
}
```

`rule_unsupported_extension.rs` gets a new `visit_fb_call` override,
flagging only when `qualifier.is_some()` — same "implement unconditionally,
let the visitor decide when to flag" pattern already used for
`FunctionBlockDeclaration`'s `EXTENDS`/`IMPLEMENTS` fields (a plain
`fbComm(...)` call is standard IEC 61131-3 and must never be flagged).

No new dialect flag — this reuses the same `P9004` mechanism as
`EXTENDS`/`IMPLEMENTS`/`INTERFACE`, which is itself gated by
`allow_oop_extensions` at the point those constructs are *recognized*
(the qualifier syntax here has no keyword to gate at the lexer level the
way `EXTENDS` does; `P9004` firing whenever a qualified call is *parsed*
is the gate).

## Non-goals

- Real method/interface dispatch (parsing `METHOD` bodies, vtables,
  indirect-call codegen) — the "Full OOP" work, already tracked separately
  in `twincat-status.md`'s "Next" list as a big, multi-PR effort.
- Distinguishing "this is really a method call" from "this is really a
  nested FB instance invoked through a field" — both produce the identical
  `qualifier.is_some()` shape here and are flagged identically; that
  distinction only matters once real dispatch is built.
- Deeper qualifier chains (`a.b.c()`) — no evidence needed.
- Any change to `ctx.fb_instances` resolution in codegen — codegen is
  never reached for a flagged file (compilation stops at the semantic
  diagnostic stage, same as `EXTENDS`/`IMPLEMENTS` today).

## File Map

| File | Change |
|------|--------|
| `compiler/dsl/src/textual.rs` | `FbCall.qualifier: Option<Id>`; `impl VendorExtension for FbCall`; update `fb_call_mapped()` |
| `compiler/parser/src/parser.rs` | `fb_invocation()` grammar: optional qualifier prefix |
| `compiler/analyzer/src/rule_unsupported_extension.rs` | New `visit_fb_call` override |
| Docs | No new `--allow-x` flag to document; may add a short mention to `syntax-support-guide.md`'s vendor-extension list for discoverability |

## Testing Strategy

- Parser tests: qualified call parses (`fbComm.Publish('a', 'b');`);
  unqualified call still parses unchanged (regression); qualified call as
  an assignment target (`fbComm.Publish := x;`, a genuine structured-field
  assignment, unrelated to this feature) still parses as an assignment,
  not swallowed by the new grammar path.
- Semantic tests: qualified call → `P9004`; plain direct FB call → no
  diagnostic (regression, proving the visitor only flags when qualified).
- No end-to-end execution test — codegen is never reached for a flagged
  file; nothing new to execute.

## Tasks

- [x] Write plan (this document)
- [x] `FbCall.qualifier` field + `VendorExtension` impl + `fb_call_mapped()` update
- [x] Grammar: qualifier prefix in `fb_invocation()`
- [x] `rule_unsupported_extension.rs`: `visit_fb_call` override
- [x] `rule_function_block_invocation.rs`: skip qualified calls (see
      Implementation Notes — not in the original plan, found via testing)
- [x] Tests from Testing Strategy
- [x] Update docs (`enabling-dialects-and-features.rst` note appended to
      the existing `--allow-oop-extensions` entry, since this construct
      needs no new flag of its own)
- [x] Run full CI pipeline (`cd compiler && just`)
- [x] Push branch to fork (no PR against `ironplc/ironplc` without explicit
      go-ahead, per standing instruction)

## Implementation Notes

- **A second, unrelated semantic rule needed a fix too, found only by
  running the smoke test, not by reading the grammar.** `P9004` fired
  correctly, but a *second*, misleading diagnostic also appeared:
  `P4012 Function block invocation is not a variable in scope
  (invocation=Publish)` — from `rule_function_block_invocation.rs`, which
  looks up `fb_call.var_name` ("Publish") in a map of declared FB-typed
  variables and, finding nothing (since "Publish" was never a declared
  variable — "fbComm" was), reports it as an undeclared invocation. That
  rule has no way to understand a qualified call at all, so it needed an
  early-return guard (`if fb_call.qualifier.is_some() { return Ok(()); }`)
  to defer entirely to `rule_unsupported_extension`'s `P9004`, rather than
  also emitting its own (in this case incorrect) diagnostic. This wasn't
  anticipated in the plan — it only surfaced by actually running the
  compiler against a real reproduction rather than just tracing the
  grammar/AST changes on paper.
- **Verified the P9004-blocks-codegen assumption holds**, and confirmed a
  real, still-open, *unrelated* gap along the way: declaring a variable of
  an `INTERFACE` type (`fbComm : I_Comm;`, the exact shape from the
  reporting codebase) already produces its own separate diagnostic
  (`P2008 Cannot determine kind of type identifier`) with or without this
  feature — a pre-existing limitation of the `INTERFACE`-as-variable-type
  support from the earlier `EXTENDS`/`IMPLEMENTS`/`INTERFACE` work, not
  something this change introduces or needs to fix.
- **Found and fixed a genuine, unrelated pre-existing renderer bug while
  writing the plc2plc round-trip test**: `visit_fb_call` in `renderer.rs`
  never wrote a trailing `;` at all — for *any* FB call, qualified or not.
  Every other statement-rendering function in the file writes its own
  trailing separator (confirmed by grepping for `write_ws(";")`/`write(";")`
  across every other `visit_*` statement function); `visit_fb_call` was
  simply missing it, presumably because no prior test round-tripped a
  standalone FB-call statement through plc2plc. Fixed as part of this
  change since it directly blocked the new round-trip test, rather than
  filing it separately.
- **Rendering the qualifier needed raw `write()`, not `visit_id()`'s
  `write_ws()`** — matching the exact pattern `visit_structured_variable`
  already uses for its own period-separated access
  (`self.write(node.field.original().as_str())` rather than
  `self.visit_id(&node.field)`), since `write_ws` unconditionally inserts
  a leading space unless the buffer already ends in one, which produced
  `fbComm. Publish` (a space after the period) the first time through.
- **No new dialect flag** — unlike every other feature this session, this
  construct needs no new keyword to gate at the lexer level (the way
  `EXTENDS` is), so there's nothing to gate the parse itself on. `P9004`
  firing whenever a qualified call is parsed *is* the gate, consistent
  with `rule_unsupported_extension`'s existing design (it doesn't check
  `allow_oop_extensions` either — `INTERFACE`/`EXTENDS`/`IMPLEMENTS` are
  gated by *whether the keyword parses at all* via token demotion, not by
  the semantic rule re-checking the flag).
