use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SideEffect {
	AddsStatus,
	UsesSkill,
}

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
