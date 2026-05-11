//! Compile-fail tests enforcing the typestate and capability guarantees
//! of `ClientBuilder` and the `define_model!` macro.
//!
//! To regenerate expected stderr after intentional API/typestate changes:
//!     TRYBUILD=overwrite cargo test -p ai-lib-core --test compile_fail

#[test]
fn compile_fail_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
