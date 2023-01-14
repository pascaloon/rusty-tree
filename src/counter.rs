use std::collections::HashMap;
use std::hash::Hash;
pub struct Counter<K>(HashMap<K, i32>)
    where K: Eq + Hash + Clone;


impl<K> Counter<K>
    where K: Eq + Hash + Clone
{
    pub fn new() -> Self {
        Counter(HashMap::new())
    }

    pub fn inc(&mut self, k: &K) {
        self.with_delta(k, 1);
    }

    pub fn dec(&mut self, k: &K) {
        self.with_delta(k, -1);
    }

    fn with_delta(&mut self, k: &K, delta: i32) {
        match self.0.get_mut(k) {
            Some(r) => *r += delta,
            None => {
                self.0.insert(k.clone(), delta);
            },
        }
    }

    pub fn get(&self, k: &K) -> Option<i32> {
        self.0.get(k).copied()
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<K, i32> {
        self.0.iter()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}