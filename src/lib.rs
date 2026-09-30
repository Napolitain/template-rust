//! Minimal library.

/// Returns a greeting for `name`.
#[must_use]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::greet;

    #[test]
    fn greets() {
        assert_eq!(greet("world"), "Hello, world!");
    }
}
