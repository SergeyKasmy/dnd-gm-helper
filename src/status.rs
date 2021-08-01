use serde::{Serialize, Deserialize};

pub type Statuses = Vec<Status>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub status_type: StatusType,
    pub status_cooldown_type: StatusCooldownType,
    pub duration: u32,
}

// TODO: use HashMap
#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum StatusCooldownType {
    Normal,
    Attacking,
    Attacked,
}
