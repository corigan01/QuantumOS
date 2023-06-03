/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use crate::heapless_vector::{HeaplessVec, HeaplessVecErr};
use core::slice::Iter;

pub struct HeaplessMap<KeyType: PartialEq, ValueType, const QTY: usize> {
    keys: HeaplessVec<KeyType, QTY>,
    values: HeaplessVec<ValueType, QTY>,
}

impl<KeyType: PartialEq, ValueType, const QTY: usize> HeaplessMap<KeyType, ValueType, QTY> {
    pub fn new() -> Self {
        Self {
            keys: HeaplessVec::new(),
            values: HeaplessVec::new(),
        }
    }

    pub fn insert(&mut self, key: KeyType, value: ValueType) -> Result<(), HeaplessVecErr> {
        self.keys.push_within_capacity(key)?;
        self.values.push_within_capacity(value)?;

        Ok(())
    }

    pub fn contains_key(&self, key: &KeyType) -> bool {
        self.keys.iter().any(|value| key == value)
    }

    pub fn remove(&mut self, key: &KeyType) {
        let index = if let Some(idx) = self.keys.find_index_of(key) {
            idx
        } else {
            return;
        };

        self.keys.remove(index);
        self.values.remove(index);
    }

    pub fn get(&self, key: &KeyType) -> Option<&ValueType> {
        let index = self.keys.find_index_of(key)?;

        Some(self.values.get(index).unwrap())
    }

    pub fn get_mut(&mut self, key: &KeyType) -> Option<&mut ValueType> {
        let index = self.keys.find_index_of(key)?;

        Some(self.values.get_mut(index).unwrap())
    }
}

#[cfg(test)]
pub mod test {
    use crate::heapless_map::HeaplessMap;
    use crate::heapless_vector::HeaplessVecErr;

    #[test]
    fn test_adding_keys() {
        let mut map: HeaplessMap<usize, &str, 4> = HeaplessMap::new();

        map.insert(0, "zero").unwrap();
        map.insert(1, "one").unwrap();
        map.insert(2, "two").unwrap();

        assert_eq!(map.get(&0), Some(&"zero"));
        assert_eq!(map.get(&1), Some(&"one"));
        assert_eq!(map.get(&2), Some(&"two"));
    }

    #[test]
    fn test_string_keys() {
        let mut map: HeaplessMap<&str, usize, 4> = HeaplessMap::new();

        map.insert("one", 1).unwrap();
        map.insert("two", 2).unwrap();
        map.insert("three", 3).unwrap();

        assert_eq!(map.get(&"three").unwrap(), &3);
        assert_eq!(map.get(&"one").unwrap(), &1);
        assert_eq!(map.get(&"zero"), None);
        assert_eq!(map.get(&"two"), Some(&2));
    }

    #[test]
    fn over_and_under_size_test() {
        let mut map: HeaplessMap<usize, usize, 10> = HeaplessMap::new();

        for i in 0..9 {
            map.insert(i, i + 1).unwrap();
        }

        assert_eq!(map.insert(9, 10), Ok(()));
        assert_eq!(map.insert(10, 11), Err(HeaplessVecErr::VectorFull));
    }
}
