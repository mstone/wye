#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-nil.rs");
    t.pass("tests/02-print.rs");
}