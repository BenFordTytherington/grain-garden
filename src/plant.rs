use crate::lsystem::LSystem;
use eframe::epaint::Color32;

pub struct Plant {
    pub system: LSystem,
    pub branch_colour: Color32,
    pub leaf_colours: Vec<Color32>,
    pub name: String,
}

impl Plant {
    pub fn tree_1() -> Self {
        let mut system = LSystem::new("x", vec!["x->f+[[x]-l]-f[-fx]+l", "f->ff"]);
        system.iterate(6);
        Self {
            system,
            branch_colour: Color32::from_hex("#63413f").unwrap(),
            leaf_colours: vec![
                Color32::from_hex("#4d5e21").unwrap(),
                Color32::from_hex("#374529").unwrap(),
                Color32::from_hex("#364f33").unwrap(),
            ],
            name: "Tree 1".to_string(),
        }
    }

    pub fn tree_2() -> Self {
        let mut system = LSystem::new("x", vec!["x->f-[[+l]+x]+f[+fx]-x", "f->ff"]);
        system.iterate(6);
        Self {
            system,
            branch_colour: Color32::from_hex("#403125").unwrap(),
            leaf_colours: vec![
                Color32::from_hex("#7a2d30").unwrap(),
                Color32::from_hex("#8f422c").unwrap(),
                Color32::from_hex("#995325").unwrap(),
            ],
            name: "Tree 2".to_string(),
        }
    }

    pub fn tree_3() -> Self {
        let mut system = LSystem::new("x", vec!["x->f[-x]f[+x]-[+x]-l", "f->ff"]);
        system.iterate(5);
        Self {
            system,
            branch_colour: Color32::from_hex("#9e6a55").unwrap(),
            leaf_colours: vec![
                Color32::from_hex("#8c7f0b").unwrap(),
                Color32::from_hex("#a67c23").unwrap(),
                Color32::from_hex("#364f33").unwrap(),
            ],
            name: "Tree 3".to_string(),
        }
    }

    // Stochastic system
    pub fn tree_4() -> Self {
        let mut system = LSystem::new("x", vec!["x->f-[[+l]+x]+f[+fx]-x:0.5,f+[[-l+l]-x]-f[+x+l]:0.25,[[f-x]+[x]]+ff:0.25", "f->ff:0.8,fff:0.1,f:0.1"]);
        system.iterate(7);
        Self {
            system,
            branch_colour: Color32::from_hex("#403125").unwrap(),
            leaf_colours: vec![
                Color32::from_hex("#8f422c").unwrap(),
                Color32::from_hex("#995325").unwrap(),
                Color32::from_hex("#ab6c2c").unwrap(),
                Color32::from_hex("#996d20").unwrap(),
            ],
            name: "Tree 4".to_string(),
        }
    }
}
