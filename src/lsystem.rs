use eframe::emath::Pos2;
use eframe::epaint::pos2;
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
    pub fn new(axiom: &str, rules: Vec<&str>) -> Self {
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

    // RL encoded version of string, only encoding 'f' runs.
    // Also removes x nodes, as these are ignored in drawing.
    pub fn encoded(&self) -> Vec<String> {
        let mut vec = vec![];
        let mut out = "".to_string();
        let mut occurrences = 1;
        let mut iter = self.result.chars().filter(|c| *c != 'x');
        let mut last = iter.next().expect("No characters to encode");

        for c in iter {
            if c == last && last == 'f' {
                // Currently in a run / started a run
                if out != *"" {
                    vec.push(out.clone());
                }
                out = "".to_string();
                occurrences += 1;
            } else if occurrences == 1 {
                // 1 length run (skip)
                out.push(last);
            } else {
                // A run ended
                vec.push(occurrences.to_string());
                occurrences = 1; // Reset counter
            }
            last = c;
        }
        vec.push(out);

        vec
    }
}

// Struct used for generating lines based off interpreting standard turtle commands
pub struct Turtle {
    pos: Pos2,
    angle: f32,
    pub width: f32,
    base_width: f32,
    min_width: f32,
    width_falloff: f32,
    stack: Vec<(Pos2, f32, f32)>,
}

impl Turtle {
    pub fn new(width: f32, min_width: f32, width_falloff: f32) -> Self {
        Self {
            pos: pos2(0.0, 0.0),
            angle: PI / 2.0,
            width,
            base_width: width,
            min_width,
            width_falloff,
            stack: vec![],
        }
    }
    pub fn forward(&mut self, dist: f32) {
        let dir = pos2(self.angle.cos() * dist, self.angle.sin() * dist);
        self.pos = pos2(self.pos.x + dir.x, self.pos.y + dir.y);
    }

    // Returns a position and a width
    pub fn get(&self) -> (Pos2, f32) {
        (self.pos, self.width)
    }

    pub fn rotate(&mut self, angle: f32) {
        self.angle += angle.to_radians();
        let decrease = (self.base_width - self.min_width) * self.width_falloff * 0.25;
        self.width = (self.width - decrease).max(self.min_width);
    }

    pub fn push(&mut self) {
        self.stack.push((self.pos, self.angle, self.width));
    }

    pub fn pop(&mut self) {
        let (pos, angle, width) = self
            .stack
            .pop()
            .expect("Error, tried to pop a position that doesn't exist, check rules!");
        self.pos = pos;
        self.angle = angle;
        self.width = width;
    }
}
