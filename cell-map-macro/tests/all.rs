//! Perform all tests using trybuild

#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/layer-pass.rs");
    t.compile_fail("tests/layer-fail.rs");
}
