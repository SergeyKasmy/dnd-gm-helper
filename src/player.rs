use crate::{Skill, Stats, Status};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Players = HashMap<usize, Player>;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Player {
    pub name: String,
    pub stats: Stats,
    pub skills: Vec<Skill>,
    pub statuses: Vec<Status>,
    pub money: i64,
}
