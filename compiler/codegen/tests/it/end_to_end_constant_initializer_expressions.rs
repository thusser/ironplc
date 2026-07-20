//! End-to-end integration tests for constant-expression `VAR` initializers
//! (e.g. `d2r : LREAL := PI/180.0;`), enabled by
//! `--allow-constant-initializer-expressions`.
//!
//! See specs/plans/2026-07-19-twincat-var-initializer-expressions.md.

use ironplc_parser::options::CompilerOptions;

use crate::common::parse_and_run;

#[test]
fn end_to_end_when_arithmetic_initializer_then_computes_value() {
    let source = "
PROGRAM main
VAR
    d2r : LREAL := 4.25/180.0;
END_VAR
END_PROGRAM
";
    let options = CompilerOptions {
        allow_constant_initializer_expressions: true,
        ..CompilerOptions::default()
    };
    let (_c, bufs) = parse_and_run(source, &options);

    let d2r = bufs.vars[0].as_f64();
    assert!((d2r - (4.25 / 180.0)).abs() < 1e-12, "got {d2r}");
}

#[test]
fn end_to_end_when_pi_used_in_initializer_then_computes_value() {
    // The dominant real-world pattern from real TwinCAT code (e.g.
    // FB_TelescopeControl.TcPOU): PI registration alone does not fix this —
    // it also needs allow_constant_initializer_expressions to fold the
    // expression at compile time.
    let source = "
PROGRAM main
VAR
    d2r : LREAL := PI/180.0;
END_VAR
END_PROGRAM
";
    let options = CompilerOptions {
        allow_math_constants: true,
        allow_constant_initializer_expressions: true,
        ..CompilerOptions::default()
    };
    let (_c, bufs) = parse_and_run(source, &options);

    // var layout: PI=0, d2r=1
    let d2r = bufs.vars[1].as_f64();
    assert!(
        (d2r - (std::f64::consts::PI / 180.0)).abs() < 1e-12,
        "got {d2r}"
    );
}

#[test]
fn end_to_end_when_user_constant_used_in_initializer_then_computes_value() {
    let source = "
VAR_GLOBAL CONSTANT
    SCALE : LREAL := 2.5;
END_VAR
PROGRAM main
VAR
    scaled : LREAL := SCALE*4.0;
END_VAR
END_PROGRAM
";
    let options = CompilerOptions {
        allow_top_level_var_global: true,
        allow_constant_initializer_expressions: true,
        ..CompilerOptions::default()
    };
    let (_c, bufs) = parse_and_run(source, &options);

    // var layout: SCALE=0, scaled=1
    let scaled = bufs.vars[1].as_f64();
    assert!((scaled - (2.5 * 4.0)).abs() < 1e-12, "got {scaled}");
}
