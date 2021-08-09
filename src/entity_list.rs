use std::collections::HashMap;
use crate::id::Uid;

pub trait EntityList {
	type Entity;

	/*
	pub fn new(new_map: HashMap<usize, Player>) -> Players {
		Players {
			map: new_map.into_iter().map(|(id, pl)| (id, pl)).collect(),
			sorted_ids: RefCell::new(None),
		}
	}
	*/

	fn new(map: HashMap<Uid, Self::Entity>) -> Self;
	fn get_map(&self) -> &HashMap<Uid, Self::Entity>;
	fn get_map_mut(&mut self) -> &mut HashMap<Uid, Self::Entity>;

	// TODO: maybe return a ref
	fn sort_ids(&self) -> Vec<Uid>;
	fn invalidate_sorted_ids(&self);

	fn get(&self, id: Uid) -> Option<&Self::Entity> {
		self.get_map().get(&id)
	}

	fn get_mut(&mut self, id: Uid) -> Option<&mut Self::Entity> {
		self.invalidate_sorted_ids();
		self.get_map_mut().get_mut(&id)
	}

	fn push(&mut self, new_val: Self::Entity) -> Uid {
		self.invalidate_sorted_ids();
		let biggest_id = if let Some(num) = self.keys().max() {
			*num + 1.into()
		} else {
			0.into()
		};

		self.insert(biggest_id, new_val);
		biggest_id
	}

	fn insert(&mut self, id: Uid, new_val: Self::Entity) {
		self.invalidate_sorted_ids();
		self.get_map_mut().insert(id, new_val);
	}

	fn remove(&mut self, id: Uid) -> Option<(Uid, Self::Entity)> {
		self.invalidate_sorted_ids();
		self.get_map_mut().remove_entry(&id)
	}

	fn clear(&mut self) {
		self.invalidate_sorted_ids();
		self.get_map_mut().clear();
	}

	fn keys(&self) -> std::collections::hash_map::Keys<Uid, Self::Entity> {
		self.get_map().keys()
	}

	fn len(&self) -> usize {
		self.get_map().len()
	}

	fn is_empty(&self) -> bool {
		self.get_map().is_empty()
	}
}

/*
impl<T: EntityList> Default for T {
	fn default() -> Self {
		T::new(HashMap::new())
	}
}
*/
