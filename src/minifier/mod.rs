struct Minifier {
    code: String,
}

impl Minifier {
    fn new(code: &str) -> Self {
        Minifier {
            code: String::from(code),
        }
    }

    fn minify(&self) -> String {
        return "".to_string();
    }
}
