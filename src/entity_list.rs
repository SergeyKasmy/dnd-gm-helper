use crate::id::Uid;
use indexmap::IndexMap;

pub trait EntityList {
	type Entity;

	fn new(map: IndexMap<Uid, Self::Entity>) -> Self;
	fn get_map(&self) -> &IndexMap<Uid, Self::Entity>;
	fn get_map_mut(&mut self) -> &mut IndexMap<Uid, Self::Entity>;

	fn sort(&mut self);

	fn get(&self, id: Uid) -> Option<&Self::Entity> {
		// TODO: is it safe not to sort it after it may have been possible that the user changed
		// the sorting property with get_map_mut()
		//self.sort();
		self.get_map().get(&id)
	}

	fn get_mut(&mut self, id: Uid) -> Option<&mut Self::Entity> {
		self.sort();
		self.get_map_mut().get_mut(&id)
	}

	fn push(&mut self, new_val: Self::Entity) -> Uid {
		let biggest_id = if let Some(num) = self.get_map().keys().max() {
			*num + 1.into()
		} else {
			0.into()
		};

		self.insert(biggest_id, new_val);
		self.sort();
		biggest_id
	}

	fn insert(&mut self, id: Uid, new_val: Self::Entity) {
		self.get_map_mut().insert(id, new_val);
		self.sort();
	}

	fn remove(&mut self, id: Uid) -> Option<(Uid, Self::Entity)> {
		let removed = self.get_map_mut().remove_entry(&id);
		self.sort();
		removed
	}

	fn clear(&mut self) {
		self.get_map_mut().clear();
	}

	/*
	fn keys(&self) -> std::collections::hash_map::Keys<Uid, Self::Entity> {
		self.get_map().keys()
	}
	*/

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
