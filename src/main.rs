mod hashtable;
use hashtable::*;
use std::cell::*;
use std::collections::hash_map::*;
use std::collections::*;
use std::hash::*;
use std::mem::*;
use std::ptr::*;
use std::rc::*;

type Block = u32;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum NodeKey {
    Leaf([[Block; 2]; 2]),
    Nonleaf {
        children: [[NonNull<Node>; 2]; 2],
        children_level: u8,
    },
}

impl NodeKey {
    fn level(&self) -> u8 {
        match self {
            NodeKey::Leaf(_) => 0,
            NodeKey::Nonleaf { children_level, .. } => children_level + 1,
        }
    }
    fn is_valid(&self)->bool {
        match self {
            NodeKey::Leaf(_) => true,
            NodeKey::Nonleaf {
                ref children,
                children_level,
            } => {
                for child in children {
                    for child in child {
                        let child = unsafe{&*child.as_ptr()};
                        if child.key.level()!= children_level{return false;}
                    }
                }
                true
            }
        }        
    }
}

#[derive(Copy, Clone, Debug)]
struct Node {
    key: NodeKey,
    next: Option<NonNull<Node>>,
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, rhs: &Self) -> bool {
        self.key == rhs.key
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.key.hash(h);
    }
}

#[derive(Debug)]
pub struct State<'a> {
    world: &'a World,
    root: NonNull<Node>,
}

impl<'a> State<'a> {
    fn create_empty(world: &'a mut World) -> Self {
        let node: NonNull<_> = world
            .get(NodeKey::Leaf([[Default::default(); 2]; 2]))
            .into();
        let mut snapshots = world.snapshots.borrow_mut();
        let value = snapshots.entry(node).or_insert(0);
        *value = *value + 1;
        Self {
            world: world,
            root: node,
        }
    }
}

impl<'a> Drop for State<'a> {
    fn drop(&mut self) {
        let snapshots = &mut *self.world.snapshots.borrow_mut();
        match snapshots.entry(self.root) {
            Entry::Occupied(mut entry) => {
                if *entry.get() <= 1 {
                    entry.remove();
                } else {
                    *entry.get_mut() = *entry.get() - 1;
                }
            }
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct World {
    nodes: HashTable<Node>,
    snapshots: RefCell<HashMap<NonNull<Node>, usize>>,
}

impl World {
    fn get(&mut self, key: NodeKey) -> &mut Node {
        self.nodes
            .insert(Node {
                key: key,
                next: None,
            })
            .1
    }
    pub fn new() -> World {
        World {
            nodes: HashTable::new(),
            snapshots: Default::default(),
        }
    }
}

fn main() {
    let mut world = World::new();
    let state = State::create_empty(&mut world);
    println!("{:?}", state);
}
