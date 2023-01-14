use std::{collections::HashMap, hash::Hash};
use smallvec::{SmallVec, smallvec};

pub struct MultiMap<K, V>
    where K: Eq + Hash + Clone
{
    inner: HashMap<K, SmallVec<[V; 1]>>
}

impl<K, V> MultiMap<K, V>
    where K: Eq + Hash + Clone
{
    pub fn new() -> MultiMap<K, V> {
        MultiMap { inner: HashMap::new() }
    }

    // `V`: `: smallvec::Array`
    pub fn get(&self, key: &K) -> Option<&[V]> {
        self.inner.get(key).map(|x| x.as_slice())
    }

    pub fn insert(&mut self, key: &K, value: V) {
        match self.inner.get_mut(key) {
            Some(v) => {
                v.push(value);
            },
            None => {
                self.inner.insert(key.clone(), smallvec![value]);
            },
        };
    }

    pub fn iter(&self) -> Vec<(&K, &[V])> {
        self.inner
        .iter()
        .map(|(k, v)| (k, v.as_slice()))
        .collect()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get() {
        let mut map: MultiMap<i32, i32> = MultiMap::new();
        map.insert(&1, 2);
        map.insert(&1, 3);
        map.insert(&2, 4);

        let r = map.get(&1).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0], 2);
        assert_eq!(r[1], 3);

        let r = map.get(&2).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0], 4);
    }
}