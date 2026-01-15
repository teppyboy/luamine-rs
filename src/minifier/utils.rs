use std::collections::HashMap;

/// Generates the next shortest variable name in sequence (a, b, c, ..., aa, ab, ac, ...)
fn generate_next_name(current_index: usize) -> String {
    let mut name = String::new();
    let mut n = current_index;

    loop {
        let remainder = n % 26;
        name.insert(0, ((remainder as u8) + b'a') as char);
        n /= 26;
        if n == 0 {
            break;
        }
        n -= 1; // Adjust for 0-indexing in the next iteration
    }

    name
}

/// Mutates the provided HashMap by mapping each unique key to a unique shortest variable name
/// The keys are mapped to names like a, b, c, ..., aa, ab, ac, ...
pub fn generate_shortest_names(map: &mut HashMap<String, String>, key: String) {
    let index = map.len();
    map.insert(key, generate_next_name(index));
}
