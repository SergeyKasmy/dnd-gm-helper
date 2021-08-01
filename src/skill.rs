use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Skill {
    pub name: String,
    pub cooldown: u32,
    pub available_after: u32,
}

impl Skill {
    #[allow(dead_code)]
    pub fn new(name: String, cooldown: u32) -> Skill {
        Skill {
            name,
            cooldown,
            available_after: 0,
        }
    }
}
