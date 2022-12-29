#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-nil.rs");
    t.pass("tests/02-print.rs");
    t.pass("tests/03-add-add.rs");
    t.pass("tests/04-let.rs");
    t.pass("tests/05-format.rs");
    t.pass("tests/06-slice.rs");
    t.pass("tests/07-debug.rs");
    t.pass("tests/08-impl-trait-arg.rs");
}
