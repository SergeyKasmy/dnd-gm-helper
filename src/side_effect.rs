use std::fmt;

use crate::status::Status;
use serde::{Deserialize, Serialize};

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

impl fmt::Display for SideEffect {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.r#type)
	}
}

impl fmt::Display for SideEffectType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SideEffectType::AddsStatus(status) => write!(f, "Adds status ({})", status.status_type),
			SideEffectType::UsesSkill => write!(f, "Uses skill"),
		}
	}
}

impl fmt::Display for SideEffectAffects {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				SideEffectAffects::Themselves => "Self",
				SideEffectAffects::SomeoneElse => "Someone else",
				SideEffectAffects::Both => "Both",
			}
		)
	}
}
