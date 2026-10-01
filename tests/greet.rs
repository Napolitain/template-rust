//! Integration tests.

#[test]
fn greets_from_outside() {
    assert_eq!(template::greet("rust"), "Hello, rust!");
}

proptest::proptest! {
    /// Property-based test: proptest generates names and shrinks failures to a minimal case.
    #[test]
    fn greeting_wraps_any_name(name in ".*") {
        let got = template::greet(&name);
        proptest::prop_assert_eq!(got.strip_prefix("Hello, ").and_then(|s| s.strip_suffix('!')), Some(name.as_str()));
    }
}
