use crate::STAT_LIST;
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

/*
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum StatType {
    Strength,
    Dexterity,
    Poise,
    Wisdom,
    Intelligence,
    Charisma,
}
*/

#[derive(Clone, Debug)]
pub struct StatList {
    map: HashMap<usize, String>,
}

impl StatList {
    pub fn new(map: HashMap<usize, String>) -> Self {
        StatList { map }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn get_name(&self, id: usize) -> &str {
        // TODO: check if exists. May crash otherwise
        self.map.get(&id).unwrap().as_str()
    }

    pub fn contains(&self, id: usize) -> bool {
        self.map.contains_key(&id)
    }

    pub fn as_vec(&self) -> Vec<(&usize, &str)> {
        let mut vec = self
            .map
            .iter()
            .map(|(id, name)| (id, name.as_str()))
            .collect::<Vec<(&usize, &str)>>();
        vec.sort_by(|a, b| a.1.cmp(b.1));
        vec
    }
}

impl Serialize for StatList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut smap = serializer.serialize_map(Some(self.map.len()))?;
        for (id, name) in self.map.iter() {
            smap.serialize_entry(id, name)?;
        }
        smap.end()
    }
}

struct StatListVisitor {
    marker: PhantomData<fn() -> StatList>,
}

impl StatListVisitor {
    fn new() -> Self {
        StatListVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for StatListVisitor {
    type Value = StatList;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("StatList.map<usize, String>")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut stat_list = StatList {
            map: HashMap::with_capacity(access.size_hint().unwrap_or(0)),
        };

        while let Some((id, name)) = access.next_entry()? {
            stat_list.map.insert(id, name);
        }

        Ok(stat_list)
    }
}

impl<'de> Deserialize<'de> for StatList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(StatListVisitor::new())
    }
}

#[derive(Clone, Default, Debug)]
pub struct Stats {
    // TODO: mb use a &str instead
    map: HashMap<usize, i32>,
}

impl Serialize for Stats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut smap = serializer.serialize_map(Some(self.map.len()))?;
        for (id, name) in self.map.iter() {
            smap.serialize_entry(id, name)?;
        }
        smap.end()
    }
}

struct StatsVisitor {
    marker: PhantomData<fn() -> Stats>,
}

impl StatsVisitor {
    fn new() -> Self {
        StatsVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for StatsVisitor {
    type Value = Stats;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Stats.map<usize, i32>")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((id, val)) = access.next_entry()? {
            map.insert(id, val);
        }

        Ok(Stats::new(map))
    }
}

impl<'de> Deserialize<'de> for Stats {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(StatsVisitor::new())
    }
}

impl Stats {
    pub fn new(mut map: HashMap<usize, i32>) -> Stats {
        // ignore all stats that's id doesn't exist
        map.retain(|&id, _| STAT_LIST.lock().unwrap().contains(id));
        Stats { map }
    }

    pub fn get(&self, id: usize) -> i32 {
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

    pub fn set(&mut self, id: usize, new_val: i32) {
        if new_val == 0 {
            self.map.remove(&id);
        } else {
            self.map.insert(id, new_val);
        }
    }
}
