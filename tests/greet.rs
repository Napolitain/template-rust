//! Integration tests.

#[test]
fn greets_from_outside() {
    assert_eq!(template::greet("rust"), "Hello, rust!");
}
