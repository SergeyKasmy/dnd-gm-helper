use crate::entity_list::EntityList;
use crate::id::Uid;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct StatList {
    // TODO: uncomment all serde(flatten) attrs when I figure out how to get rid of "invalid type: string "", expected usize" error
	//#[serde(flatten)]
	map: HashMap<Uid, String>,

	#[serde(skip)]
	sorted_ids: RefCell<Option<Vec<Uid>>>,
}

impl StatList {
	pub fn get_name(&self, id: Uid) -> Option<&str> {
		Some(self.map.get(&id)?.as_str())
	}

	pub fn contains(&self, id: Uid) -> bool {
		self.map.contains_key(&id)
	}
}

impl EntityList for StatList {
	type Entity = String;

	fn new(map: HashMap<Uid, Self::Entity>) -> Self {
		Self {
			map,
			sorted_ids: RefCell::new(None),
		}
	}

	fn get_map(&self) -> &HashMap<Uid, Self::Entity> {
		&self.map
	}

	fn get_map_mut(&mut self) -> &mut HashMap<Uid, Self::Entity> {
		&mut self.map
	}

	fn sort_ids(&self) -> Vec<Uid> {
		if self.sorted_ids.borrow().is_none() {
			log::debug!("Sorting stat list");
			*self.sorted_ids.borrow_mut() = Some({
				let mut unsorted: Vec<Uid> = self.map.iter().map(|(id, _)| *id).collect();
				unsorted.sort_by(|a, b| self.map.get(&a).unwrap().cmp(&self.map.get(&b).unwrap()));
				unsorted
			});
		}
		match &*self.sorted_ids.borrow() {
			Some(ids) => ids.clone(),
			None => {
				log::error!("Somehow the sorted list of status ids is None even though we should've just created it");
				unreachable!();
			}
		}
	}

	fn invalidate_sorted_ids(&self) {
		*self.sorted_ids.borrow_mut() = None;
	}
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Stats {
	//#[serde(flatten)]
	map: HashMap<Uid, i32>,
}

impl Stats {
	pub fn new(mut map: HashMap<Uid, i32>, stat_list: &StatList) -> Stats {
		// ignore all stats that's id doesn't exist
		map.retain(|&id, _| stat_list.contains(id));
		Stats { map }
	}

	pub fn get(&self, id: Uid) -> i32 {
		*self.map.get(&id).unwrap_or(&0)
	}

	/*
	pub fn get_mut(&mut self, id: usize) -> &mut i32 {
		// TODO: check if id is a real stat
		if !self.map.contains_key(&id) {
			self.map.insert(id, 0);
		}

		self.map.get_mut(&id).unwrap()
	}
	*/

	pub fn set(&mut self, id: Uid, new_val: i32) {
		if new_val == 0 {
			self.map.remove(&id);
		} else {
			self.map.insert(id, new_val);
		}
	}
}
