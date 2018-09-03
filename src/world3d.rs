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
use hashtable::*;
use math::{self, Dot, Mappable, Reducible};
use std::cell::UnsafeCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::BuildHasher;
use std::hash::{Hash, Hasher};
use std::ptr::{self, NonNull};
use std::sync::{Arc, Mutex};

pub trait BlockType: Copy + Default + Eq + PartialEq + Hash + fmt::Debug {}

impl<T: Copy + Default + Eq + PartialEq + Hash + fmt::Debug> BlockType for T {}

pub trait StepFn<Block: BlockType> {
    fn step(&self, neighborhood: &[[[Block; 3]; 3]; 3]) -> Block;
}

impl<Block: BlockType, T: Fn(&[[[Block; 3]; 3]; 3]) -> Block> StepFn<Block> for T {
    fn step(&self, neighborhood: &[[[Block; 3]; 3]; 3]) -> Block {
        self(neighborhood)
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct NodeKeyNonleaf<Block: BlockType> {
    children: [[[NonNull<Node<Block>>; 2]; 2]; 2],
    children_level: u8,
}

type NodeKeyLeaf<Block> = [[[Block; 2]; 2]; 2];

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum NodeKey<Block: BlockType> {
    Leaf(NodeKeyLeaf<Block>),
    Nonleaf(NodeKeyNonleaf<Block>),
}

impl<Block: BlockType> NodeKey<Block> {
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
    fn as_leaf(&self) -> &NodeKeyLeaf<Block> {
        match self {
            NodeKey::Leaf(retval) => retval,
            NodeKey::Nonleaf(_) => panic!("as_leaf called on Nonleaf"),
        }
    }
    fn as_nonleaf(&self) -> &NodeKeyNonleaf<Block> {
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
struct Node<Block: BlockType> {
    key: NodeKey<Block>,
    next: [Option<NonNull<Node<Block>>>; 2],
    gc_state: GcState,
}

macro_rules! get_size_from_level {
    ($level:expr) => {
        2u32 << $level
    };
}

impl<Block: BlockType> Node<Block> {
    fn get_filled_node<Step: StepFn<Block>, H: BuildHasher>(
        block: Block,
        level: u8,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
        if level == 0 {
            world.get(NodeKey::Leaf([[[block; 2]; 2]; 2])).into()
        } else {
            let child = Node::get_filled_node(block, level - 1, world);
            world
                .get(NodeKey::Nonleaf(NodeKeyNonleaf {
                    children: [[[child; 2]; 2]; 2],
                    children_level: level - 1,
                })).into()
        }
    }
    fn get_empty_node<Step: StepFn<Block>, H: BuildHasher>(
        level: u8,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
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
    fn compute_next<Step: StepFn<Block>, H: BuildHasher>(
        mut node: NonNull<Node<Block>>,
        log2_generation_count: u32,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
        let root = unsafe { node.as_mut() };
        let is_double_step = root.is_double_step(log2_generation_count);
        let next_index = is_double_step as usize;
        if let Some(retval) = &mut root.next[next_index] {
            return *retval;
        }
        let retval = match root.key.as_nonleaf() {
            NodeKeyNonleaf {
                children,
                children_level: 0,
            } => {
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
                let mut next_key: NodeKeyLeaf<Block> = Default::default();
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
                            next_key[dx][dy][dz] = world.step.step(&step_input);
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
                    let mut final_key = NodeKeyNonleaf {
                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                        children_level: *children_level - 1,
                    };
                    for x in 0..2 {
                        for y in 0..2 {
                            for z in 0..2 {
                                if *children_level == 1 {
                                    let mut key: NodeKeyLeaf<
                                        Block,
                                    > = Default::default();
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
        retval
    }
    fn expand_root<Step: StepFn<Block>, H: BuildHasher>(
        root: NonNull<Node<Block>>,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
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
                            let mut key: NodeKeyLeaf<Block> = Default::default();
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
    fn truncate_root<Step: StepFn<Block>, H: BuildHasher>(
        root: NonNull<Node<Block>>,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
        match unsafe { &root.as_ref().key } {
            NodeKey::Leaf(_) => panic!("can't truncate leaf node"),
            NodeKey::Nonleaf(NodeKeyNonleaf {
                children,
                children_level: 0,
            }) => {
                let mut retval_key: NodeKeyLeaf<Block> = Default::default();
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
    fn truncate_root_to<Step: StepFn<Block>, H: BuildHasher>(
        level: u32,
        mut root: NonNull<Node<Block>>,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
        assert!(level <= unsafe { root.as_ref() }.key.level());
        for _ in level..unsafe { root.as_ref() }.key.level() {
            root = Node::truncate_root(root, world);
        }
        assert!(level == unsafe { root.as_ref() }.key.level());
        root
    }
    fn get_block(root: NonNull<Node<Block>>, mut position: math::Vec3<u32>) -> Block {
        let mut root = unsafe { root.as_ref() };
        loop {
            let size = get_size_from_level!(root.key.level());
            assert!(position.x < size && position.y < size && position.z < size);
            match &root.key {
                NodeKey::Leaf(key) => {
                    break key[position.x as usize][position.y as usize][position.z as usize]
                }
                NodeKey::Nonleaf(key) => {
                    let index = position.map(|v| (v / (size / 2)) as usize);
                    position %= math::Vec3::splat(size / 2);
                    root = unsafe { key.children[index.x][index.y][index.z].as_ref() };
                }
            }
        }
    }
    fn get_cube_pow2(
        root: NonNull<Node<Block>>,
        position: math::Vec3<u32>,
        result_size: u32,
        stride: math::Vec3<usize>,
        result: &mut [Block],
    ) {
        let root = unsafe { root.as_ref() };
        let size = get_size_from_level!(root.key.level());
        assert!(position.x < size && position.y < size && position.z < size);
        assert!(
            position.x + result_size <= size
                && position.y + result_size <= size
                && position.z + result_size <= size
        );
        match &root.key {
            NodeKey::Leaf(key) => if result_size == 1 {
                result[0] = key[position.x as usize][position.y as usize][position.z as usize]
            } else {
                assert_eq!(position, math::Vec3::splat(0));
                for z in 0..1 {
                    for y in 0..1 {
                        for x in 0..1 {
                            result[math::Vec3::new(x, y, z).dot(stride)] = key[x][y][z];
                        }
                    }
                }
            },
            NodeKey::Nonleaf(key) => if result_size == size {
                assert_eq!(position, math::Vec3::splat(0));
                for zi in 0..1 {
                    for yi in 0..1 {
                        for xi in 0..1 {
                            let offset = stride.dot(
                                math::Vec3::new(xi, yi, zi).map(|v| (v * (size / 2) as usize)),
                            );
                            Self::get_cube_pow2(
                                key.children[xi][yi][zi],
                                math::Vec3::splat(0),
                                size / 2,
                                stride,
                                &mut result[offset..],
                            );
                        }
                    }
                }
            } else {
                assert!(result_size <= size / 2);
                let index = position.map(|v| (v / (size / 2)) as usize);
                Self::get_cube_pow2(
                    key.children[index.x][index.y][index.z],
                    position % math::Vec3::splat(size / 2),
                    result_size,
                    stride,
                    result,
                );
            },
        }
    }
    fn get_child_node(
        mut root: NonNull<Node<Block>>,
        mut position: math::Vec3<u32>,
        child_size: u32,
    ) -> NonNull<Node<Block>> {
        assert!(child_size >= 2);
        loop {
            let size = get_size_from_level!(unsafe { root.as_ref() }.key.level());
            assert!(position.x < size && position.y < size && position.z < size);
            assert!(
                position.x + child_size <= size
                    && position.y + child_size <= size
                    && position.z + child_size <= size
            );
            assert!(child_size <= size);
            if size == child_size {
                break root;
            }
            root = match &unsafe { root.as_ref() }.key {
                NodeKey::Leaf(_) => unreachable!(),
                NodeKey::Nonleaf(key) => {
                    assert!(child_size <= size / 2);
                    let index = position.map(|v| (v / (size / 2)) as usize);
                    position %= math::Vec3::splat(size / 2);
                    key.children[index.x][index.y][index.z]
                }
            }
        }
    }
    fn set_block_without_expanding<Step: StepFn<Block>, H: BuildHasher>(
        root: NonNull<Node<Block>>,
        position: math::Vec3<u32>,
        block: Block,
        world: &mut World<Block, Step, H>,
    ) -> NonNull<Node<Block>> {
        let root = unsafe { root.as_ref() };
        let size = get_size_from_level!(root.key.level());
        assert!(position.x < size && position.y < size && position.z < size);
        match &root.key {
            NodeKey::Leaf(key) => {
                let mut new_key = *key;
                new_key[position.x as usize][position.y as usize][position.z as usize] = block;
                world.get(NodeKey::Leaf(new_key)).into()
            }
            NodeKey::Nonleaf(key) => {
                let mut new_key = *key;
                let index = position.map(|v| (v / (size / 2)) as usize);
                new_key.children[index.x][index.y][index.z] = Node::set_block_without_expanding(
                    key.children[index.x][index.y][index.z],
                    position % math::Vec3::splat(size / 2),
                    block,
                    world,
                );
                world.get(NodeKey::Nonleaf(new_key)).into()
            }
        }
    }
}

impl<Block: BlockType> Default for NodeKey<Block> {
    fn default() -> Self {
        NodeKey::Leaf([[[Default::default(); 2]; 2]; 2])
    }
}

impl Default for GcState {
    fn default() -> Self {
        GcState::Reachable
    }
}

impl<Block: BlockType> Default for Node<Block> {
    fn default() -> Self {
        Self {
            key: Default::default(),
            next: [None; 2],
            gc_state: Default::default(),
        }
    }
}

impl<Block: BlockType> Eq for Node<Block> {}

impl<Block: BlockType> PartialEq for Node<Block> {
    fn eq(&self, rhs: &Self) -> bool {
        self.key == rhs.key
    }
}

impl<Block: BlockType> Hash for Node<Block> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.key.hash(h);
    }
}

#[derive(Debug)]
struct SharedWorldState<Block: BlockType, H: BuildHasher> {
    nodes: UnsafeCell<HashTable<Node<Block>, H>>,
    snapshots: Mutex<HashMap<NonNull<Node<Block>>, Arc<NonNull<Node<Block>>>>>,
}

impl<Block: BlockType, H: BuildHasher> PartialEq for SharedWorldState<Block, H> {
    fn eq(&self, rhs: &Self) -> bool {
        ptr::eq(self, rhs)
    }
}

#[derive(Debug)]
pub struct Substate<Block: BlockType, H: BuildHasher> {
    referenced_root: Arc<NonNull<Node<Block>>>,
    root: NonNull<Node<Block>>,
    shared_world_state: Arc<SharedWorldState<Block, H>>,
}

unsafe impl<Block: BlockType, H: BuildHasher> Send for Substate<Block, H> {}
unsafe impl<Block: BlockType, H: BuildHasher> Sync for Substate<Block, H> {}

impl<Block: BlockType, H: BuildHasher> Clone for Substate<Block, H> {
    fn clone(&self) -> Self {
        Self {
            referenced_root: self.referenced_root.clone(),
            root: self.root,
            shared_world_state: self.shared_world_state.clone(),
        }
    }
}

impl<Block: BlockType, H: BuildHasher> Substate<Block, H> {
    fn create_empty<Step: StepFn<Block>>(world: &mut World<Block, Step, H>) -> Self {
        let mut root: NonNull<Node<Block>> = world.get(Default::default()).into();
        while unsafe { root.as_ref() }.key.level() < State::<Block, H>::MAX_LEVEL {
            root = Node::expand_root(root, world);
        }
        Self::create_independent_reference(world.shared_world_state.clone(), root)
    }
    fn create_independent_reference(
        shared_world_state: Arc<SharedWorldState<Block, H>>,
        root: NonNull<Node<Block>>,
    ) -> Self {
        let referenced_root = shared_world_state
            .snapshots
            .lock()
            .unwrap()
            .entry(root)
            .or_insert_with(|| Arc::new(root))
            .clone();
        Self {
            referenced_root: referenced_root,
            root: root,
            shared_world_state: shared_world_state,
        }
    }
    fn create_dependent_reference(base: Self, root: NonNull<Node<Block>>) -> Self {
        Self {
            referenced_root: base.referenced_root,
            root: root,
            shared_world_state: base.shared_world_state,
        }
    }
    pub fn size(&self) -> u32 {
        let key = &unsafe { self.root.as_ref() }.key;
        let level = key.level();
        get_size_from_level!(level)
    }
    pub fn get(&self, position: math::Vec3<u32>) -> Block {
        let size = self.size();
        if position.x >= size || position.y >= size || position.z >= size {
            Default::default()
        } else {
            Node::get_block(self.root, position)
        }
    }
    pub fn get_cube_pow2(
        &self,
        position: math::Vec3<u32>,
        result_size: u32,
        stride: math::Vec3<usize>,
        result: &mut [Block],
    ) {
        assert!(result_size.is_power_of_two());
        assert_eq!(
            position & math::Vec3::splat(result_size - 1),
            math::Vec3::splat(0)
        );
        let size = self.size();
        for rz in 0..result_size {
            if rz + position.z < size {
                continue;
            }
            for ry in 0..result_size {
                if ry + position.y < size {
                    continue;
                }
                for rx in 0..result_size {
                    if rx + position.x < size {
                        continue;
                    }
                    result[math::Vec3::new(rx, ry, rz).map(|v| v as usize).dot(stride)] =
                        Default::default();
                }
            }
        }
        if position.map(|v| v < size).reduce(|a, b| a && b) {
            if result_size > size {
                Node::get_cube_pow2(self.root, position, size, stride, result)
            } else {
                Node::get_cube_pow2(self.root, position, result_size, stride, result);
            }
        }
    }
    pub fn get_substate(self, position: math::Vec3<u32>, size: u32) -> Self {
        assert!(size >= 2);
        assert!(size.is_power_of_two());
        assert!(size <= self.size());
        assert_eq!(position.map(|v| v % size), math::Vec3::splat(0));
        let root = Node::get_child_node(self.root, position, size);
        Self::create_dependent_reference(self, root)
    }
}

impl<Block: BlockType, H: BuildHasher> Eq for Substate<Block, H> {}

impl<Block: BlockType, H: BuildHasher> PartialEq for Substate<Block, H> {
    fn eq(&self, rhs: &Self) -> bool {
        self.root.as_ptr() == rhs.root.as_ptr()
    }
}

impl<Block: BlockType, H: BuildHasher> Hash for Substate<Block, H> {
    fn hash<H2: Hasher>(&self, h: &mut H2) {
        self.root.as_ptr().hash(h)
    }
}

#[derive(Debug, Clone)]
pub struct State<Block: BlockType, H: BuildHasher> {
    state: Substate<Block, H>,
    empty_state: Substate<Block, H>,
}

impl<Block: BlockType, H: BuildHasher> State<Block, H> {
    const MAX_LEVEL: u32 = 20;
    const MAX_LEVEL_SIZE: u32 = get_size_from_level!(Self::MAX_LEVEL);
    const OFFSET: u32 = Self::MAX_LEVEL_SIZE / 2;
    fn new(
        shared_world_state: Arc<SharedWorldState<Block, H>>,
        root: NonNull<Node<Block>>,
        empty_state: Substate<Block, H>,
    ) -> Self {
        assert_eq!(
            unsafe { root.as_ref() }.key.level(),
            State::<Block, H>::MAX_LEVEL
        );
        Self {
            state: Substate::create_independent_reference(shared_world_state, root),
            empty_state: empty_state,
        }
    }
    fn new_from_world<Step: StepFn<Block>>(
        world: &mut World<Block, Step, H>,
        mut root: NonNull<Node<Block>>,
    ) -> Self {
        while unsafe { root.as_ref() }.key.level() < State::<Block, H>::MAX_LEVEL {
            root = Node::expand_root(root, world);
        }
        let empty_state = Substate::create_empty(world);
        Self::new(world.shared_world_state.clone(), root, empty_state)
    }
    pub fn create_empty<Step: StepFn<Block>>(world: &mut World<Block, Step, H>) -> Self {
        let empty_state = Substate::create_empty(world);
        Self {
            state: empty_state.clone(),
            empty_state: empty_state,
        }
    }
    pub fn get(&self, position: math::Vec3<i32>) -> Block {
        self.state
            .get(position.map(|v| (v as u32).wrapping_add(Self::OFFSET)))
    }
    pub fn get_substate(&self, position: math::Vec3<i32>, size: u32) -> Substate<Block, H> {
        assert!(size >= 2);
        assert!(size.is_power_of_two());
        assert!(size <= Self::MAX_LEVEL_SIZE);
        let position = position.map(|v| (v as u32).wrapping_add(Self::OFFSET));
        assert_eq!(position.map(|v| v % size), math::Vec3::splat(0));
        if position
            .map(|v| v >= Self::MAX_LEVEL_SIZE)
            .reduce(|a, b| a || b)
        {
            self.empty_state
                .clone()
                .get_substate(math::Vec3::splat(0), size)
        } else {
            self.state.clone().get_substate(position, size)
        }
    }
    fn set_helper<Step: StepFn<Block>>(
        &self,
        world: &mut World<Block, Step, H>,
        position: math::Vec3<i32>,
        block: Block,
    ) -> Self {
        assert_eq!(
            position.map(|v| (v as u32).wrapping_add(Self::OFFSET) < Self::MAX_LEVEL_SIZE),
            math::Vec3::splat(true)
        );
        let position = position.map(|v| (v as u32).wrapping_add(Self::OFFSET));
        let root = Node::set_block_without_expanding(self.state.root, position, block, world);
        State::new_from_world(world, root)
    }
    pub fn set<Step: StepFn<Block>>(
        &mut self,
        world: &mut World<Block, Step, H>,
        position: math::Vec3<i32>,
        block: Block,
    ) {
        assert!(self.state.shared_world_state == world.shared_world_state);
        *self = self.set_helper(world, position, block);
    }
    fn step_helper<Step: StepFn<Block>>(
        &self,
        world: &mut World<Block, Step, H>,
        log2_generation_count: u32,
    ) -> Self {
        let mut root = self.state.root;
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
        if unsafe { root.as_ref() }.key.level() > State::<Block, H>::MAX_LEVEL {
            root = Node::truncate_root_to(State::<Block, H>::MAX_LEVEL, root, world);
        }
        State::new_from_world(world, root)
    }
    pub fn step<Step: StepFn<Block>>(
        &mut self,
        world: &mut World<Block, Step, H>,
        log2_generation_count: u32,
    ) {
        assert!(self.state.shared_world_state == world.shared_world_state);
        *self = self.step_helper(world, log2_generation_count);
    }
}

impl<Block: BlockType, H: BuildHasher> Eq for State<Block, H> {}

impl<Block: BlockType, H: BuildHasher> PartialEq for State<Block, H> {
    fn eq(&self, rhs: &Self) -> bool {
        self.state == rhs.state
    }
}

impl<Block: BlockType, H: BuildHasher> Hash for State<Block, H> {
    fn hash<H2: Hasher>(&self, h: &mut H2) {
        self.state.hash(h)
    }
}

#[derive(Debug)]
pub struct World<Block: BlockType, Step: StepFn<Block>, H: BuildHasher> {
    shared_world_state: Arc<SharedWorldState<Block, H>>,
    step: Step,
}

impl<Block: BlockType, Step: StepFn<Block>, H: BuildHasher> World<Block, Step, H> {
    fn get(&mut self, key: NodeKey<Block>) -> &mut Node<Block> {
        debug_assert!(if !key.is_valid() {
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
            false
        } else {
            true
        });
        let nodes = unsafe { &mut *self.shared_world_state.nodes.get() };
        let (_, retval) = nodes.insert(Node {
            key: key,
            ..Default::default()
        });
        retval
    }
    pub fn new(step: Step, build_hasher: H) -> World<Block, Step, H> {
        World {
            shared_world_state: Arc::new(SharedWorldState {
                nodes: UnsafeCell::new(HashTable::with_hasher(build_hasher)),
                snapshots: Default::default(),
            }),
            step: step,
        }
    }
    fn mark_node<'a>(node: NonNull<Node<Block>>, work_queue: &mut VecDeque<&'a mut Node<Block>>) {
        let node = unsafe { &mut *node.as_ptr() };
        if let GcState::Unreachable = node.gc_state {
            node.gc_state = GcState::Reachable;
            work_queue.push_back(node);
        }
    }
    pub fn gc(&mut self) {
        let nodes = unsafe { &mut *self.shared_world_state.nodes.get() };
        for node in nodes.iter_mut() {
            node.gc_state = GcState::Unreachable;
        }
        let mut work_queue = Default::default();
        self.shared_world_state
            .snapshots
            .lock()
            .unwrap()
            .retain(|k, v| {
                if Arc::get_mut(v).is_none() {
                    Self::mark_node(*k, &mut work_queue);
                    true
                } else {
                    false
                }
            });
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
        nodes.retain(|node| match node.gc_state {
            GcState::Reachable => true,
            GcState::Unreachable => false,
        });
    }
}

unsafe impl<Block: BlockType + Send + Sync, Step: StepFn<Block> + Send, H: BuildHasher + Send> Send
    for World<Block, Step, H>
{}

unsafe impl<Block: BlockType + Send + Sync, H: BuildHasher + Send> Send for State<Block, H> {}

unsafe impl<Block: BlockType + Send + Sync, H: BuildHasher + Send> Sync for State<Block, H> {}
