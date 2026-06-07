use rusticle::interpreter::Interpreter;

#[test]
fn context_write_from_proc() {
    let mut interp = Interpreter::new();
    interp.eval(r#"context st { set x "old" }"#).unwrap();
    interp.eval(r#"proc change {val} { set st::x $val }"#).unwrap();
    interp.eval(r#"change "new""#).unwrap();
    let val = interp.eval("return $st::x").unwrap();
    assert_eq!(val.as_str(), "new");
}

#[test]
fn context_write_invalid_context_fails() {
    let mut interp = Interpreter::new();
    let result = interp.eval(r#"set bogus::x hello"#);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("no such context"), "got: {msg}");
}

#[test]
fn context_validate_warns_on_bad_context() {
    let interp = Interpreter::new();
    let result = interp.validate(r#"set bad::x 5"#);
    assert!(!result.warnings.is_empty(), "should warn about unknown context");
    assert!(result.warnings[0].message.contains("no such context"));
}

#[test]
fn context_validate_no_warn_on_good_context() {
    let interp = Interpreter::new();
    let result = interp.validate(r#"
        context cfg { set mode "normal" }
        set cfg::mode "fast"
    "#);
    let ctx_warns: Vec<_> = result.warnings.iter()
        .filter(|w| w.message.contains("context"))
        .collect();
    assert!(ctx_warns.is_empty(), "should not warn: {:?}", ctx_warns);
}
