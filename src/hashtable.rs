// This file is part of Hashlife3d.
//
// Hashlife3d is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hashlife3d is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Hashlife3d.  If not, see <https://www.gnu.org/licenses/>
use std::cmp::Eq;
use std::collections::hash_map;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::*;
use std::mem;

struct Node<T: Eq + Hash> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T: Eq + Hash + fmt::Debug> fmt::Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}:", self)?;
        self.value.fmt(f)
    }
}

#[derive(Debug)]
struct DrainNodes<T: Eq + Hash> {
    head: Option<Box<Node<T>>>,
    size: usize,
}

impl<T: Eq + Hash> Iterator for DrainNodes<T> {
    type Item = Box<Node<T>>;
    fn next(&mut self) -> Option<Box<Node<T>>> {
        self.head.take().map(|mut node| {
            self.size = self.size - 1;
            self.head = node.next.take();
            node
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

#[derive(Debug)]
pub struct Drain<T: Eq + Hash> {
    drain_nodes: DrainNodes<T>,
}

impl<T: Eq + Hash> Iterator for Drain<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match self.drain_nodes.next() {
            Some(node) => Some(node.value),
            None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain_nodes.size_hint()
    }
}

pub type DefaultBuildHasher = hash_map::RandomState;

pub struct HashTable<T: Eq + Hash, H: BuildHasher = DefaultBuildHasher> {
    table: Vec<Option<Box<Node<T>>>>,
    size: usize,
    build_hasher: H,
    load_factor: f32,
}

impl<T: Eq + Hash + fmt::Debug, H: BuildHasher> fmt::Debug for HashTable<T, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.node_iter()).finish()
    }
}

#[derive(Debug)]
pub struct Iter<'a, T: Eq + Hash + 'a> {
    node_iter: NodeIter<'a, T>,
}

impl<'a, T: Eq + Hash + 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.node_iter.next().map(|a| &a.value)
    }
}

#[derive(Debug)]
pub struct IterMut<'a, T: Eq + Hash + 'a> {
    table: Option<&'a mut [Option<Box<Node<T>>>]>,
    size: usize,
    node: Option<&'a mut Node<T>>,
}

impl<'a, T: Eq + Hash + 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        loop {
            if let Some(node) = self.node.take() {
                self.node = node.next.as_mut().map(|node| &mut **node);
                self.size = self.size - 1;
                return Some(&mut node.value);
            }
            match self.table.take().unwrap().split_first_mut() {
                None => {
                    return None;
                }
                Some((node, rest)) => {
                    self.node = match node {
                        None => None,
                        Some(node) => Some(&mut *node),
                    };
                    self.table = Some(rest);
                }
            }
        }
    }
}

#[derive(Debug)]
struct NodeIter<'a, T: Eq + Hash + 'a> {
    table: &'a [Option<Box<Node<T>>>],
    size: usize,
    node: Option<&'a Node<T>>,
}

impl<'a, T: Eq + Hash + 'a> Iterator for NodeIter<'a, T> {
    type Item = &'a Node<T>;
    fn next(&mut self) -> Option<&'a Node<T>> {
        loop {
            if let Some(node) = self.node.take() {
                self.node = node.next.as_ref().map(|node| &**node);
                self.size = self.size - 1;
                return Some(node);
            }
            if self.table.is_empty() {
                return None;
            }
            self.node = self.table[0].as_ref().map(|node| &**node);
            self.table = &self.table[1..];
        }
    }
}

const INITIAL_SIZE: usize = 1 << 5;

fn is_power_of_2(v: usize) -> bool {
    v != 0 && (v & (v - 1)) == 0
}

#[derive(Debug)]
pub struct VacantEntry<'a, T: Eq + Hash + 'a> {
    slot: &'a mut Option<Box<Node<T>>>,
    size: &'a mut usize,
    value: T,
}

impl<'a, T: Eq + Hash> VacantEntry<'a, T> {
    pub fn get(&self) -> &T {
        &self.value
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }
    pub fn into(self) -> T {
        self.value
    }
    pub fn insert(self) -> &'a mut T {
        *self.slot = Some(Box::new(Node {
            value: self.value,
            next: self.slot.take(),
        }));
        *self.size = *self.size + 1;
        &mut self.slot.as_mut().unwrap().value
    }
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, T: Eq + Hash + 'a> {
    node_ref: &'a mut Option<Box<Node<T>>>,
    size: &'a mut usize,
}

impl<'a, T: Eq + Hash> OccupiedEntry<'a, T> {
    pub fn get(&self) -> &T {
        match &self.node_ref {
            Some(node) => &node.value,
            None => panic!(),
        }
    }
    pub fn get_mut(&mut self) -> &mut T {
        match &mut self.node_ref {
            Some(node) => &mut node.value,
            None => panic!(),
        }
    }
    pub fn into_mut(self) -> &'a mut T {
        match self.node_ref {
            Some(node) => &mut node.value,
            None => panic!(),
        }
    }
    pub fn remove(self) -> T {
        let node = *self.node_ref.take().unwrap();
        *self.node_ref = node.next;
        *self.size = *self.size - 1;
        node.value
    }
}

pub struct DrainFilter<'a, T: Eq + Hash + 'a, F: FnMut(&mut T) -> bool> {
    table: Option<&'a mut [Option<Box<Node<T>>>]>,
    slot: Option<&'a mut Option<Box<Node<T>>>>,
    size: &'a mut usize,
    f: F,
}

impl<'a, T: Eq + Hash + 'a, F: FnMut(&mut T) -> bool> Iterator for DrainFilter<'a, T, F> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        loop {
            let slot_ref = mem::replace(&mut self.slot, None);
            if let Some(slot_ref) = slot_ref {
                let slot = mem::replace(slot_ref, None);
                if let Some(mut slot) = slot {
                    if (self.f)(&mut slot.value) {
                        *slot_ref = slot.next.take();
                        self.slot = Some(slot_ref);
                        *self.size = *self.size - 1;
                        return Some(slot.value);
                    }
                    *slot_ref = Some(slot);
                    self.slot = match slot_ref {
                        Some(slot) => Some(&mut slot.next),
                        None => None,
                    };
                    continue;
                }
            }
            match self.table.take().unwrap().split_first_mut() {
                None => {
                    return None;
                }
                Some((slot, rest)) => {
                    self.slot = Some(slot);
                    self.table = Some(rest);
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Entry<'a, T: Eq + Hash + 'a> {
    Vacant(VacantEntry<'a, T>),
    Occupied(OccupiedEntry<'a, T>),
}

impl<'a, T: Eq + Hash + 'a> Entry<'a, T> {
    pub fn or_insert(self) -> &'a mut T {
        match self {
            Entry::Vacant(entry) => entry.insert(),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }
    pub fn get(&self) -> &T {
        match self {
            Entry::Vacant(entry) => entry.get(),
            Entry::Occupied(entry) => entry.get(),
        }
    }
    pub fn get_mut(&mut self) -> &mut T {
        match self {
            Entry::Vacant(entry) => entry.get_mut(),
            Entry::Occupied(entry) => entry.get_mut(),
        }
    }
    pub fn and_modify<F: FnOnce(&mut T)>(mut self, f: F) -> Entry<'a, T> {
        if let Entry::Occupied(entry) = &mut self {
            f(entry.get_mut())
        }
        self
    }
}

impl<T: Eq + Hash, H: BuildHasher> HashTable<T, H> {
    pub fn with_hasher_and_load_factor(build_hasher: H, load_factor: f32) -> Self {
        assert!(load_factor > 0.0);
        Self {
            table: repeat(0).map(|_| None).take(INITIAL_SIZE).collect(),
            size: 0,
            build_hasher: build_hasher,
            load_factor: load_factor,
        }
    }
    pub fn with_hasher(build_hasher: H) -> Self {
        Self::with_hasher_and_load_factor(build_hasher, 1.0)
    }
    fn get_index(&self, key: &T) -> usize {
        assert!(is_power_of_2(self.table.len()));
        let mask = (self.table.len() - 1) as u64;
        let mut hasher = self.build_hasher.build_hasher();
        key.hash(&mut hasher);
        (hasher.finish() & mask) as usize
    }
    fn find_node(&self, key: &T, index: usize) -> Option<&Node<T>> {
        let mut retval = &self.table[index];
        while let Some(node) = retval {
            if key == &node.value {
                return Some(&*node);
            }
            retval = &node.next;
        }
        None
    }
    fn find_node_mut<'a>(&'a mut self, key: &T, index: usize) -> Option<&'a mut Node<T>> {
        let mut retval = Some(&mut self.table[index]);
        while let Some(Some(node)) = retval.take() {
            if key == &node.value {
                return Some(&mut *node);
            }
            retval = Some(&mut node.next);
        }
        None
    }
    fn insert_node(&mut self, mut node: Box<Node<T>>, index: usize) -> &mut Node<T> {
        let slot = &mut self.table[index];
        node.next = slot.take();
        *slot = Some(node);
        self.size = self.size + 1;
        &mut *slot.as_mut().unwrap()
    }
    fn expand_if_needed(&mut self) {
        let table_size = self.table.len();
        if self.load_factor * (table_size as f32) < self.size as f32 {
            self.rehash(table_size * 2);
        }
    }
    fn drain_nodes(&mut self) -> DrainNodes<T> {
        let mut head: Option<Box<Node<T>>> = None;
        let size = self.size;
        self.size = 0;
        for slot in &mut self.table {
            let mut slot = slot.take();
            while let Some(mut node) = slot.take() {
                slot = mem::replace(&mut node.next, head);
                head = Some(node);
            }
        }
        DrainNodes {
            head: head,
            size: size,
        }
    }
    fn entry_helper<'a, 'b: 'a>(
        &'b mut self,
        key: &T,
        index: usize,
    ) -> Option<OccupiedEntry<'a, T>> {
        let mut node_ref = Some(&mut self.table[index]);
        while let Some(node) = node_ref.take() {
            if node
                .as_ref()
                .and_then(|v| if key == &v.value { Some(()) } else { None })
                .is_some()
            {
                return Some(OccupiedEntry {
                    node_ref: node,
                    size: &mut self.size,
                });
            }
            node_ref = node.as_mut().map(|v| &mut v.next);
        }
        None
    }
    pub fn entry<'a, 'b: 'a>(&'b mut self, key: T) -> Entry<'a, T> {
        self.expand_if_needed();
        let index = self.get_index(&key);
        let self2: &'b mut Self = unsafe { &mut *(self as *mut Self) };
        match self.entry_helper(&key, index) {
            Some(retval) => return Entry::Occupied(retval),
            None => (),
        }
        Entry::Vacant(VacantEntry {
            slot: &mut self2.table[index],
            size: &mut self2.size,
            value: key,
        })
    }
    pub fn drain(&mut self) -> Drain<T> {
        Drain {
            drain_nodes: self.drain_nodes(),
        }
    }
    pub fn iter(&self) -> Iter<T> {
        Iter {
            node_iter: self.node_iter(),
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            table: Some(&mut self.table),
            size: self.size,
            node: None,
        }
    }
    pub fn len(&self) -> usize {
        self.size
    }
    fn node_iter(&self) -> NodeIter<T> {
        NodeIter {
            table: &self.table,
            size: self.size,
            node: None,
        }
    }
    fn rehash(&mut self, new_table_size: usize) {
        let nodes = self.drain_nodes();
        self.table = repeat(0).map(|_| None).take(new_table_size).collect();
        for node in nodes {
            let index = self.get_index(&node.value);
            self.insert_node(node, index);
        }
    }
    pub fn get(&self, key: &T) -> Option<&T> {
        match self.find_node(key, self.get_index(key)) {
            Some(retval) => Some(&retval.value),
            None => None,
        }
    }
    pub fn get_mut(&mut self, key: &T) -> Option<&mut T> {
        let index = self.get_index(key);
        match self.find_node_mut(key, index) {
            Some(retval) => Some(&mut retval.value),
            None => None,
        }
    }
    pub fn insert<'a>(&'a mut self, value: T) -> (bool, &'a mut T) {
        match self.entry(value) {
            Entry::Occupied(entry) => (false, entry.into_mut()),
            Entry::Vacant(entry) => (true, entry.insert()),
        }
    }
    pub fn clear(&mut self) {
        self.drain();
    }
    pub fn drain_filter<'a, F: FnMut(&mut T) -> bool>(&'a mut self, f: F) -> DrainFilter<'a, T, F> {
        DrainFilter {
            table: Some(&mut self.table),
            slot: None,
            size: &mut self.size,
            f: f,
        }
    }
    pub fn retain<F: FnMut(&mut T) -> bool>(&mut self, mut f: F) {
        for _ in self.drain_filter(|v| !f(v)) {}
    }
}

impl<T: Eq + Hash, H: BuildHasher + Default> HashTable<T, H> {
    pub fn new() -> Self {
        HashTable::with_hasher(Default::default())
    }
}

impl<T: Eq + Hash + Clone, H: BuildHasher + Clone> Clone for HashTable<T, H> {
    fn clone(&self) -> Self {
        let mut retval = Self {
            table: Vec::with_capacity(self.table.len()),
            size: self.size,
            build_hasher: self.build_hasher.clone(),
            load_factor: self.load_factor,
        };
        retval.size = self.size;
        for mut v in &self.table {
            let mut bucket = None;
            while let Some(node) = v {
                bucket = Some(Box::new(Node {
                    value: node.value.clone(),
                    next: bucket,
                }));
                v = &node.next;
            }
            retval.table.push(bucket);
        }
        retval
    }
}

impl<'a, T: Eq + Hash + 'a, H: BuildHasher> IntoIterator for &'a HashTable<T, H> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Eq + Hash + 'a, H: BuildHasher> IntoIterator for &'a mut HashTable<T, H> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: Eq + Hash, H: BuildHasher> IntoIterator for HashTable<T, H> {
    type Item = T;
    type IntoIter = Drain<T>;
    fn into_iter(mut self) -> Self::IntoIter {
        self.drain()
    }
}
