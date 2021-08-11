// TODO: maybe use a generic struct instead of a trait????
use crate::id::Id;
use crate::id::OrderNum;
use crate::id::Uid;

#[macro_export]
macro_rules! id_list {
    ($t:ty) => {
        indexmap::IndexMap<Uid, $t>
    }
}

pub trait IdListImpl {
	type Value: Id;

	fn new(list: id_list!(Self::Value)) -> Self;
	fn get_list(&self) -> &id_list!(Self::Value);
	fn get_list_mut(&mut self) -> &mut id_list!(Self::Value);
}

#[macro_export]
macro_rules! impl_idlist_default {
	($i:ident, $value_type:ty) => {
		impl crate::list::IdListImpl for $i {
			type Value = $value_type;

			fn new(list: crate::id_list!(Self::Value)) -> Self {
				Self { list }
			}

			fn get_list(&self) -> &crate::id_list!(Self::Value) {
				&self.list
			}

			fn get_list_mut(&mut self) -> &mut crate::id_list!(Self::Value) {
				&mut self.list
			}
		}
	};
}

pub trait IdList: IdListImpl {
	fn sort(&mut self);

	fn get(&self, id: Uid) -> Option<&Self::Value> {
		// TODO: is it safe not to sort it after it may have been possible that the user changed
		// the sorting property with get_map_mut()
		//self.sort();
		self.get_list().get(&id)
	}

	fn get_mut(&mut self, id: Uid) -> Option<&mut Self::Value> {
		self.sort();
		self.get_list_mut().get_mut(&id)
	}

	fn get_by_index(&self, num: OrderNum) -> Option<(&Uid, &Self::Value)> {
		self.get_list().get_index(*num)
	}

	fn get_index_of(&self, id: Uid) -> Option<OrderNum> {
		self.get_list().get_index_of(&id).map(|x| OrderNum(x))
	}

	// TODO: don't allocate for no reason
	//fn iter(&self) -> impl Iterator<Item = Self::Value> {
	fn iter(&self) -> Box<dyn Iterator<Item = (&Uid, &Self::Value)> + '_> {
		Box::new(self.get_list().iter())
	}

	fn push(&mut self, new_val: Self::Value) -> Uid {
		let biggest_id = if let Some(num) = self.get_list().keys().max() {
			*num + 1.into()
		} else {
			0.into()
		};

		self.insert(biggest_id, new_val);
		self.sort();
		biggest_id
	}

	fn insert(&mut self, id: Uid, mut new_val: Self::Value) {
		*new_val.id() = Some(id);
		self.get_list_mut().insert(id, new_val);
		self.sort();
	}

	fn remove(&mut self, id: Uid) -> Option<(Uid, Self::Value)> {
		let removed = self.get_list_mut().remove_entry(&id);
		self.sort();
		removed
	}

	fn clear(&mut self) {
		self.get_list_mut().clear();
	}

	fn len(&self) -> usize {
		self.get_list().len()
	}

	fn is_empty(&self) -> bool {
		self.get_list().is_empty()
	}
}
