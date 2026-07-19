# Plan: Parse Qualified Method-Call Statements (Recognized, Not Yet Supported)

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
(`/home/husser/code/brotlib`): **`fbComm : I_Comm;`** — an **interface
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
- [ ] `FbCall.qualifier` field + `VendorExtension` impl + `fb_call_mapped()` update
- [ ] Grammar: qualifier prefix in `fb_invocation()`
- [ ] `rule_unsupported_extension.rs`: `visit_fb_call` override
- [ ] Tests from Testing Strategy
- [ ] Update docs (syntax-support-guide.md mention)
- [ ] Run full CI pipeline (`cd compiler && just`)
- [ ] Push branch to fork (no PR against `ironplc/ironplc` without explicit
      go-ahead, per standing instruction)
