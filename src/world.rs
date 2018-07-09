use hashtable::*;
use std::cell::*;
use std::collections::hash_map::Entry;
use std::collections::*;
use std::hash::*;
use std::ptr::*;
use std::rc::*;
pub type Block = u32;

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
    fn is_valid(&self) -> bool {
        match self {
            NodeKey::Leaf(_) => true,
            NodeKey::Nonleaf {
                children,
                children_level,
            } => {
                for child in children {
                    for child in child {
                        let child = unsafe { &*child.as_ptr() };
                        if child.key.level() != *children_level {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum GcState {
    Unreachable,
    Reachable,
}

#[derive(Copy, Clone, Debug)]
struct Node {
    key: NodeKey,
    next: Option<NonNull<Node>>,
    gc_state: GcState,
}

impl Default for NodeKey {
    fn default() -> Self {
        NodeKey::Leaf([[Default::default(); 2]; 2])
    }
}

impl Default for GcState {
    fn default() -> Self {
        GcState::Reachable
    }
}

impl Default for Node {
    fn default() -> Self {
        Self {
            key: Default::default(),
            next: None,
            gc_state: Default::default(),
        }
    }
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
pub struct State {
    world: Rc<RefCell<World>>,
    root: NonNull<Node>,
}

impl State {
    pub fn create_empty(world: &Rc<RefCell<World>>) -> Self {
        let world = world.clone();
        let node: NonNull<_>;
        {
            let mut world_borrow = world.borrow_mut();
            node = world_borrow.get(Default::default()).into();
            let mut snapshots = world_borrow.snapshots.borrow_mut();
            let value = snapshots.entry(node).or_insert(0);
            *value = *value + 1;
        }
        Self {
            world: world,
            root: node,
        }
    }
}

impl<'a> Drop for State {
    fn drop(&mut self) {
        let mut world = self.world.borrow_mut();
        let snapshots = &mut *world.snapshots.borrow_mut();
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
        assert!(key.is_valid());
        self.nodes
            .insert(Node {
                key: key,
                ..Default::default()
            })
            .1
    }
    pub fn new() -> Rc<RefCell<World>> {
        Rc::new(RefCell::new(World {
            nodes: HashTable::new(),
            snapshots: Default::default(),
        }))
    }
    fn mark_node<'a>(node: NonNull<Node>, work_queue: &mut VecDeque<&'a mut Node>) {
        let node = unsafe { &mut *node.as_ptr() };
        if let GcState::Unreachable = node.gc_state {
            node.gc_state = GcState::Reachable;
            work_queue.push_back(node);
        }
    }
    pub fn gc(&mut self) {
        for node in &mut self.nodes {
            node.gc_state = GcState::Unreachable;
        }
        let mut work_queue = Default::default();
        for node in self.snapshots.borrow().keys() {
            Self::mark_node(*node, &mut work_queue);
        }
        while let Some(node) = work_queue.pop_front() {
            if let Some(next) = node.next {
                Self::mark_node(next, &mut work_queue);
            }
            match &node.key {
                NodeKey::Leaf(_) => (),
                NodeKey::Nonleaf { children, .. } => {
                    for child in children {
                        for child in child {
                            Self::mark_node(*child, &mut work_queue);
                        }
                    }
                }
            }
        }
        self.nodes.retain(|node| match node.gc_state {
            GcState::Reachable => true,
            GcState::Unreachable => false,
        });
    }
}
