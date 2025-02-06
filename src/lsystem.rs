use std::collections::HashMap;
use std::f32::consts::PI;

// Struct representing a DOL-System, including necessary logic to iterate it
pub struct LSystem {
    pub axiom: String,
    pub result: String,
    pub rules: HashMap<char, String>,
}

impl LSystem {
    pub fn iterate(&mut self, n: usize) {
        for _ in 0..n {
            let mut result = String::new();
            for c in self.result.chars() {
                let replacement = self.rules.get(&c).cloned().unwrap_or(c.to_string());
                result.push_str(replacement.as_str());
            }
            self.result = result;
        }
    }

    // Parse rules as strings into rule set
    pub fn new(axiom: char, rules: Vec<&str>) -> Self {
        let mut rule_map = HashMap::new();
        // TODO This is some ugly parsing, make it better
        rules.iter().for_each(|rule| {
            let components: Vec<&str> = rule.split("->").take(2).collect();
            let key = components[0].chars().collect::<Vec<_>>()[0];
            rule_map.insert(key, components[1].to_string());
        });
        Self {
            result: axiom.to_string(),
            axiom: axiom.to_string(),
            rules: rule_map,
        }
    }

    // Create vector of lines from system, using turtle commands
    pub fn lines(&self) -> Vec<Vec<(f32, f32)>> {
        let mut turtle = Turtle::new();
        let mut lines = vec![];
        let mut current_line: Vec<(f32, f32)> = vec![(0.0, 0.0)];

        for c in self.result.chars() {
            if c == ']' {
                turtle.pop();
                turtle.rotate(-PI / 4.0);
                lines.push(current_line.clone());
                current_line = vec![turtle.pos()]
            } else {
                match c {
                    '0' => {
                        turtle.forward(1.0);
                    }
                    '1' => {
                        turtle.forward(1.0);
                    }
                    '[' => {
                        turtle.push();
                        turtle.rotate(PI / 4.0);
                    }
                    _ => panic!("Invalid symbol found in L-System!"),
                };
                current_line.push(turtle.pos())
            }
        }
        lines.push(current_line.clone());

        lines
    }
}

// Struct used for generating points based off interpeting standard turtle commands
pub struct Turtle {
    pos: (f32, f32),
    angle: f32,
    stack: Vec<(f32, f32, f32)>,
}

impl Turtle {
    pub fn new() -> Self {
        Self {
            pos: (0.0, 0.0),
            angle: PI / 2.0,
            stack: vec![],
        }
    }
    pub fn forward(&mut self, dist: f32) {
        let dir = (self.angle.cos() * dist, self.angle.sin() * dist);
        self.pos = (self.pos.0 + dir.0, self.pos.1 + dir.1);
    }

    pub fn pos(&self) -> (f32, f32) {
        self.pos
    }

    pub fn rotate(&mut self, angle: f32) {
        self.angle += angle;
    }

    pub fn push(&mut self) {
        self.stack.push((self.pos.0, self.pos.1, self.angle));
    }

    pub fn pop(&mut self) {
        let (x, y, angle) = self
            .stack
            .pop()
            .expect("Error, tried to pop a position that doesn't exist, check rules!");
        self.pos = (x, y);
        self.angle = angle;
    }
}
