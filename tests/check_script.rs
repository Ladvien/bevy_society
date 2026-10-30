//! `check_script` refuses bad syntax with its line, and kind decides what parses.

use bevy_society::{ScriptKind, check_script};

#[test]
fn a_syntax_error_names_its_line() {
    let err = check_script("let a = 1;\nlet b = ;\n", ScriptKind::Statements).unwrap_err();
    assert_eq!(err.line, Some(2));
    assert!(err.column.is_some());
    assert!(err.to_string().starts_with("line 2, column"));
}

#[test]
fn kind_decides_what_parses() {
    assert!(check_script("1 + 1 > 1", ScriptKind::Expression).is_ok());
    assert!(check_script("let a = 1; a", ScriptKind::Statements).is_ok());
    assert!(check_script("let a = 1; a", ScriptKind::Expression).is_err());
}
