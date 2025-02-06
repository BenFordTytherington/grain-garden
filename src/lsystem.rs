use std::collections::HashMap;

pub struct LSystem {
    pub axiom: String,
    pub result: String,
    pub rules: HashMap<char, String>
}

impl LSystem {
    pub fn iterate(&mut self, n: usize) {
        for _ in (0..n) {
            let mut result = String::new();
            for c in self.result.chars() {
                let replacement = self.rules.get(&c).cloned().unwrap_or(c.to_string());
                result.push_str(replacement.as_str());
            }
            self.result = result;
            println!("{}", self.result);
        }
    }
}