use crate::entity_list::EntityList;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

// TODO: use HashMap
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
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

impl fmt::Display for StatusType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Clone, serde::Serialize, serde::Deserialize, PartialEq, Debug)]
pub enum StatusCooldownType {
	Normal,
	OnAttacking,
	OnGettingAttacked,
	Manual,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct Status {
	pub status_type: StatusType,
	pub status_cooldown_type: StatusCooldownType,
	pub duration: u32,
}

#[derive(Clone, Debug)]
pub struct Statuses {
	map: HashMap<usize, Status>,
	sorted_ids: RefCell<Option<Vec<usize>>>,
}

impl Statuses {
	pub fn drain_by_type(&mut self, status_type: StatusCooldownType) {
		self.invalidate_sorted_ids();
		// decrease all statuses duration with the status cooldown type provided
		self.map.iter_mut().for_each(|(_, status)| {
			if status.status_cooldown_type == status_type && status.duration > 0 {
				log::debug!("Drained {:?}", status.status_type);
				status.duration -= 1
			}
		});
		// remove all statuses that have run out = retain all statuses that haven't yet run out
		self.map.retain(|_, status| status.duration > 0);
	}
}

impl EntityList for Statuses {
	type Entity = Status;

	fn new(map: HashMap<usize, Self::Entity>) -> Self {
		Self {
			map,
			sorted_ids: RefCell::new(None),
		}
	}

	fn get_map(&self) -> &HashMap<usize, Self::Entity> {
		&self.map
	}

	fn get_map_mut(&mut self) -> &mut HashMap<usize, Self::Entity> {
		&mut self.map
	}

	fn sort_ids(&self) -> Vec<usize> {
		if self.sorted_ids.borrow().is_none() {
			log::debug!("Sorting player list");
			*self.sorted_ids.borrow_mut() = Some({
				let mut unsorted: Vec<usize> = self.map.iter().map(|(id, _)| *id).collect();
				unsorted.sort_by(|a, b| {
					self.map
						.get(&a)
						.unwrap()
						.status_type
						.to_string()
						.cmp(&self.map.get(&b).unwrap().status_type.to_string())
				});
				unsorted
			});
		}
		match &*self.sorted_ids.borrow() {
			Some(ids) => ids.clone(),
			None => {
				log::error!("Somehow the sorted list of player ids is None even though we should've just created it");
				unreachable!();
			}
		}
	}

	fn invalidate_sorted_ids(&self) {
		*self.sorted_ids.borrow_mut() = None;
	}
}

impl Default for Statuses {
	fn default() -> Self {
		Self::new(HashMap::new())
	}
}

impl Serialize for Statuses {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut smap = serializer.serialize_map(Some(self.map.len()))?;
		for (id, player) in self.map.iter() {
			smap.serialize_entry(id, player)?;
		}
		smap.end()
	}
}

struct StatusesVisitor {
	marker: PhantomData<fn() -> Statuses>,
}

impl StatusesVisitor {
	fn new() -> Self {
		StatusesVisitor {
			marker: PhantomData,
		}
	}
}

impl<'de> Visitor<'de> for StatusesVisitor {
	type Value = Statuses;

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

		Ok(Statuses::new(map))
	}
}

impl<'de> Deserialize<'de> for Statuses {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserializer.deserialize_map(StatusesVisitor::new())
	}
}
