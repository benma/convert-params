#[test]
fn all() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01_basic.rs");
}
