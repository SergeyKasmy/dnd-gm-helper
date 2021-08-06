use crate::skill::Skill;
use crate::stats::Stats;
use crate::status::Status;
use crate::status::StatusCooldownType;
use crate::term::Term;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::rc::Weak;

pub type Hp = u16;

pub enum PlayerState {
    Alive(Hp),
    Dead,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default, Debug)]
pub struct Player {
    // permanent state
    pub name: String,
    pub stats: Stats,
    max_hp: Hp,

    // temporary state
    hp: Hp,
    money: i64,
    pub skills: Vec<Skill>,
    pub statuses: Vec<Status>,
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

    fn get_player_state(&self) -> PlayerState {
        if self.hp == 0 {
            PlayerState::Dead
        } else {
            PlayerState::Alive(self.hp)
        }
    }

    pub fn damage(&mut self, amount: Hp) -> PlayerState {
        if let Some(hp) = self.hp.checked_add(amount) {
            self.hp = hp;
        }

        self.get_player_state()
    }

    pub fn heal(&mut self, mut amount: Hp) -> PlayerState {
        if self.hp + amount > self.max_hp {
            amount = self.max_hp - self.hp;
        }

        self.hp += amount;
        self.get_player_state()
    }

    pub fn manage_money(&mut self, term: &Term) {
        let diff = term.get_money_amount();
        log::debug!("Adding {} money to Player {}", diff, self.name);
        self.money += diff;
    }
}

#[derive(Debug)]
pub struct Players {
    map: HashMap<usize, Rc<Player>>,
    sorted_vec: Option<Vec<(usize, Weak<Player>)>>,
}

impl Players {
    pub fn new(new_map: HashMap<usize, Player>) -> Players {
        Players {
            map: new_map
                .into_iter()
                .map(|(id, pl)| (id, Rc::new(pl)))
                .collect(),
            sorted_vec: None,
        }
    }

    pub fn get(&self, id: usize) -> Option<&Player> {
        let pl = self.map.get(&id);
        log::debug!(
            "Player #{} refs: {:?} strong, {:?} weak",
            id,
            pl.map(|x| Rc::strong_count(x)),
            pl.map(|x| Rc::weak_count(x))
        );
        pl.map(|x| x.as_ref())
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Player> {
        self.sorted_vec = None;
        self.map.get_mut(&id).and_then(|x| Rc::get_mut(x))
    }

    pub fn insert(&mut self, id: usize, new_val: Player) {
        // invalidate sorted vec
        self.sorted_vec = None;
        self.map.insert(id, Rc::new(new_val));
    }

    pub fn remove(&mut self, id: usize) -> Option<(usize, Player)> {
        // invalidate sorted vec
        self.sorted_vec = None;
        // TODO: remove this clone
        self.map
            .remove_entry(&id)
            .map(|(id, pl)| (id, (*pl.as_ref()).clone()))
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<usize, Rc<Player>> {
        self.map.keys()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn as_vec(&mut self) -> &[(usize, Weak<Player>)] {
        if self.sorted_vec.is_none() {
            log::debug!("Sorting player list");
            let mut unsorted_vec = self
                .map
                .iter()
                .map(|(a, b)| (*a, Rc::downgrade(b)))
                .collect::<Vec<(usize, Weak<Player>)>>();
            unsorted_vec.sort_by(|a, b| {
                a.1.upgrade()
                    .unwrap()
                    .name
                    .cmp(&b.1.upgrade().unwrap().name)
            });
            self.sorted_vec = Some(unsorted_vec);
        }
        match &self.sorted_vec {
            Some(vec) => vec.as_slice(),
            None => {
                log::error!("Somehow the sorted vec player list is None even though we should've just created it");
                unreachable!();
            }
        }
    }
}

impl Default for Players {
    fn default() -> Self {
        Players {
            map: HashMap::new(),
            sorted_vec: None,
        }
    }
}

impl Serialize for Players {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut smap = serializer.serialize_map(Some(self.map.len()))?;
        for (id, player) in self.map.iter() {
            smap.serialize_entry(id, &**player)?;
        }
        smap.end()
    }
}

struct PlayersVisitor {
    marker: PhantomData<fn() -> Players>,
}

impl PlayersVisitor {
    fn new() -> Self {
        PlayersVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for PlayersVisitor {
    type Value = Players;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("StatList.map<usize, String>")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((id, pl)) = access.next_entry()? {
            map.insert(id, pl);
        }

        Ok(Players::new(map))
    }
}

impl<'de> Deserialize<'de> for Players {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(PlayersVisitor::new())
    }
}
