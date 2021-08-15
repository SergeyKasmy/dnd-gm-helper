use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SideEffect {
	AddsStatus,
	UsesSkill,
}
