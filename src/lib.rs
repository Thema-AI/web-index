pub mod data;
pub mod domain;
pub mod insert;
pub mod path;
pub mod query;

pub fn foobar() -> String {
    "foo".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_foobar_foo_bar_s() {
        assert_eq!(foobar(), "foo".to_string())
    }
}
