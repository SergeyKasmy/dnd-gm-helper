use serde::{Deserialize, Serialize};

use crate::id::Uid;

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

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct SideEffect {
	pub r#type: SideEffectType,
	pub affects: SideEffectAffects,
	pub description: String,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub enum SideEffectType {
	#[default]
	AddsStatus,
	UsesSkill,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub enum SideEffectAffects {
	#[default]
	Themselves,
	SomeoneElse(Uid),
	Both(Uid),
}
