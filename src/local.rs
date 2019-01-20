use hashbrown::HashMap;
use hashbrown::hash_map::Entry::{Occupied, Vacant};
use std::hash::Hash;
use std::iter::FromIterator;

pub trait Local {
    fn channel(&self) -> String;
    fn obj(&self) -> String;
}

#[derive(Default)]
pub struct LocalMap<T: Local + Eq + Hash>(HashMap<String, HashMap<String, T>>);

#[allow(dead_code)]
impl<T: Local + Eq + Hash> LocalMap<T> {
    pub fn new() -> Self {
        LocalMap(HashMap::new())
    }
    pub fn with_capacity(capacity: usize) -> LocalMap<T> {
        LocalMap(HashMap::with_capacity(capacity))
    }
    pub fn insert(&mut self, value: T) -> Option<T> {
        match self.0.entry(value.channel()) {
            Occupied(k) => k.into_mut().insert(value.obj(), value),
            Vacant(k)   => k.insert(HashMap::new()).insert(value.obj(), value)
        }
    }
    pub fn contains(&self, channel: &str, user: &str) -> bool {
        match self.0.get(channel) {
            None    => false,
            Some(x) => x.contains_key(user)
        }
    }
    pub fn get(&self, channel: &str, user: &str) -> Option<&T> {
        self.0.get(channel)?.get(user)
    }
    pub fn get_mut(&mut self, channel: &str, user: &str) -> Option<&mut T> {
        self.0.get_mut(channel)?.get_mut(user)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn remove(&mut self, channel: &str, user: &str) -> Option<T> {
        self.0.get_mut(channel)?.remove(user)
    }
    pub fn remove_by(&mut self, t: &T) -> Option<T> {
        self.remove(&t.channel(), &t.obj())
    }
}

impl<T: Local + Eq + Hash> FromIterator<T> for LocalMap<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> LocalMap<T> {
        let iter = iterable.into_iter();
        let hint = iter.size_hint().0;

        let mut localmap = LocalMap::with_capacity(hint);
        for t in iter {
            localmap.insert(t);
        }

        localmap
    }
}
