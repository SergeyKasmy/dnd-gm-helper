use serde::{Serialize, Deserialize};

// TODO: reimplement using HashMap with StatType as keys
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Stats {
    pub strength: i64,
    pub dexterity: i64,
    pub poise: i64,
    pub wisdom: i64,
    pub intelligence: i64,
    pub charisma: i64,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum StatType {
    Strength,
    Dexterity,
    Poise,
    Wisdom,
    Intelligence,
    Charisma,
}
