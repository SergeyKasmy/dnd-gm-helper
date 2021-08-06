use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
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

    pub fn r#use(&mut self) -> Result<(), ()> {
        if self.available_after == 0 {
            log::debug!("Using skill {}", self.name);
            self.available_after = self.cooldown;
            Ok(())
        } else {
            log::info!("Skill {} is still on cooldown", self.name);
            Err(())
        }
    }
}
