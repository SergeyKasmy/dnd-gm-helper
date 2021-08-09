use std::collections::HashMap;
use derive_more::{From, Add, Display, Deref};
use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, From, Add, Display, Deref)]
pub struct Uid(pub usize);

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, From, Add, Display, Deref)]
pub struct OrderNum(pub usize);

impl Uid {
    pub fn to_order_num(&self, map: &HashMap<OrderNum, Uid>) -> Option<OrderNum> {
        map.iter().find_map(|(&k, &v)| if v == *self { Some(k) } else { None })
    }
}

impl OrderNum {
    pub fn to_uid(&self, map: &HashMap<OrderNum, Uid>) -> Option<Uid> {
        map.get(&self).map(|x| *x)
    }
}
