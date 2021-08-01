use crate::{Stats, Skill, Status};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Player {
    pub name: String,
    pub stats: Stats,
    pub skills: Vec<Skill>,
    pub statuses: Vec<Status>,
    pub money: i64,
}
