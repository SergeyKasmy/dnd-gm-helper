use serde::{Deserialize, Serialize};
use crate::side_effect::SideEffect;

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Skill {
	pub name: String,
	pub cooldown: u32,
	pub cooldown_left: u32,
    pub side_effect: Option<SideEffect>,
}

impl Skill {
	#[allow(dead_code)]
	pub fn new(name: String, cooldown: u32, side_effect: Option<SideEffect>) -> Skill {
		Skill {
			name,
			cooldown,
			cooldown_left: 0,
            side_effect
		}
	}

	pub fn r#use(&mut self) -> Result<(), ()> {
		if self.cooldown_left == 0 {
			log::debug!("Using skill {}", self.name);
			self.cooldown_left = self.cooldown;
			Ok(())
		} else {
			log::info!("Skill {} is still on cooldown", self.name);
			Err(())
		}
	}
}
