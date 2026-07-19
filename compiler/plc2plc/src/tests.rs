//! Tests of renderer.
#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;

    use dsl::core::FileId;

    use ironplc_parser::options::CompilerOptions;
    use ironplc_parser::parse_program;
    use ironplc_test::read_shared_resource;

    use crate::write_to_string;

    pub fn read_resource(name: &'static str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test");
        path.push(name);

        fs::read_to_string(path.clone()).unwrap_or_else(|_| panic!("Unable to read file {path:?}"))
    }

    pub fn parse_and_render_resource(name: &'static str) -> String {
        let source = read_shared_resource(name);
        let library =
            parse_program(&source, &FileId::default(), &CompilerOptions::default()).unwrap();
        write_to_string(&library).unwrap()
    }

    pub fn parse_and_render_resource_with_partial_access(name: &'static str) -> String {
        let source = read_shared_resource(name);
        let options = CompilerOptions {
            allow_partial_access_syntax: true,
            ..CompilerOptions::default()
        };
        let library = parse_program(&source, &FileId::default(), &options).unwrap();
        write_to_string(&library).unwrap()
    }

    #[test]
    fn write_to_string_arrays() {
        let rendered = parse_and_render_resource("array.st");
        let expected = read_resource("array_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_when_wstring_operations_then_round_trips() {
        let rendered = parse_and_render_resource("wstring_ops.st");
        let expected = read_resource("wstring_ops_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_when_array_in_function_var_then_renders() {
        let rendered = parse_and_render_resource("array_in_function_var.st");
        let expected = read_resource("array_in_function_var_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_conditional() {
        let rendered = parse_and_render_resource("conditional.st");
        let expected = read_resource("conditional_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_configuration() {
        let rendered = parse_and_render_resource("configuration.st");
        let expected = read_resource("configuration_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_expressions() {
        let rendered = parse_and_render_resource("expressions.st");
        let expected = read_resource("expressions_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_inout_var_decl() {
        let rendered = parse_and_render_resource("inout_var_decl.st");
        let expected = read_resource("inout_var_decl_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_input_var_decl() {
        let rendered = parse_and_render_resource("input_var_decl.st");
        let expected = read_resource("input_var_decl_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_literal() {
        let rendered = parse_and_render_resource("literal.st");
        let expected = read_resource("literal_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_nested() {
        let rendered = parse_and_render_resource("nested.st");
        let expected = read_resource("nested_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_program() {
        let rendered = parse_and_render_resource("program.st");
        let expected = read_resource("program_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_sfc() {
        let rendered = parse_and_render_resource("sfc.st");
        let expected = read_resource("sfc_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_strings() {
        let rendered = parse_and_render_resource("strings.st");
        let expected = read_resource("strings_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_textual() {
        let rendered = parse_and_render_resource("textual.st");
        let expected = read_resource("textual_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_type_decl() {
        let rendered = parse_and_render_resource("type_decl.st");
        let expected = read_resource("type_decl_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_var_decl() {
        let rendered = parse_and_render_resource("var_decl.st");
        let expected = read_resource("var_decl_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_sized_string_contexts() {
        let rendered = parse_and_render_resource("sized_string_contexts.st");
        let expected = read_resource("sized_string_contexts_rendered.st");
        assert_eq!(rendered, expected);
    }

    pub fn parse_and_render_resource_edition3(name: &'static str) -> String {
        let source = read_shared_resource(name);
        let options = CompilerOptions {
            allow_iec_61131_3_2013: true,
            ..CompilerOptions::default()
        };
        let library = parse_program(&source, &FileId::default(), &options).unwrap();
        write_to_string(&library).unwrap()
    }

    fn parse_and_render_edition3(source: &str) -> String {
        let options = CompilerOptions {
            allow_iec_61131_3_2013: true,
            ..CompilerOptions::default()
        };
        let library = parse_program(source, &FileId::default(), &options).unwrap();
        write_to_string(&library).unwrap()
    }

    #[test]
    fn write_to_string_ref() {
        let rendered = parse_and_render_resource_edition3("ref.st");
        let expected = read_resource("ref_rendered.st");
        assert_eq!(rendered, expected);
    }

    /// REQ-PAB-060: plc2plc normalizes `.%Xn` to `.n`. Re-parsing the rendered
    /// output must yield the same AST (structural equivalence).
    #[test]
    fn plc2plc_spec_req_pab_060_percent_x_round_trips_through_short_form() {
        let rendered = parse_and_render_resource_with_partial_access("partial_access_bit.st");
        let expected = read_resource("partial_access_bit_rendered.st");
        assert_eq!(rendered, expected);

        // Confirm re-parsing the rendered output yields the same AST as parsing
        // the original source, proving the normalization is semantics-preserving.
        let original = read_shared_resource("partial_access_bit.st");
        let options = CompilerOptions {
            allow_partial_access_syntax: true,
            ..CompilerOptions::default()
        };
        let library_original = parse_program(&original, &FileId::default(), &options).unwrap();
        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse under default (no-flag) options");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn plc2plc_when_partial_access_multi_then_round_trips() {
        let rendered = parse_and_render_resource_with_partial_access("partial_access_multi.st");
        let expected = read_resource("partial_access_multi_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_when_ref_to_var_decl_then_preserves_ref_to() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    x : REF_TO INT;
END_VAR
END_PROGRAM",
        );
        assert!(
            rendered.contains("REF_TO INT"),
            "Expected REF_TO INT in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_ref_to_array_var_decl_then_preserves_ref_to() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    PT : REF_TO ARRAY[0..10] OF BYTE;
END_VAR
END_PROGRAM",
        );
        assert!(
            rendered.contains("REF_TO ARRAY"),
            "Expected REF_TO ARRAY in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_ref_to_var_decl_with_null_init_then_preserves() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    x : REF_TO INT := NULL;
END_VAR
END_PROGRAM",
        );
        assert!(
            rendered.contains("REF_TO INT := NULL"),
            "Expected REF_TO INT := NULL in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_ref_to_var_decl_with_ref_init_then_preserves() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    counter : INT;
    x : REF_TO INT := REF(counter);
END_VAR
END_PROGRAM",
        );
        assert!(
            rendered.contains("REF_TO INT := REF("),
            "Expected REF_TO INT := REF(...) in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_deref_assign_then_preserves_caret() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    myRef : REF_TO INT;
END_VAR
    myRef^ := 42;
END_PROGRAM",
        );
        assert!(
            rendered.contains("myRef^ :="),
            "Expected myRef^ := in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_deref_expression_then_preserves_caret() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    myRef : REF_TO INT;
    value : INT;
END_VAR
    value := myRef^;
END_PROGRAM",
        );
        assert!(
            rendered.contains("myRef ^"),
            "Expected myRef ^ in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_deref_array_expression_then_preserves() {
        let rendered = parse_and_render_edition3(
            "FUNCTION my_func : BYTE
VAR_INPUT
    PT : REF_TO ARRAY[0..10] OF BYTE;
END_VAR
    my_func := PT^[0];
END_FUNCTION",
        );
        assert!(
            rendered.contains("PT^"),
            "Expected PT^ in output, got: {rendered}"
        );
        assert!(
            rendered.contains("[ 0 ]"),
            "Expected array subscript in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_ref_expression_then_preserves() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    counter : INT;
    x : REF_TO INT;
END_VAR
    x := REF(counter);
END_PROGRAM",
        );
        assert!(
            rendered.contains("REF("),
            "Expected REF( in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_null_expression_then_preserves() {
        let rendered = parse_and_render_edition3(
            "PROGRAM main
VAR
    x : REF_TO INT;
END_VAR
    x := NULL;
END_PROGRAM",
        );
        assert!(
            rendered.contains("NULL"),
            "Expected NULL in output, got: {rendered}"
        );
    }

    #[test]
    fn write_to_string_when_ref_to_type_decl_then_preserves() {
        let rendered = parse_and_render_edition3("TYPE IntRef : REF_TO INT; END_TYPE");
        let expected = "TYPE\n   IntRef : REF_TO INT ;\nEND_TYPE\n";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_late_bound_declaration() {
        use ironplc_dsl::common::{
            DataTypeDeclarationKind, LateBoundDeclaration, Library, LibraryElementKind, TypeName,
        };

        // Create a library with a late bound declaration in code
        let late_bound_decl = LateBoundDeclaration {
            data_type_name: TypeName::from("MY_ALIAS"),
            base_type_name: TypeName::from("INT"),
        };

        let library = Library {
            elements: vec![LibraryElementKind::DataTypeDeclaration(
                DataTypeDeclarationKind::LateBound(late_bound_decl),
            )],
        };

        // Render the library to string
        let result = crate::write_to_string(&library).unwrap();

        // Expected output should be a TYPE declaration with the alias
        let expected = "TYPE\n   MY_ALIAS : INT ;\nEND_TYPE\n";
        assert_eq!(result, expected);
    }

    fn parse_and_render_resource_empty_var_blocks(name: &'static str) -> String {
        let source = read_shared_resource(name);
        let options = CompilerOptions {
            allow_empty_var_blocks: true,
            ..CompilerOptions::default()
        };
        let library = parse_program(&source, &FileId::default(), &options).unwrap();
        write_to_string(&library).unwrap()
    }

    #[test]
    fn write_to_string_empty_var_block() {
        let rendered = parse_and_render_resource_empty_var_blocks("empty_var_block.st");
        let expected = read_resource("empty_var_block_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_var_temp() {
        let rendered = parse_and_render_resource("var_temp.st");
        let expected = read_resource("var_temp_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_when_array_of_string_with_size_then_renders_size() {
        let rendered = parse_and_render_resource("array_of_string.st");
        let expected = read_resource("array_of_string_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_when_time_function_decl_then_round_trips() {
        let source = read_shared_resource("time_function_decl.st");
        let options = CompilerOptions {
            allow_time_as_function_name: true,
            ..CompilerOptions::default()
        };
        let library = parse_program(&source, &FileId::default(), &options).unwrap();
        let rendered = write_to_string(&library).unwrap();
        let expected = read_resource("time_function_decl_rendered.st");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn write_to_string_sizeof() {
        let source = read_shared_resource("sizeof.st");
        let options = CompilerOptions {
            allow_sizeof: true,
            ..CompilerOptions::default()
        };
        let library = parse_program(&source, &FileId::default(), &options).unwrap();
        let rendered = write_to_string(&library).unwrap();
        let expected = read_resource("sizeof_rendered.st");
        assert_eq!(rendered, expected);
    }

    // ---------------------------------------------------------------------
    // CODESYS/TwinCAT OOP extensions: EXTENDS/IMPLEMENTS/INTERFACE round-trip.
    // See specs/plans/2026-07-18-twincat-extends-implements-interface.md.
    // ---------------------------------------------------------------------

    #[test]
    fn write_to_string_when_extends_implements_and_interface_then_round_trips() {
        let source = "
INTERFACE I_Drivable
END_INTERFACE

INTERFACE I_Loggable
END_INTERFACE

FUNCTION_BLOCK FB_AdvancedMotor EXTENDS FB_Motor IMPLEMENTS I_Drivable, I_Loggable
VAR
    bRunning : BOOL;
END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK FB_Motor
VAR
    bRunning : BOOL;
END_VAR
END_FUNCTION_BLOCK
";
        let options = CompilerOptions {
            allow_oop_extensions: true,
            ..CompilerOptions::default()
        };
        let library_original = parse_program(source, &FileId::default(), &options).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("EXTENDS FB_Motor"));
        assert!(rendered.contains("IMPLEMENTS I_Drivable , I_Loggable"));
        assert!(rendered.contains("INTERFACE I_Drivable"));
        assert!(rendered.contains("END_INTERFACE"));

        let library_rendered = parse_program(&rendered, &FileId::default(), &options)
            .expect("rendered output must parse under the same dialect");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_interface_extends_base_then_round_trips() {
        let source = "
INTERFACE I_BaseAxis
END_INTERFACE

INTERFACE I_Focus EXTENDS I_BaseAxis
END_INTERFACE
";
        let options = CompilerOptions {
            allow_oop_extensions: true,
            ..CompilerOptions::default()
        };
        let library_original = parse_program(source, &FileId::default(), &options).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("INTERFACE I_Focus"));
        assert!(rendered.contains("EXTENDS I_BaseAxis"));

        let library_rendered = parse_program(&rendered, &FileId::default(), &options)
            .expect("rendered output must parse under the same dialect");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_constant_expression_initializer_then_round_trips() {
        let source = "
PROGRAM main
VAR
    d2r : LREAL := PI/180.5;
END_VAR
END_PROGRAM
";
        let options = CompilerOptions {
            allow_constant_initializer_expressions: true,
            ..CompilerOptions::default()
        };
        let library_original = parse_program(source, &FileId::default(), &options).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("PI"));
        assert!(rendered.contains("180.5"));

        let library_rendered = parse_program(&rendered, &FileId::default(), &options)
            .expect("rendered output must parse under the same dialect");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_mixed_located_var_declaration_then_round_trips() {
        use dsl::common::{LibraryElementKind, VariableIdentifier};

        let source = "
FUNCTION_BLOCK FB_Example
VAR
    tempSensor AT%I*: INT;
    fbComm : BOOL;
END_VAR
END_FUNCTION_BLOCK
";
        let options = CompilerOptions {
            allow_mixed_located_var_declarations: true,
            ..CompilerOptions::default()
        };
        let library_original = parse_program(source, &FileId::default(), &options).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("AT %I*"));
        assert!(rendered.contains("tempSensor"));
        assert!(rendered.contains("fbComm"));

        // The renderer emits one VAR...END_VAR block per declaration
        // (pre-existing behavior, not specific to this feature), so the
        // located and plain variables end up in separate rendered blocks.
        // Re-parsing therefore no longer sees them as "mixed" (each
        // rendered block contains only one variable), so a strict library
        // equality check against the original isn't meaningful here.
        // Instead, verify the rendered output re-parses cleanly under the
        // same flag and both variables keep their original shape.
        let library_rendered = parse_program(&rendered, &FileId::default(), &options)
            .expect("rendered output must parse under the same dialect");

        let fb = match &library_rendered.elements[0] {
            LibraryElementKind::FunctionBlockDeclaration(fb) => fb,
            other => panic!("expected FunctionBlockDeclaration, got {other:?}"),
        };
        assert_eq!(fb.variables.len(), 2);
        let temp_sensor = fb
            .variables
            .iter()
            .find(|v| matches!(&v.identifier, VariableIdentifier::Direct(d) if d.name.as_ref().map(|n| n.to_string()) == Some("tempSensor".to_string())))
            .expect("tempSensor should still be a Direct (located) variable");
        assert!(matches!(
            &temp_sensor.identifier,
            VariableIdentifier::Direct(_)
        ));

        let fb_comm = fb
            .variables
            .iter()
            .find(|v| matches!(&v.identifier, VariableIdentifier::Symbol(id) if id.to_string() == "fbComm"))
            .expect("fbComm should still be a plain Symbol variable");
        assert!(matches!(&fb_comm.identifier, VariableIdentifier::Symbol(_)));
    }

    #[test]
    fn write_to_string_when_qualified_fb_call_then_round_trips() {
        let source = "
FUNCTION_BLOCK FB_Outer
VAR
    fbComm : FB_Inner;
END_VAR
    fbComm.Publish('a', 'b');
END_FUNCTION_BLOCK
";
        let library_original =
            parse_program(source, &FileId::default(), &CompilerOptions::default()).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("fbComm.Publish"));

        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_string_parenthesis_length_then_normalizes_to_brackets() {
        let source = "
FUNCTION_BLOCK FB_Example
VAR
    hostName : STRING(255);
END_VAR
END_FUNCTION_BLOCK
";
        let library_original =
            parse_program(source, &FileId::default(), &CompilerOptions::default()).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        // The renderer always normalizes to the bracket form -- there's no
        // bracket/paren marker stored in the DSL, matching how
        // StringSpecification/StringInitializer already only store
        // length: Option<IntegerRef> with no delimiter distinction.
        assert!(rendered.contains("STRING [ 255 ]"));

        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_fb_instance_call_style_init_then_round_trips() {
        let source = "
FUNCTION_BLOCK FB_Comm
VAR_INPUT
    retries : INT;
END_VAR
END_FUNCTION_BLOCK

FUNCTION_BLOCK FB_Example
VAR
    comm : FB_Comm(retries := 3, THIS);
END_VAR
END_FUNCTION_BLOCK
";
        let library_original =
            parse_program(source, &FileId::default(), &CompilerOptions::default()).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("FB_Comm ( retries := 3 , THIS )"));

        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_enum_explicit_values_then_round_trips() {
        let source = "
TYPE
E_ModeLanguage : (Deutsch := 1, English := 2);
END_TYPE
";
        let library_original =
            parse_program(source, &FileId::default(), &CompilerOptions::default()).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("Deutsch := 1"));
        assert!(rendered.contains("English := 2"));

        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_enum_base_type_suffix_then_round_trips() {
        let source = "
TYPE
E_AssertionType : (Type_UNDEFINED := 0, Type_ANY, Type_BOOL) BYTE;
END_TYPE
";
        let library_original =
            parse_program(source, &FileId::default(), &CompilerOptions::default()).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("BYTE"));

        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse");
        assert_eq!(library_original, library_rendered);
    }

    #[test]
    fn write_to_string_when_qualified_enum_value_then_renders_hash() {
        // Regression for a pre-existing bug found while adding
        // explicit_value rendering: COLOR#RED previously rendered as
        // "COLOR RED" (missing the '#'), because there was no dedicated
        // visit_enumerated_value override -- the default recursive
        // visitor used visit_id's write_ws, inserting a space instead of
        // the qualifier separator.
        let source = "
TYPE
COLOR : (RED, GREEN, BLUE);
END_TYPE
FUNCTION_BLOCK FB_Example
VAR
    x : COLOR := COLOR#RED;
END_VAR
END_FUNCTION_BLOCK
";
        let library_original =
            parse_program(source, &FileId::default(), &CompilerOptions::default()).unwrap();
        let rendered = write_to_string(&library_original).unwrap();

        assert!(rendered.contains("COLOR#RED"));

        let library_rendered =
            parse_program(&rendered, &FileId::default(), &CompilerOptions::default())
                .expect("rendered output must parse");
        assert_eq!(library_original, library_rendered);
    }
}
