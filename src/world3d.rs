use hashtable::HashTable;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::mem;
use std::ptr::NonNull;
use std::rc::Rc;

pub type Block = u32;

pub fn step(blocks: &[[[Block; 3]; 3]; 3]) -> Block {
    if false {
        return blocks[1][1][1];
    }
    if true {
        return blocks[0][1][1];
    }
    let sum = blocks.iter().fold(0, |acc: u32, blocks: &_| {
        acc + blocks.iter().fold(0, |acc: u32, blocks: &_| {
            acc + blocks
                .iter()
                .fold(0, |acc: u32, block: &Block| acc + *block)
        })
    });
    let retval = if blocks[1][1][1] != 0 {
        if sum >= 3 && sum <= 4 {
            1
        } else {
            0
        }
    } else {
        if sum == 3 {
            1
        } else {
            0
        }
    };
    retval
}

mod slow_state {
    use super::dump_helper;
    use super::step;
    use super::Block;
    use super::Dump;
    use std::iter;
    pub struct SlowState {
        size: u32,
        elements: Vec<Block>,
    }

    impl SlowState {
        fn get_index(&self, x: u32, y: u32, z: u32) -> usize {
            assert!(x < self.size && y < self.size && z < self.size);
            x as usize + self.size as usize * (y as usize + self.size as usize * z as usize)
        }
        pub fn size(&self) -> u32 {
            self.size
        }
        pub fn get(&self, x: u32, y: u32, z: u32) -> Block {
            self.elements[self.get_index(x, y, z)]
        }
        pub fn set(&mut self, x: u32, y: u32, z: u32, block: Block) {
            let index = self.get_index(x, y, z);
            self.elements[index] = block;
        }
        fn step_helper(&self) -> Self {
            assert!(self.size >= 2);
            let mut retval = Self::new(self.size - 2);
            for z in 0..retval.size {
                for y in 0..retval.size {
                    for x in 0..retval.size {
                        let mut step_input: [[[Block; 3]; 3]; 3] = Default::default();
                        for ix in 0..3 {
                            for iy in 0..3 {
                                for iz in 0..3 {
                                    step_input[ix as usize][iy as usize][iz as usize] =
                                        self.get(ix + x, iy + y, iz + z);
                                }
                            }
                        }
                        retval.set(x, y, z, step(&step_input));
                    }
                }
            }
            retval
        }
        pub fn step(&mut self) {
            *self = self.step_helper();
        }
        pub fn new(size: u32) -> Self {
            SlowState {
                size: size,
                elements: iter::repeat(Default::default())
                    .take(size as usize * size as usize * size as usize)
                    .collect(),
            }
        }
    }
    impl Dump for SlowState {
        fn dump(&self) {
            dump_helper(|x, y, z| self.get(x, y, z), self.size());
        }
    }
}

trait Dump {
    fn dump(&self);
}

fn dump_helper<F: FnMut(u32, u32, u32) -> Block>(mut f: F, size: u32) {
    let get = |top: bool, bottom: bool| match (top, bottom) {
        (false, false) => '\u{1F063}',
        (true, false) => '\u{1F086}',
        (false, true) => '\u{1F068}',
        (true, true) => '\u{1F08B}',
    };
    for z in 0..size {
        println!("z={}", z);
        print!("\u{250C}");
        for _ in 0..size {
            print!("\u{2500}");
        }
        println!("\u{2510}");
        for y in (0..size).step_by(2) {
            print!("\u{2502}");
            if y + 1 >= size {
                for x in 0..size {
                    print!("{}", get(f(x, y, z) != 0, false));
                }
            } else {
                for x in 0..size {
                    print!("{}", get(f(x, y, z) != 0, f(x, y + 1, z) != 0));
                }
            }
            println!("\u{2502}");
        }
        print!("\u{2514}");
        for _ in 0..size {
            print!("\u{2500}");
        }
        println!("\u{2518}");
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct NodeKeyNonleaf {
    children: [[[NonNull<Node>; 2]; 2]; 2],
    children_level: u8,
}

impl Dump for NodeKeyNonleaf {
    fn dump(&self) {
        NodeKey::Nonleaf(*self).dump()
    }
}

type NodeKeyLeaf = [[[Block; 2]; 2]; 2];

impl Dump for NodeKeyLeaf {
    fn dump(&self) {
        NodeKey::Leaf(*self).dump()
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum NodeKey {
    Leaf(NodeKeyLeaf),
    Nonleaf(NodeKeyNonleaf),
}

impl Dump for NodeKey {
    fn dump(&self) {
        let level = self.level();
        let size = Node::get_size_from_level(level);
        println!("level = {}, size = {}", level, size);
        dump_helper(
            |x, y, z| {
                Node::get_block(
                    (&Node {
                        key: *self,
                        ..Default::default()
                    }).into(),
                    x,
                    y,
                    z,
                )
            },
            size,
        );
    }
}

impl NodeKey {
    fn level(&self) -> u32 {
        match self {
            NodeKey::Leaf(_) => 0,
            NodeKey::Nonleaf(NodeKeyNonleaf { children_level, .. }) => *children_level as u32 + 1,
        }
    }
    fn is_valid(&self) -> bool {
        match self {
            NodeKey::Leaf(_) => true,
            NodeKey::Nonleaf(NodeKeyNonleaf {
                children,
                children_level,
            }) => {
                for child in children {
                    for child in child {
                        for child in child {
                            let child = unsafe { &*child.as_ptr() };
                            if child.key.level() != *children_level as u32 {
                                return false;
                            }
                        }
                    }
                }
                true
            }
        }
    }
    fn as_leaf(&self) -> &NodeKeyLeaf {
        match self {
            NodeKey::Leaf(retval) => retval,
            NodeKey::Nonleaf(_) => panic!("as_leaf called on Nonleaf"),
        }
    }
    fn as_nonleaf(&self) -> &NodeKeyNonleaf {
        match self {
            NodeKey::Nonleaf(retval) => retval,
            NodeKey::Leaf(_) => panic!("as_nonleaf called on Leaf"),
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
    next: [Option<NonNull<Node>>; 2],
    gc_state: GcState,
}

impl Dump for Node {
    fn dump(&self) {
        self.key.dump()
    }
}

impl Dump for NonNull<Node> {
    fn dump(&self) {
        unsafe { self.as_ref() }.dump()
    }
}

impl Node {
    fn get_filled_node(block: Block, level: u8, world: &mut World) -> NonNull<Node> {
        if level == 0 {
            world.get(NodeKey::Leaf([[[block; 2]; 2]; 2])).into()
        } else {
            let child = Node::get_filled_node(block, level - 1, world);
            world
                .get(NodeKey::Nonleaf(NodeKeyNonleaf {
                    children: [[[child; 2]; 2]; 2],
                    children_level: level - 1,
                }))
                .into()
        }
    }
    fn get_empty_node(level: u8, world: &mut World) -> NonNull<Node> {
        Node::get_filled_node(Default::default(), level, world)
    }
    fn get_log2_of_max_generation_step_for_level(level: u32) -> Option<u32> {
        if level == 0 {
            None
        } else {
            Some(level - 1)
        }
    }
    fn get_log2_of_max_generation_step(&self) -> Option<u32> {
        Self::get_log2_of_max_generation_step_for_level(self.key.level())
    }
    fn is_double_step(&self, log2_generation_count: u32) -> bool {
        self.get_log2_of_max_generation_step().unwrap() <= log2_generation_count
    }
    fn compute_next(
        mut node: NonNull<Node>,
        log2_generation_count: u32,
        world: &mut World,
    ) -> NonNull<Node> {
        let root = unsafe { node.as_mut() };
        let is_double_step = root.is_double_step(log2_generation_count);
        let next_index = is_double_step as usize;
        if let Some(retval) = &mut root.next[next_index] {
            if false {
                println!("compute_next({})", log2_generation_count);
                root.key.dump();
                println!("-> (cached)");
                unsafe { retval.as_ref() }.dump();
            }
            return *retval;
        }
        let step_count;
        let retval = match root.key.as_nonleaf() {
            NodeKeyNonleaf {
                children,
                children_level: 0,
            } => {
                step_count = 1;
                let mut input: [[[Block; 4]; 4]; 4] = Default::default();
                for outer_x in 0..2 {
                    for outer_y in 0..2 {
                        for outer_z in 0..2 {
                            let inner = unsafe { children[outer_x][outer_y][outer_z].as_ref() };
                            for inner_x in 0..2 {
                                for inner_y in 0..2 {
                                    for inner_z in 0..2 {
                                        input[outer_x * 2 + inner_x][outer_y * 2 + inner_y]
                                            [outer_z * 2 + inner_z] =
                                            inner.key.as_leaf()[inner_x][inner_y][inner_z];
                                    }
                                }
                            }
                        }
                    }
                }
                let mut next_key: NodeKeyLeaf = Default::default();
                for dx in 0..2 {
                    for dy in 0..2 {
                        for dz in 0..2 {
                            let mut step_input: [[[Block; 3]; 3]; 3] = Default::default();
                            for x in 0..3 {
                                for y in 0..3 {
                                    for z in 0..3 {
                                        step_input[x][y][z] = input[x + dx][y + dy][z + dz];
                                    }
                                }
                            }
                            next_key[dx][dy][dz] = step(&step_input);
                        }
                    }
                }
                let retval = world.get(NodeKey::Leaf(next_key)).into();
                root.next[next_index] = Some(retval);
                retval
            }
            NodeKeyNonleaf {
                children,
                children_level,
            } => {
                if is_double_step {
                    step_count = 1 << *children_level;
                    let mut intermediate_state = [[[None; 3]; 3]; 3];
                    for x in 0..3 {
                        for y in 0..3 {
                            for z in 0..3 {
                                let is_x_at_edge = x == 0 || x == 2;
                                let is_y_at_edge = y == 0 || y == 2;
                                let is_z_at_edge = z == 0 || z == 2;
                                let is_at_corner = is_x_at_edge && is_y_at_edge && is_z_at_edge;
                                let initial_state_node;
                                if is_at_corner {
                                    initial_state_node = children[x / 2][y / 2][z / 2];
                                } else {
                                    let mut key = NodeKeyNonleaf {
                                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                                        children_level: *children_level - 1,
                                    };
                                    for kx in 0..2 {
                                        for ky in 0..2 {
                                            for kz in 0..2 {
                                                let x = x + kx;
                                                let y = y + ky;
                                                let z = z + kz;
                                                key.children[kx][ky][kz] = unsafe {
                                                    children[x / 2][y / 2][z / 2].as_ref()
                                                }.key
                                                    .as_nonleaf()
                                                    .children[x % 2][y % 2][z % 2];
                                            }
                                        }
                                    }
                                    initial_state_node = world.get(NodeKey::Nonleaf(key)).into();
                                };
                                intermediate_state[x][y][z] = Some(Node::compute_next(
                                    initial_state_node,
                                    log2_generation_count,
                                    world,
                                ));
                            }
                        }
                    }
                    let mut final_key = NodeKeyNonleaf {
                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                        children_level: *children_level - 1,
                    };
                    for x in 0..2 {
                        for y in 0..2 {
                            for z in 0..2 {
                                let mut key = NodeKeyNonleaf {
                                    children: [[[NonNull::dangling(); 2]; 2]; 2],
                                    children_level: *children_level - 1,
                                };
                                for kx in 0..2 {
                                    for ky in 0..2 {
                                        for kz in 0..2 {
                                            let x = x + kx;
                                            let y = y + ky;
                                            let z = z + kz;
                                            key.children[kx][ky][kz] =
                                                intermediate_state[x][y][z].unwrap();
                                        }
                                    }
                                }
                                let intermediate_state_node =
                                    world.get(NodeKey::Nonleaf(key)).into();
                                final_key.children[x][y][z] = Node::compute_next(
                                    intermediate_state_node,
                                    log2_generation_count,
                                    world,
                                );
                            }
                        }
                    }
                    let retval = world.get(NodeKey::Nonleaf(final_key)).into();
                    root.next[next_index] = Some(retval);
                    retval
                } else {
                    step_count = 1 << log2_generation_count;
                    let mut final_state = [[[None; 3]; 3]; 3];
                    for x in 0..3 {
                        for y in 0..3 {
                            for z in 0..3 {
                                let is_x_at_edge = x == 0 || x == 2;
                                let is_y_at_edge = y == 0 || y == 2;
                                let is_z_at_edge = z == 0 || z == 2;
                                let is_at_corner = is_x_at_edge && is_y_at_edge && is_z_at_edge;
                                let initial_state_node;
                                if is_at_corner {
                                    initial_state_node = children[x / 2][y / 2][z / 2];
                                } else {
                                    let mut key = NodeKeyNonleaf {
                                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                                        children_level: *children_level - 1,
                                    };
                                    for kx in 0..2 {
                                        for ky in 0..2 {
                                            for kz in 0..2 {
                                                let x = x + kx;
                                                let y = y + ky;
                                                let z = z + kz;
                                                key.children[kx][ky][kz] = unsafe {
                                                    children[x / 2][y / 2][z / 2].as_ref()
                                                }.key
                                                    .as_nonleaf()
                                                    .children[x % 2][y % 2][z % 2];
                                            }
                                        }
                                    }
                                    initial_state_node = world.get(NodeKey::Nonleaf(key)).into();
                                };
                                final_state[x][y][z] = Some(Node::compute_next(
                                    initial_state_node,
                                    log2_generation_count,
                                    world,
                                ));
                            }
                        }
                    }
                    if false {
                        println!("final_state:");
                        let size = Node::get_size_from_level(
                            unsafe { final_state[0][0][0].unwrap().as_ref() }
                                .key
                                .level(),
                        );
                        dump_helper(
                            |x, y, z| {
                                Node::get_block(
                                    final_state[(x / size) as usize][(y / size) as usize]
                                        [(z / size) as usize]
                                        .unwrap(),
                                    x % size,
                                    y % size,
                                    z % size,
                                )
                            },
                            size * 3,
                        );
                    }
                    let mut final_key = NodeKeyNonleaf {
                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                        children_level: *children_level - 1,
                    };
                    for x in 0..2 {
                        for y in 0..2 {
                            for z in 0..2 {
                                if *children_level == 1 {
                                    let mut key: NodeKeyLeaf = Default::default();
                                    for kx in 0..2 {
                                        for ky in 0..2 {
                                            for kz in 0..2 {
                                                let x = 1 + x * 2 + kx;
                                                let y = 1 + y * 2 + ky;
                                                let z = 1 + z * 2 + kz;
                                                key[kx][ky][kz] = unsafe {
                                                    final_state[x / 2][y / 2][z / 2]
                                                        .unwrap()
                                                        .as_ref()
                                                }.key
                                                    .as_leaf()[x % 2][y % 2][z % 2];
                                            }
                                        }
                                    }
                                    final_key.children[x][y][z] =
                                        world.get(NodeKey::Leaf(key)).into();
                                } else {
                                    let mut key = NodeKeyNonleaf {
                                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                                        children_level: *children_level - 2,
                                    };
                                    for kx in 0..2 {
                                        for ky in 0..2 {
                                            for kz in 0..2 {
                                                let x = 1 + x * 2 + kx;
                                                let y = 1 + y * 2 + ky;
                                                let z = 1 + z * 2 + kz;
                                                key.children[kx][ky][kz] = unsafe {
                                                    final_state[x / 2][y / 2][z / 2]
                                                        .unwrap()
                                                        .as_ref()
                                                }.key
                                                    .as_nonleaf()
                                                    .children[x % 2][y % 2][z % 2];
                                            }
                                        }
                                    }
                                    final_key.children[x][y][z] =
                                        world.get(NodeKey::Nonleaf(key)).into();
                                }
                            }
                        }
                    }
                    let retval = world.get(NodeKey::Nonleaf(final_key)).into();
                    root.next[next_index] = Some(retval);
                    retval
                }
            }
        };
        if false {
            println!(
                "compute_next(log2_generation_count = {}, step_count = {})",
                log2_generation_count, step_count
            );
            root.dump();
            println!("->");
            let retval = unsafe { retval.as_ref() };
            retval.dump();
            let root_size = Node::get_size_from_level(root.key.level());
            let mut state = slow_state::SlowState::new(root_size);
            for z in 0..root_size {
                for y in 0..root_size {
                    for x in 0..root_size {
                        state.set(x, y, z, Node::get_block(root.into(), x, y, z));
                    }
                }
            }
            for _ in 0..step_count {
                state.step();
            }
            if !is_double_step {
                let old_state = mem::replace(&mut state, slow_state::SlowState::new(root_size / 2));
                let offset = root_size / 4 - step_count;
                for z in 0..state.size() {
                    for y in 0..state.size() {
                        for x in 0..state.size() {
                            state.set(x, y, z, old_state.get(x + offset, y + offset, z + offset));
                        }
                    }
                }
            }
            for z in 0..state.size() {
                for y in 0..state.size() {
                    for x in 0..state.size() {
                        if Node::get_block(retval.into(), x, y, z) != state.get(x, y, z) {
                            println!("doesn't match!");
                            state.dump();
                            panic!();
                        }
                    }
                }
            }
        }
        retval
    }
    fn expand_root(root: NonNull<Node>, world: &mut World) -> NonNull<Node> {
        let root_key = unsafe { root.as_ref() }.key;
        let root_key_level = root_key.level();
        assert!(root_key_level <= u8::max_value() as u32);
        let mut retval_key = NodeKeyNonleaf {
            children: [[[NonNull::dangling(); 2]; 2]; 2],
            children_level: root_key.level() as u8,
        };
        match root_key {
            NodeKey::Leaf(children) => {
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            let mut key: NodeKeyLeaf = Default::default();
                            key[1 - x][1 - y][1 - z] = children[x][y][z];
                            retval_key.children[x][y][z] = world.get(NodeKey::Leaf(key)).into();
                        }
                    }
                }
            }
            NodeKey::Nonleaf(NodeKeyNonleaf {
                children,
                children_level,
            }) => {
                let empty_node = Node::get_empty_node(children_level, world);
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            let mut key = NodeKeyNonleaf {
                                children: [[[empty_node; 2]; 2]; 2],
                                children_level: children_level,
                            };
                            key.children[1 - x][1 - y][1 - z] = children[x][y][z];
                            retval_key.children[x][y][z] = world.get(NodeKey::Nonleaf(key)).into();
                        }
                    }
                }
            }
        }
        world.get(NodeKey::Nonleaf(retval_key)).into()
    }
    fn truncate_root(root: NonNull<Node>, world: &mut World) -> NonNull<Node> {
        match unsafe { &root.as_ref().key } {
            NodeKey::Leaf(_) => panic!("can't truncate leaf node"),
            NodeKey::Nonleaf(NodeKeyNonleaf {
                children,
                children_level: 0,
            }) => {
                let mut retval_key: NodeKeyLeaf = Default::default();
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            retval_key[x][y][z] = unsafe { children[x][y][z].as_ref() }
                                .key
                                .as_leaf()[1 - x][1 - y][1 - z];
                        }
                    }
                }
                world.get(NodeKey::Leaf(retval_key)).into()
            }
            NodeKey::Nonleaf(NodeKeyNonleaf {
                children,
                children_level,
            }) => {
                let mut retval_key = NodeKeyNonleaf {
                    children: [[[NonNull::dangling(); 2]; 2]; 2],
                    children_level: children_level - 1,
                };
                for x in 0..2 {
                    for y in 0..2 {
                        for z in 0..2 {
                            retval_key.children[x][y][z] = unsafe { children[x][y][z].as_ref() }
                                .key
                                .as_nonleaf()
                                .children[1 - x][1 - y][1 - z];
                        }
                    }
                }
                world.get(NodeKey::Nonleaf(retval_key)).into()
            }
        }
    }
    fn truncate_root_to(level: u32, mut root: NonNull<Node>, world: &mut World) -> NonNull<Node> {
        assert!(level <= unsafe { root.as_ref() }.key.level());
        for _ in level..unsafe { root.as_ref() }.key.level() {
            root = Node::truncate_root(root, world);
        }
        assert!(level == unsafe { root.as_ref() }.key.level());
        root
    }
    fn get_size_from_level(level: u32) -> u32 {
        2u32 << level
    }
    fn get_block(root: NonNull<Node>, mut x: u32, mut y: u32, mut z: u32) -> Block {
        let mut root = unsafe { root.as_ref() };
        loop {
            let size = Node::get_size_from_level(root.key.level());
            assert!(x < size && y < size && z < size);
            match &root.key {
                NodeKey::Leaf(key) => break key[x as usize][y as usize][z as usize],
                NodeKey::Nonleaf(key) => {
                    let xi = (x / (size / 2)) as usize;
                    let yi = (y / (size / 2)) as usize;
                    let zi = (z / (size / 2)) as usize;
                    x %= size / 2;
                    y %= size / 2;
                    z %= size / 2;
                    root = unsafe { key.children[xi][yi][zi].as_ref() };
                }
            }
        }
    }
    fn set_block_without_expanding(
        root: NonNull<Node>,
        x: u32,
        y: u32,
        z: u32,
        block: Block,
        world: &mut World,
    ) -> NonNull<Node> {
        let root = unsafe { root.as_ref() };
        let size = Node::get_size_from_level(root.key.level());
        assert!(x < size && y < size && z < size);
        match &root.key {
            NodeKey::Leaf(key) => {
                let mut new_key = *key;
                new_key[x as usize][y as usize][z as usize] = block;
                world.get(NodeKey::Leaf(new_key)).into()
            }
            NodeKey::Nonleaf(key) => {
                let mut new_key = *key;
                let xi = (x / (size / 2)) as usize;
                let yi = (y / (size / 2)) as usize;
                let zi = (z / (size / 2)) as usize;
                let x = x % (size / 2);
                let y = y % (size / 2);
                let z = z % (size / 2);
                new_key.children[xi][yi][zi] = Node::set_block_without_expanding(
                    key.children[xi][yi][zi],
                    x,
                    y,
                    z,
                    block,
                    world,
                );
                world.get(NodeKey::Nonleaf(new_key)).into()
            }
        }
    }
}

impl Default for NodeKey {
    fn default() -> Self {
        NodeKey::Leaf([[[Default::default(); 2]; 2]; 2])
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
            next: [None; 2],
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
    const MAX_LEVEL: u32 = 20;
    fn new(world: Rc<RefCell<World>>, world_ref: &World, root: NonNull<Node>) -> Self {
        assert!(unsafe { root.as_ref() }.key.level() <= State::MAX_LEVEL);
        let mut snapshots = world_ref.snapshots.borrow_mut();
        let value = snapshots.entry(root).or_insert(0);
        *value = *value + 1;
        Self {
            world: world,
            root: root,
        }
    }
    pub fn create_empty(world: &Rc<RefCell<World>>) -> Self {
        let mut world_borrow = world.borrow_mut();
        let node = world_borrow.get(Default::default()).into();
        State::new(world.clone(), &*world_borrow, node)
    }
    pub fn get(&self, x: i32, y: i32, z: i32) -> Block {
        let key = &unsafe { self.root.as_ref() }.key;
        let level = key.level();
        let size = Node::get_size_from_level(level);
        let x = (x as u32).wrapping_add(size / 2);
        let y = (y as u32).wrapping_add(size / 2);
        let z = (z as u32).wrapping_add(size / 2);
        if x >= size || y >= size || z >= size {
            Default::default()
        } else {
            Node::get_block(self.root, x, y, z)
        }
    }
    fn set_helper(&self, x: i32, y: i32, z: i32, block: Block) -> Self {
        let mut world_borrow = self.world.borrow_mut();
        let world = &mut *world_borrow;
        let mut root = self.root;
        loop {
            let key = &unsafe { *root.as_ptr() }.key;
            let size = Node::get_size_from_level(key.level());
            let xu = (x as u32).wrapping_add(size / 2);
            let yu = (y as u32).wrapping_add(size / 2);
            let zu = (z as u32).wrapping_add(size / 2);
            if xu < size && yu < size && zu < size {
                root = Node::set_block_without_expanding(root, xu, yu, zu, block, world);
                break;
            } else {
                root = Node::expand_root(root, world);
            }
        }
        State::new(self.world.clone(), world, root)
    }
    pub fn set(&mut self, x: i32, y: i32, z: i32, block: Block) {
        *self = self.set_helper(x, y, z, block);
    }
    fn step_helper(&self, log2_generation_count: u32) -> Self {
        let mut root = self.root;
        let mut world_borrow = self.world.borrow_mut();
        let world = &mut *world_borrow;
        loop {
            let log2_of_max_generation_step: Option<u32> =
                unsafe { root.as_ref() }.get_log2_of_max_generation_step();
            if log2_of_max_generation_step
                .filter(|v| *v > log2_generation_count)
                .is_some()
            {
                break;
            }
            root = Node::expand_root(root, world);
        }
        root = Node::expand_root(root, world);
        root = Node::compute_next(root, log2_generation_count, world);
        if unsafe { root.as_ref() }.key.level() > State::MAX_LEVEL {
            root = Node::truncate_root_to(State::MAX_LEVEL, root, world);
        }
        State::new(self.world.clone(), world, root)
    }
    pub fn step(&mut self, log2_generation_count: u32) {
        *self = self.step_helper(log2_generation_count);
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        State::new(self.world.clone(), &*self.world.borrow(), self.root)
    }
}

impl Drop for State {
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
        if !key.is_valid() {
            let mut key = key;
            loop {
                println!("{:#?}", key);
                match key {
                    NodeKey::Leaf(_) => break,
                    NodeKey::Nonleaf(NodeKeyNonleaf { children, .. }) => {
                        for v in &children {
                            for v in v {
                                for v in v {
                                    println!("{:?}", unsafe { v.as_ref() });
                                }
                            }
                        }
                        key = unsafe { children[0][0][0].as_ref().key };
                    }
                }
            }
        }
        assert!(key.is_valid());
        let (_, retval) = self.nodes.insert(Node {
            key: key,
            ..Default::default()
        });
        retval
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
            for i in node.next.iter() {
                if let Some(next) = *i {
                    Self::mark_node(next, &mut work_queue);
                }
            }
            match &node.key {
                NodeKey::Leaf(_) => (),
                NodeKey::Nonleaf(NodeKeyNonleaf { children, .. }) => {
                    for child in children {
                        for child in child {
                            for child in child {
                                Self::mark_node(*child, &mut work_queue);
                            }
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
