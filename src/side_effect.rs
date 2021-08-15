use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum SideEffect {
    AddsStatus,
    UsesSkill,
}
