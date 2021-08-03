use crate::skill::Skill;
use crate::stats::Stats;
use crate::status::Status;
use crate::status::StatusCooldownType;
use crate::term::Term;
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

impl Player {
    pub fn turn(&mut self) {
        log::debug!("{}'s turn has ended", self.name);
        self.skills.iter_mut().for_each(|skill| {
            if skill.available_after > 0 {
                skill.available_after -= 1
            }
        });
        self.drain_status(StatusCooldownType::Normal);
    }

    pub fn add_status(&mut self, status: Status) {
        self.statuses.push(status);
    }

    pub fn drain_status(&mut self, status_type: StatusCooldownType) {
        log::debug!(
            "Draining statuses for {} with type {:?}",
            self.name,
            status_type
        );
        // decrease all statuses duration with the status cooldown type provided
        self.statuses.iter_mut().for_each(|status| {
            if status.status_cooldown_type == status_type && status.duration > 0 {
                log::debug!("Drained {:?}", status.status_type);
                status.duration -= 1
            }
        });
        // remove all statuses that have run out = retain all statuses that haven't yet run out
        self.statuses.retain(|status| status.duration > 0);
    }

    pub fn manage_money(&mut self, term: &Term) {
        let diff = term.get_money_amount();
        log::debug!("Adding {} money to Player {}", diff, self.name);
        self.money += diff;
    }
}
