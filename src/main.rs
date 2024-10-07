fn greeter() -> String {
    "hello world".to_string()
}
pub fn main() {
    println!("{}", greeter());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greeter_likes_whole_world() {
        assert_eq!(greeter(), "hello world")
    }
}
