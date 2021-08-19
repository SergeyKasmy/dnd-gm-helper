use crate::status::Status;
use serde::{Deserialize, Serialize};

/*
impl fmt::Display for SideEffect {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				SideEffect::AddsStatus => "Adds status",
				SideEffect::UsesSkill => "Uses skill",
			}
		)
	}
}
*/

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SideEffect {
	pub r#type: SideEffectType,
	pub affects: SideEffectAffects,
	pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SideEffectType {
	AddsStatus(Status),
	UsesSkill,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SideEffectAffects {
	Themselves,
	SomeoneElse,
	Both,
}
