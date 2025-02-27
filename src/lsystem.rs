use eframe::emath::Pos2;
use eframe::epaint::pos2;
use std::collections::HashMap;
use std::f32::consts::PI;
use egui::TextBuffer;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;
use rand::rng;

// Struct representing a DOL-System, including necessary logic to iterate it
pub struct LSystem {
    pub axiom: String,
    pub results: Vec<String>,
    pub results_aux: Vec<String>, // Used to recompute plant while still being readable
    pub iterations: usize,
    pub current_iteration: usize,
    pub rules: HashMap<char, Rule>,
}

#[derive(Clone)]
struct Rule {
    key: char,
    replacements: Vec<String>,
    probabilities: WeightedIndex<f32>
}

impl Rule {
    pub fn new(text: &str) -> Self {
        let components: Vec<&str> = text.split("->").take(2).collect();
        let key = components[0].chars().collect::<Vec<_>>()[0];
        let mut replacements = vec![];
        let mut probabilities = vec![];
        for rule in components[1].split(",") {
            let splits: Vec<&str> = rule.split(":").take(2).collect();
            let out = splits[0];
            let prob = if let Some(prob) = splits.get(1) {
                prob.parse::<f32>().expect("Expected a float probability")
            } else {
                1.0
            };
            replacements.push(out.to_string());
            probabilities.push(prob);
        }

        Self {
            key,
            replacements,
            probabilities: WeightedIndex::new(probabilities).expect("Invalid Probabilities!"),
        }
    }
}

impl LSystem {
    pub fn iterate(&mut self, n: usize) {
        for _ in 0..n {
            let mut result = String::new();
            for c in self.results.last().unwrap().chars() {
                let str = self.replace(c);
                result.push_str(str.as_str());
            }
            self.iterations += 1;
            self.current_iteration = self.iterations;
            self.results.push(result);
        }
    }

    fn replace(&self, c: char) -> String {
        if let Some(rule) = self.rules.get(&c) {
            let mut rng = rng();
            // Randomly sample the replacements using weighted index
            rule.replacements[rule.probabilities.sample(&mut rng)].clone()
        } else {
            c.to_string()
        }
    }

    pub fn iterate_aux(&mut self, n: usize) {
        for _ in 0..n {
            let mut result = String::new();
            for c in self.results_aux.last().unwrap().chars() {
                let str = self.replace(c);
                result.push_str(str.as_str());
            }
            self.results_aux.push(result);
        }
    }

    /// Recompute the stochastic generation if it exists.
    pub fn recompute(&mut self) {
        self.results_aux.push(self.axiom.clone());
        self.iterate_aux(self.iterations);
        std::mem::swap(&mut self.results, &mut self.results_aux);
        self.results_aux.clear();
    }

    // Parse rules as strings into rule set
    pub fn new(axiom: &str, rules: Vec<&str>) -> Self {
        let mut rule_map = HashMap::new();
        // TODO This is some ugly parsing, make it better
        rules.iter().for_each(|rule| {
            let rule = Rule::new(rule);
            if rule.key == 'x' {
                // X and L should have the same rule,
                // as they are treated the same, except for generating leaves
                let mut new_rule = rule.clone();
                new_rule.key = 'l';
                rule_map.insert('l', new_rule);
            }
            rule_map.insert(rule.key, rule);
        });
        Self {
            results: vec![axiom.to_string()],
            axiom: axiom.to_string(),
            rules: rule_map,
            iterations: 0,
            current_iteration: 0,
            results_aux: vec![],
        }
    }

    // RL encoded version of string, only encoding 'f' runs.
    // Also removes x nodes, as these are ignored in drawing.
    // Encoded lines for 6 iteration system is a 17% reduction
    // TODO! Could this parsing could be made cleaner with match?
    pub fn encoded(&self) -> Vec<String> {
        let mut vec = vec![];
        let mut out = "".to_string();
        let mut occurrences = 1;
        let mut iter = self.results[self.current_iteration]
            .chars()
            .filter(|c| *c != 'x');
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
        // Handling the case where the string ends with a run
        if out == *"" {
            vec.push(occurrences.to_string());
        } else {
            vec.push(out);
        }

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

    pub fn angle(&self) -> f32 {
        self.angle
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
