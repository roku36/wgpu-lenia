#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct Rule {
    pub born: u16,
    pub survives: u16,
    pub initial_density: u8,
    name: &'static str,
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl Rule {
    pub(crate) fn rule_array(&self) -> [u32; 2] {
        [u32::from(self.born), u32::from(self.survives)]
    }

    pub fn name(&self) -> String {
        let mut born = String::from("B");
        let mut survives = String::from("S");
        for i in 0..9 {
            if self.born & (1 << i) != 0 {
                born.push_str(&format!("{i}"));
            }
            if self.survives & (1 << i) != 0 {
                survives.push_str(&format!("{i}"));
            }
        }
        format!("{} {}/{}", self.name, born, survives)
    }
}

pub static RULES: [Rule; 2] = [
    Rule {
        born: 0b1000,
        survives: 0b1100,
        name: "Conway's Life",
        initial_density: 12,
    },
    Rule {
        born: 0b0_1011_1000,
        survives: 0b1_0111_0000,
        name: "Gems",
        initial_density: 15,
    },
];
