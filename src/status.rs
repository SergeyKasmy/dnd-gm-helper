use serde::{Deserialize, Serialize};

// TODO: use HashMap
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum StatusType {
	Discharge,
	FireAttack,
	FireShield,
	IceShield,
	Blizzard,
	Fusion,
	Luck,

	Knockdown,
	Poison,
	Stun,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum StatusCooldownType {
	Normal,
	Attacking,
	Attacked,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Status {
	pub status_type: StatusType,
	pub status_cooldown_type: StatusCooldownType,
	pub duration: u32,
}
