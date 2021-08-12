use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Skill {
	pub name: String,
	pub cooldown: u32,
	pub cooldown_left: u32,
}

impl Skill {
	#[allow(dead_code)]
	pub fn new(name: String, cooldown: u32) -> Skill {
		Skill {
			name,
			cooldown,
			cooldown_left: 0,
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
