
// Define functions with native Rust types.
// The macro will generate wrappers that convert Vec<Val> to these types.

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn sum(a: Vec<i64>) -> i64 {
    a.iter().sum()
}

fn concat(a: String, b: String) -> String {
    format!("{}{}", a, b)
}

fn concat2(a: Vec<String>) -> String {
    a.join("")
}

fn split(s: String, delimiter: String) -> Vec<String> {
    s.split(&delimiter).map(|s| s.to_string()).collect()
}

fn length(s: String) -> i64 {
    s.len() as i64
}

fn is_empty(s: String) -> bool {
    s.is_empty()
}

fn is_empty2(s: f64) -> bool {
    s.is_nan()
}