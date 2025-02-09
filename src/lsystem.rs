use eframe::emath::Pos2;
use eframe::epaint::pos2;
use std::collections::HashMap;
use std::f32::consts::PI;

// Struct representing a DOL-System, including necessary logic to iterate it
pub struct LSystem {
    pub axiom: String,
    pub result: String,
    pub rules: HashMap<char, String>,
    pub angle: f32,
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
    pub fn new(axiom: char, rules: Vec<&str>, angle: f32) -> Self {
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
            angle,
        }
    }

    // Create vector of lines from system, using turtle commands
    pub fn lines(&self) -> Vec<Vec<Pos2>> {
        let mut turtle = Turtle::new();
        let mut lines = vec![];
        let mut current_line: Vec<Pos2> = vec![pos2(0.0, 0.0)];

        for c in self.result.chars() {
            if c == ']' {
                turtle.pop();
                lines.push(current_line.clone());
                current_line = vec![turtle.pos()]
            } else {
                match c {
                    'x' => {
                        // turtle.forward(1.0);
                    }
                    'f' => {
                        turtle.forward(2.0);
                    }
                    '+' => {
                        turtle.rotate(self.angle);
                    }
                    '-' => {
                        turtle.rotate(-self.angle);
                    }
                    '[' => {
                        turtle.push();
                    }
                    s => panic!("Invalid symbol: {s} found in L-System!"),
                };
                current_line.push(turtle.pos())
            }
        }
        lines.push(current_line.clone());

        lines
    }
}

// Struct used for generating points based off interpeting standard turtle commands
struct Turtle {
    pos: Pos2,
    angle: f32,
    stack: Vec<(Pos2, f32)>,
}

impl Turtle {
    pub fn new() -> Self {
        Self {
            pos: pos2(0.0, 0.0),
            angle: PI / 2.0,
            stack: vec![],
        }
    }
    pub fn forward(&mut self, dist: f32) {
        let dir = pos2(self.angle.cos() * dist, self.angle.sin() * dist);
        self.pos = pos2(self.pos.x + dir.x, self.pos.y + dir.y);
    }

    pub fn pos(&self) -> Pos2 {
        self.pos
    }

    pub fn rotate(&mut self, angle: f32) {
        self.angle += angle.to_radians();
    }

    pub fn push(&mut self) {
        self.stack.push((self.pos, self.angle));
    }

    pub fn pop(&mut self) {
        let (pos, angle) = self
            .stack
            .pop()
            .expect("Error, tried to pop a position that doesn't exist, check rules!");
        self.pos = pos;
        self.angle = angle;
    }
}
