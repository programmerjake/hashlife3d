use std::cmp::Eq;
use std::collections::hash_map;
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::mem;
use std::ptr::{null_mut, NonNull};

#[derive(Debug)]
struct Node<T: Eq + Hash> {
    value: T,
    next: *mut Node<T>,
}

#[derive(Debug)]
struct DrainNodes<T: Eq + Hash> {
    head: *mut Node<T>,
    size: usize,
}

impl<T: Eq + Hash> Iterator for DrainNodes<T> {
    type Item = NonNull<Node<T>>;
    fn next(&mut self) -> Option<NonNull<Node<T>>> {
        if self.head.is_null() {
            return None;
        }
        self.size = self.size - 1;
        unsafe {
            let retval = &mut *self.head;
            self.head = retval.next;
            retval.next = null_mut();
            let retval = retval.into();
            Some(retval)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<T: Eq + Hash> Drop for DrainNodes<T> {
    fn drop(&mut self) {
        for i in self {
            unsafe {
                free_node(i);
            }
        }
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
            Some(node) => unsafe { Some(free_node(node)) },
            None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain_nodes.size_hint()
    }
}

type DefaultBuildHasher = hash_map::RandomState;

pub struct HashTable<T: Eq + Hash, H: BuildHasher = DefaultBuildHasher> {
    table: Vec<*mut Node<T>>,
    size: usize,
    build_hasher: H,
    load_factor: f32,
}

impl<T: Eq + Hash + fmt::Debug, H: BuildHasher> fmt::Debug for HashTable<T, H> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[derive(Debug)]
pub struct Iter<'a, T: Eq + Hash + 'a> {
    table: &'a [*mut Node<T>],
    size: usize,
    node: *mut Node<T>,
}

impl<'a, T: Eq + Hash + 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        loop {
            if !self.node.is_null() {
                unsafe {
                    let node = &*self.node;
                    self.node = node.next;
                    self.size = self.size - 1;
                    return Some(&node.value);
                }
            }
            if self.table.is_empty() {
                return None;
            }
            self.node = self.table[0];
            self.table = &self.table[1..];
        }
    }
}

fn new_node<T: Eq + Hash>(node: Node<T>) -> NonNull<Node<T>> {
    NonNull::new(Box::into_raw(Box::new(node))).unwrap()
}

unsafe fn free_node<T: Eq + Hash>(node: NonNull<Node<T>>) -> T {
    (*Box::from_raw(node.as_ptr())).value
}

const INITIAL_SIZE: usize = 1 << 10;

fn is_power_of_2(v: usize) -> bool {
    v != 0 && (v & (v - 1)) == 0
}

impl<T: Eq + Hash, H: BuildHasher> HashTable<T, H> {
    pub fn with_hasher_and_load_factor(build_hasher: H, load_factor: f32) -> Self {
        assert!(load_factor > 0.0);
        let mut table = Vec::new();
        table.resize(INITIAL_SIZE, null_mut());
        Self {
            table: table,
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
    fn find_node(&self, key: &T, index: usize) -> Option<&*mut Node<T>> {
        let retval = &self.table[index];
        while !retval.is_null() {
            if key == unsafe { &(**retval).value } {
                return Some(retval);
            }
        }
        None
    }
    fn insert_node(&mut self, mut node: NonNull<Node<T>>, index: usize) -> NonNull<Node<T>> {
        let node = unsafe { node.as_mut() };
        node.next = self.table[index];
        self.table[index] = node;
        self.size = self.size + 1;
        node.into()
    }
    fn expand_if_needed(&mut self) {
        let table_size = self.table.len();
        if self.load_factor * (table_size as f32) < self.size as f32 {
            self.rehash(table_size * 2);
        }
    }
    fn insert_new_node(&mut self, value: T, index: usize) -> NonNull<Node<T>> {
        self.insert_node(
            new_node(Node {
                value: value,
                next: null_mut(),
            }),
            index,
        )
    }
    fn drain_nodes(&mut self) -> DrainNodes<T> {
        let mut head: *mut Node<T> = null_mut();
        let size = self.size;
        self.size = 0;
        {
            let mut tail = &mut head;
            for i in 0..self.table.len() {
                *tail = self.table[i];
                self.table[i] = null_mut();
                while !tail.is_null() {
                    tail = unsafe { &mut (**tail).next };
                }
            }
        }
        DrainNodes {
            head: head,
            size: size,
        }
    }
    pub fn drain(&mut self) -> Drain<T> {
        Drain {
            drain_nodes: self.drain_nodes(),
        }
    }
    pub fn iter(&self) -> Iter<T> {
        Iter {
            table: &self.table,
            size: self.size,
            node: null_mut(),
        }
    }
    fn rehash(&mut self, new_table_size: usize) {
        let nodes = self.drain_nodes();
        self.table.resize(new_table_size, null_mut());
        for node in nodes {
            let index = self.get_index(unsafe { &node.as_ref().value });
            self.insert_node(node, index);
        }
    }
    pub fn get(&self, key: &T) -> Option<&T> {
        match self.find_node(key, self.get_index(key)) {
            Some(retval) => Some(unsafe { &(**retval).value }),
            None => None,
        }
    }
    pub fn get_mut(&mut self, key: &T) -> Option<&mut T> {
        match self.find_node(key, self.get_index(key)) {
            Some(retval) => Some(unsafe { &mut (**retval).value }),
            None => None,
        }
    }
    pub fn insert(&mut self, value: T) -> (bool, &mut T) {
        let index = self.get_index(&value);
        if let Some(retval) = self.find_node(&value, index) {
            return (false, unsafe { &mut (**retval).value });
        }
        let retval = self.insert_new_node(value, index);
        self.expand_if_needed();
        (true, unsafe { &mut (*retval.as_ptr()).value })
    }
    pub fn replace(&mut self, value: T) -> Option<T> {
        let index = self.get_index(&value);
        if let Some(node) = self.find_node(&value, index) {
            return Some(mem::replace(unsafe { &mut (**node).value }, value));
        }
        self.insert_new_node(value, index);
        None
    }
    pub fn clear(&mut self) {
        self.drain();
    }
}

impl<T: Eq + Hash, H: BuildHasher + Default> HashTable<T, H> {
    pub fn new() -> Self {
        HashTable::with_hasher(Default::default())
    }
}

impl<T: Eq + Hash, H: BuildHasher> Drop for HashTable<T, H> {
    fn drop(&mut self) {
        self.clear()
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
            let mut bucket = Bucket::new();
            while !v.is_null() {
                unsafe {
                    bucket.head = new_node(Node {
                        value: (**v).value.clone(),
                        next: bucket.head,
                    }).as_ptr();
                    v = &(**v).next;
                }
            }
            retval.table.push(bucket.release());

            struct Bucket<T: Eq + Hash> {
                head: *mut Node<T>,
            }

            impl<T: Eq + Hash> Bucket<T> {
                fn new() -> Self {
                    Self { head: null_mut() }
                }
                fn release(&mut self) -> *mut Node<T> {
                    let retval = self.head;
                    self.head = null_mut();
                    retval
                }
            }

            impl<T: Eq + Hash> Drop for Bucket<T> {
                fn drop(&mut self) {
                    while let Some(node) = NonNull::new(self.head) {
                        unsafe {
                            let next = node.as_ref().next;
                            free_node(node);
                            self.head = next;
                        }
                    }
                }
            }
        }
        retval
    }
}
