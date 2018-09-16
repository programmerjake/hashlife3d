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
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::cell::UnsafeCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::BuildHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ptr::{self, NonNull};
use std::sync::{Arc, Mutex};
use std::u32;

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
        cube_size: u32,
        stride: math::Vec3<usize>,
        result: &mut [Block],
    ) {
        let root = unsafe { root.as_ref() };
        let size = get_size_from_level!(root.key.level());
        assert!(position.x < size && position.y < size && position.z < size);
        assert!(
            position.x + cube_size <= size
                && position.y + cube_size <= size
                && position.z + cube_size <= size
        );
        match &root.key {
            NodeKey::Leaf(key) => if cube_size == 1 {
                result[0] = key[position.x as usize][position.y as usize][position.z as usize]
            } else {
                assert_eq!(position, math::Vec3::splat(0));
                for z in 0..2 {
                    for y in 0..2 {
                        for x in 0..2 {
                            result[math::Vec3::new(x, y, z).dot(stride)] = key[x][y][z];
                        }
                    }
                }
            },
            NodeKey::Nonleaf(key) => if cube_size == size {
                assert_eq!(position, math::Vec3::splat(0));
                for zi in 0..2 {
                    for yi in 0..2 {
                        for xi in 0..2 {
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
                assert!(cube_size <= size / 2);
                let index = position.map(|v| (v / (size / 2)) as usize);
                Self::get_cube_pow2(
                    key.children[index.x][index.y][index.z],
                    position % math::Vec3::splat(size / 2),
                    cube_size,
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
    fn set_cube_pow2_without_expanding<
        Step: StepFn<Block>,
        H: BuildHasher,
        F: FnMut(math::Vec3<u32>, Block) -> Block,
    >(
        root: NonNull<Node<Block>>,
        position: math::Vec3<u32>,
        cube_size: u32,
        cube_offset: math::Vec3<u32>,
        world: &mut World<Block, Step, H>,
        f: &mut F,
    ) -> NonNull<Node<Block>> {
        let key = unsafe { root.as_ref().key };
        let size = get_size_from_level!(key.level());
        assert!(position.x < size && position.y < size && position.z < size);
        assert!(
            position.x + cube_size <= size
                && position.y + cube_size <= size
                && position.z + cube_size <= size
        );
        match key {
            NodeKey::Leaf(mut key) => if cube_size == 1 {
                key[position.x as usize][position.y as usize][position.z as usize] = f(
                    cube_offset,
                    key[position.x as usize][position.y as usize][position.z as usize],
                );
                world.get(NodeKey::Leaf(key)).into()
            } else {
                assert_eq!(position, math::Vec3::splat(0));
                for z in 0..2 {
                    for y in 0..2 {
                        for x in 0..2 {
                            key[x as usize][y as usize][z as usize] = f(
                                cube_offset + math::Vec3::new(x, y, z),
                                key[x as usize][y as usize][z as usize],
                            );
                        }
                    }
                }
                world.get(NodeKey::Leaf(key)).into()
            },
            NodeKey::Nonleaf(mut key) => if cube_size == size {
                assert_eq!(position, math::Vec3::splat(0));
                for zi in 0..2 {
                    for yi in 0..2 {
                        for xi in 0..2 {
                            let offset = math::Vec3::new(xi, yi, zi).map(|v| (v * (size / 2)));
                            let child = &mut key.children[xi as usize][yi as usize][zi as usize];
                            *child = Self::set_cube_pow2_without_expanding(
                                *child,
                                math::Vec3::splat(0),
                                size / 2,
                                cube_offset + offset,
                                world,
                                f,
                            );
                        }
                    }
                }
                world.get(NodeKey::Nonleaf(key)).into()
            } else {
                assert!(cube_size <= size / 2);
                let index = position.map(|v| (v / (size / 2)) as usize);
                {
                    let child = &mut key.children[index.x][index.y][index.z];
                    *child = Self::set_cube_pow2_without_expanding(
                        *child,
                        position % math::Vec3::splat(size / 2),
                        cube_size,
                        cube_offset,
                        world,
                        f,
                    );
                }
                world.get(NodeKey::Nonleaf(key)).into()
            },
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
        while unsafe { root.as_ref() }.key.level() < MAX_LEVEL {
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
    #[allow(dead_code)]
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
        cube_size: u32,
        stride: math::Vec3<usize>,
        result: &mut [Block],
    ) {
        assert!(cube_size.is_power_of_two());
        assert_eq!(
            position & math::Vec3::splat(cube_size - 1),
            math::Vec3::splat(0)
        );
        let size = self.size();
        assert!(cube_size <= size);
        if position.map(|v| v < size).reduce(|a, b| a && b) {
            Node::get_cube_pow2(self.root, position, cube_size, stride, result);
        } else {
            for rz in 0..cube_size {
                for ry in 0..cube_size {
                    for rx in 0..cube_size {
                        result[math::Vec3::new(rx, ry, rz).map(|v| v as usize).dot(stride)] =
                            Default::default();
                    }
                }
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

const MAX_LEVEL: u32 = 20;
const MAX_LEVEL_SIZE: u32 = get_size_from_level!(MAX_LEVEL);

impl<Block: BlockType, H: BuildHasher> State<Block, H> {
    const OFFSET: u32 = MAX_LEVEL_SIZE / 2;
    fn new(
        shared_world_state: Arc<SharedWorldState<Block, H>>,
        root: NonNull<Node<Block>>,
        empty_state: Substate<Block, H>,
    ) -> Self {
        assert_eq!(unsafe { root.as_ref() }.key.level(), MAX_LEVEL);
        Self {
            state: Substate::create_independent_reference(shared_world_state, root),
            empty_state: empty_state,
        }
    }
    fn new_from_world<Step: StepFn<Block>>(
        world: &mut World<Block, Step, H>,
        mut root: NonNull<Node<Block>>,
    ) -> Self {
        while unsafe { root.as_ref() }.key.level() < MAX_LEVEL {
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
    #[allow(dead_code)]
    pub fn from<Step: StepFn<Block>>(
        state: &SerializedState<Block>,
        world: &mut World<Block, Step, H>,
    ) -> Self {
        let mut nodes: Vec<NonNull<Node<Block>>> = Vec::with_capacity(state.0.len());
        for i in 0..state.0.len() {
            let key = match &state.0[i] {
                &SerializedNode::Leaf(key) => NodeKey::Leaf(key),
                SerializedNode::Nonleaf(key) => unsafe {
                    let mut retval_key = NodeKeyNonleaf {
                        children: [[[NonNull::dangling(); 2]; 2]; 2],
                        children_level: nodes[key[0][0][0].0 as usize].as_ref().key.level() as u8,
                    };
                    for (child, key) in retval_key.children.iter_mut().zip(key.iter()) {
                        for (child, key) in child.iter_mut().zip(key.iter()) {
                            for (child, key) in child.iter_mut().zip(key.iter()) {
                                *child = nodes[key.0 as usize];
                            }
                        }
                    }
                    NodeKey::Nonleaf(retval_key)
                },
            };
            nodes.push(world.get(key).into());
        }
        Self::new_from_world(world, *nodes.last().unwrap())
    }
    pub fn get_substate(&self, position: math::Vec3<i32>, size: u32) -> Substate<Block, H> {
        assert!(size >= 2);
        assert!(size.is_power_of_two());
        assert!(size <= MAX_LEVEL_SIZE);
        let position = position.map(|v| (v as u32).wrapping_add(Self::OFFSET));
        assert_eq!(position.map(|v| v % size), math::Vec3::splat(0));
        if position.map(|v| v >= MAX_LEVEL_SIZE).reduce(|a, b| a || b) {
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
        let position = position.map(|v| (v as u32).wrapping_add(Self::OFFSET));
        assert_eq!(
            position.map(|v| v < MAX_LEVEL_SIZE),
            math::Vec3::splat(true)
        );
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
    fn set_cube_pow2_helper<Step: StepFn<Block>, F: FnMut(math::Vec3<u32>, Block) -> Block>(
        &self,
        world: &mut World<Block, Step, H>,
        position: math::Vec3<i32>,
        cube_size: u32,
        mut f: F,
    ) -> Self {
        assert!(cube_size.is_power_of_two());
        assert!(cube_size <= MAX_LEVEL_SIZE);
        let position = position.map(|v| (v as u32).wrapping_add(Self::OFFSET));
        assert_eq!(position.map(|v| v % cube_size), math::Vec3::splat(0));
        let root = Node::set_cube_pow2_without_expanding(
            self.state.root,
            position,
            cube_size,
            math::Vec3::splat(0),
            world,
            &mut f,
        );
        State::new_from_world(world, root)
    }
    pub fn set_cube_pow2<Step: StepFn<Block>, F: FnMut(math::Vec3<u32>, Block) -> Block>(
        &mut self,
        world: &mut World<Block, Step, H>,
        position: math::Vec3<i32>,
        cube_size: u32,
        f: F,
    ) {
        assert!(self.state.shared_world_state == world.shared_world_state);
        *self = self.set_cube_pow2_helper(world, position, cube_size, f);
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
        if unsafe { root.as_ref() }.key.level() > MAX_LEVEL {
            root = Node::truncate_root_to(MAX_LEVEL, root, world);
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
        let mut initial_node_count = 0usize;
        let mut final_node_count = 0usize;
        nodes.retain(|node| {
            initial_node_count += 1;
            match node.gc_state {
                GcState::Reachable => {
                    final_node_count += 1;
                    true
                }
                GcState::Unreachable => false,
            }
        });
        println!("GC: {} -> {}", initial_node_count, final_node_count);
    }
}

unsafe impl<Block: BlockType + Send + Sync, Step: StepFn<Block> + Send, H: BuildHasher + Send> Send
    for World<Block, Step, H>
{}

unsafe impl<Block: BlockType + Send + Sync, H: BuildHasher + Send> Send for State<Block, H> {}

unsafe impl<Block: BlockType + Send + Sync, H: BuildHasher + Send> Sync for State<Block, H> {}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SerializedNodeIndex(pub u32);

impl SerializedNodeIndex {
    pub const MAX: SerializedNodeIndex = SerializedNodeIndex(u32::MAX);
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum SerializedNode<Block: BlockType> {
    Leaf([[[Block; 2]; 2]; 2]),
    Nonleaf([[[SerializedNodeIndex; 2]; 2]; 2]),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize)]
#[serde(transparent)]
pub struct SerializedState<Block: BlockType>(Vec<SerializedNode<Block>>);

impl<'de, Block: BlockType + Deserialize<'de>> Deserialize<'de> for SerializedState<Block> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let nodes: Vec<SerializedNode<Block>> = Deserialize::deserialize(deserializer)?;
        if nodes.len() as u64 > SerializedNodeIndex::MAX.0 as u64 {
            return Err(Error::custom("too many nodes"));
        }
        if nodes.is_empty() {
            return Err(Error::custom("no root node"));
        }
        let mut levels = Vec::<u8>::with_capacity(nodes.len());
        for i in 0..nodes.len() {
            let level = match &nodes[i] {
                SerializedNode::Leaf(_) => 0,
                SerializedNode::Nonleaf(key) => {
                    let mut level = None;
                    for key in key {
                        for key in key {
                            for key in key {
                                if key.0 as u64 >= i as u64 {
                                    return Err(Error::custom("node index out of range"));
                                }
                                let computed_level = levels[key.0 as usize] + 1;
                                if computed_level as u32 > MAX_LEVEL {
                                    return Err(Error::custom("node nested too deeply"));
                                }
                                if level == None {
                                    level = Some(computed_level);
                                } else if level != Some(computed_level) {
                                    return Err(Error::custom("node children at different levels"));
                                }
                            }
                        }
                    }
                    level.unwrap()
                }
            };
            levels.push(level);
        }
        assert_eq!(levels.len(), nodes.len());
        if *levels.last().unwrap() != MAX_LEVEL as u8 {
            return Err(Error::custom("root node not nested deeply enough"));
        }
        mem::drop(levels);
        let mut used = Vec::new();
        used.resize(nodes.len(), false);
        *used.last_mut().unwrap() = true;
        for i in (0..nodes.len()).rev() {
            if !used[i] {
                return Err(Error::custom("node is unused"));
            }
            if let SerializedNode::Nonleaf(key) = &nodes[i] {
                for key in key {
                    for key in key {
                        for key in key {
                            used[key.0 as usize] = true;
                        }
                    }
                }
            }
        }
        Ok(SerializedState(nodes))
    }
}

impl<Block: BlockType> SerializedState<Block> {
    fn from_node(root: NonNull<Node<Block>>) -> Self {
        let mut nodes = Vec::new();
        let mut nodes_map = HashMap::new();
        fn serialize_node<Block: BlockType>(
            node: NonNull<Node<Block>>,
            nodes_map: &mut HashMap<NonNull<Node<Block>>, SerializedNodeIndex>,
            nodes: &mut Vec<SerializedNode<Block>>,
        ) -> SerializedNodeIndex {
            if let Some(&value) = nodes_map.get(&node) {
                value
            } else {
                let key = match unsafe { &node.as_ref().key } {
                    &NodeKey::Leaf(key) => SerializedNode::Leaf(key),
                    NodeKey::Nonleaf(key) => {
                        let mut new_key = [[[SerializedNodeIndex(0); 2]; 2]; 2];
                        for (new_key, key) in new_key.iter_mut().zip(key.children.iter()) {
                            for (new_key, key) in new_key.iter_mut().zip(key.iter()) {
                                for (new_key, &key) in new_key.iter_mut().zip(key.iter()) {
                                    *new_key = serialize_node(key, nodes_map, nodes);
                                }
                            }
                        }
                        SerializedNode::Nonleaf(new_key)
                    }
                };
                let retval = nodes.len();
                assert!(retval as u64 <= SerializedNodeIndex::MAX.0 as u64);
                let retval = SerializedNodeIndex(retval as u32);
                use std::collections::hash_map::Entry::Vacant;
                if let Vacant(entry) = nodes_map.entry(node) {
                    entry.insert(retval);
                } else {
                    unreachable!()
                }
                nodes.push(key);
                retval
            }
        }
        serialize_node(root, &mut nodes_map, &mut nodes);
        SerializedState(nodes)
    }
}

impl<'a, Block: BlockType, H: BuildHasher> From<&'a State<Block, H>> for SerializedState<Block> {
    fn from(state: &'a State<Block, H>) -> SerializedState<Block> {
        Self::from_node(state.state.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_tokens, Token};
    type Block = u32;

    const TEST_SIZE: u32 = 1 << 3;

    fn encode_position(position: math::Vec3<u32>) -> Block {
        position.x + TEST_SIZE * (position.y + TEST_SIZE * position.z)
    }

    fn decode_position(value: Block) -> math::Vec3<u32> {
        let yz = value / TEST_SIZE;
        let z = yz / TEST_SIZE;
        math::Vec3::new(value % TEST_SIZE, yz % TEST_SIZE, z)
    }

    fn verify_subslice(
        substate: &Substate<Block, DefaultBuildHasher>,
        offset: math::Vec3<u32>,
        size: u32,
    ) {
        assert_eq!(substate.size(), size);
        for x in 0..size {
            for y in 0..size {
                for z in 0..size {
                    let position = math::Vec3::new(x, y, z);
                    assert_eq!(
                        decode_position(substate.get(position)),
                        position + offset,
                        "checking if block matches expected value: offset = {:?}, size = {}",
                        offset,
                        size
                    );
                }
            }
        }
    }

    fn create_test_state<Step: StepFn<Block>>(
        world: &mut World<Block, Step, DefaultBuildHasher>,
        offset: math::Vec3<i32>,
    ) -> State<Block, DefaultBuildHasher> {
        let mut state = State::create_empty(world);
        let set_position = math::Vec3::new(1, 2, 3);
        state.set_cube_pow2(world, offset, TEST_SIZE, |position, block| {
            assert_eq!(block, Default::default());
            if position == set_position {
                return !0 as Block;
            }
            encode_position(position)
        });
        state.set(
            world,
            set_position.map(|v| v as i32) + offset,
            encode_position(set_position),
        );
        state
    }

    #[test]
    fn test_substate() {
        let mut world = World::new(
            |neighborhood: &[[[Block; 3]; 3]; 3]| neighborhood[1][1][1],
            DefaultBuildHasher::new(),
        );
        let offset = -math::Vec3::splat(TEST_SIZE as i32);
        let state = create_test_state(&mut world, offset);
        let verify_subslice = |position: math::Vec3<u32>, size: u32| {
            verify_subslice(
                &state.get_substate(position.map(|v| v as i32) + offset, size),
                position,
                size,
            )
        };
        verify_subslice(math::Vec3::new(0, 0, 0), TEST_SIZE / 2);
        verify_subslice(math::Vec3::new(0, 0, TEST_SIZE / 2), TEST_SIZE / 2);
        verify_subslice(math::Vec3::new(0, TEST_SIZE / 2, 0), TEST_SIZE / 2);
        verify_subslice(
            math::Vec3::new(0, TEST_SIZE / 2, TEST_SIZE / 2),
            TEST_SIZE / 2,
        );
        verify_subslice(math::Vec3::new(TEST_SIZE / 2, 0, 0), TEST_SIZE / 2);
        verify_subslice(
            math::Vec3::new(TEST_SIZE / 2, 0, TEST_SIZE / 2),
            TEST_SIZE / 2,
        );
        verify_subslice(
            math::Vec3::new(TEST_SIZE / 2, TEST_SIZE / 2, 0),
            TEST_SIZE / 2,
        );
        verify_subslice(
            math::Vec3::new(TEST_SIZE / 2, TEST_SIZE / 2, TEST_SIZE / 2),
            TEST_SIZE / 2,
        );
        verify_subslice(math::Vec3::new(2, 2, 2), 2);
    }

    #[test]
    fn test_get_cube() {
        let mut world = World::new(
            |neighborhood: &[[[Block; 3]; 3]; 3]| neighborhood[1][1][1],
            DefaultBuildHasher::new(),
        );
        let state = create_test_state(&mut world, math::Vec3::splat(0));
        let substate = state.get_substate(math::Vec3::splat(0), TEST_SIZE);
        let verify_cube = |offset: math::Vec3<u32>, size: u32| {
            let mut blocks = Vec::with_capacity(size as usize * size as usize * size as usize);
            for _ in 0..size as usize * size as usize * size as usize {
                blocks.push(0 as Block);
            }
            let stride = math::Vec3::new(1, size as usize, size as usize * size as usize);
            substate.get_cube_pow2(offset, size, stride, &mut blocks);
            for x in 0..size {
                for y in 0..size {
                    for z in 0..size {
                        let position = math::Vec3::new(x, y, z);
                        assert_eq!(
                            decode_position(blocks[stride.dot(position.map(|v| v as usize))]),
                            position + offset,
                            "checking if block matches expected value: size = {}",
                            size
                        );
                    }
                }
            }
        };
        verify_cube(math::Vec3::new(1, 1, 1), 1);
        verify_cube(math::Vec3::new(2, 2, 2), 2);
        verify_cube(math::Vec3::new(0, 0, 0), TEST_SIZE);
        verify_cube(math::Vec3::new(0, 0, 0), TEST_SIZE / 2);
        verify_cube(math::Vec3::new(0, 0, TEST_SIZE / 2), TEST_SIZE / 2);
        verify_cube(math::Vec3::new(0, TEST_SIZE / 2, 0), TEST_SIZE / 2);
        verify_cube(
            math::Vec3::new(0, TEST_SIZE / 2, TEST_SIZE / 2),
            TEST_SIZE / 2,
        );
        verify_cube(math::Vec3::new(TEST_SIZE / 2, 0, 0), TEST_SIZE / 2);
        verify_cube(
            math::Vec3::new(TEST_SIZE / 2, 0, TEST_SIZE / 2),
            TEST_SIZE / 2,
        );
        verify_cube(
            math::Vec3::new(TEST_SIZE / 2, TEST_SIZE / 2, 0),
            TEST_SIZE / 2,
        );
        verify_cube(
            math::Vec3::new(TEST_SIZE / 2, TEST_SIZE / 2, TEST_SIZE / 2),
            TEST_SIZE / 2,
        );
    }

    #[test]
    fn test_serde() {
        let mut world = World::new(
            |neighborhood: &[[[Block; 3]; 3]; 3]| neighborhood[1][1][1],
            DefaultBuildHasher::new(),
        );
        let mut state = State::create_empty(&mut world);
        state.set(&mut world, math::Vec3::new(1, 2, 3), 1 as Block);
        let serialized_state = SerializedState::from(&state);
        assert_eq!(state, State::from(&serialized_state, &mut world));
        let mut tokens = Vec::new();
        tokens.push(Token::Seq { len: Some(41) });
        {
            let mut push_node = |node: SerializedNode<Block>| match node {
                SerializedNode::Leaf(node) => tokens.extend_from_slice(&[
                    Token::NewtypeVariant {
                        name: "SerializedNode",
                        variant: "Leaf",
                    },
                    Token::Tuple { len: 2 },
                    Token::Tuple { len: 2 },
                    Token::Tuple { len: 2 },
                    Token::U32(node[0][0][0]),
                    Token::U32(node[0][0][1]),
                    Token::TupleEnd,
                    Token::Tuple { len: 2 },
                    Token::U32(node[0][1][0]),
                    Token::U32(node[0][1][1]),
                    Token::TupleEnd,
                    Token::TupleEnd,
                    Token::Tuple { len: 2 },
                    Token::Tuple { len: 2 },
                    Token::U32(node[1][0][0]),
                    Token::U32(node[1][0][1]),
                    Token::TupleEnd,
                    Token::Tuple { len: 2 },
                    Token::U32(node[1][1][0]),
                    Token::U32(node[1][1][1]),
                    Token::TupleEnd,
                    Token::TupleEnd,
                    Token::TupleEnd,
                ]),
                SerializedNode::Nonleaf(node) => tokens.extend_from_slice(&[
                    Token::NewtypeVariant {
                        name: "SerializedNode",
                        variant: "Nonleaf",
                    },
                    Token::Tuple { len: 2 },
                    Token::Tuple { len: 2 },
                    Token::Tuple { len: 2 },
                    Token::U32(node[0][0][0].0),
                    Token::U32(node[0][0][1].0),
                    Token::TupleEnd,
                    Token::Tuple { len: 2 },
                    Token::U32(node[0][1][0].0),
                    Token::U32(node[0][1][1].0),
                    Token::TupleEnd,
                    Token::TupleEnd,
                    Token::Tuple { len: 2 },
                    Token::Tuple { len: 2 },
                    Token::U32(node[1][0][0].0),
                    Token::U32(node[1][0][1].0),
                    Token::TupleEnd,
                    Token::Tuple { len: 2 },
                    Token::U32(node[1][1][0].0),
                    Token::U32(node[1][1][1].0),
                    Token::TupleEnd,
                    Token::TupleEnd,
                    Token::TupleEnd,
                ]),
            };
            push_node(SerializedNode::Leaf([[[0; 2]; 2]; 2]));
            let sni = |v| SerializedNodeIndex(v);
            for i in 0..19 {
                push_node(SerializedNode::Nonleaf([[[sni(i); 2]; 2]; 2]));
            }
            push_node(SerializedNode::Leaf([[[0, 0], [0, 0]], [[0, 1], [0, 0]]]));
            push_node(SerializedNode::Nonleaf([
                [[sni(0), sni(0)], [sni(0), sni(20)]],
                [[sni(0), sni(0)], [sni(0), sni(0)]],
            ]));
            for i in 1..19 {
                push_node(SerializedNode::Nonleaf([
                    [[sni(20 + i), sni(i)], [sni(i), sni(i)]],
                    [[sni(i), sni(i)], [sni(i), sni(i)]],
                ]));
            }
            push_node(SerializedNode::Nonleaf([
                [[sni(19), sni(19)], [sni(19), sni(19)]],
                [[sni(19), sni(19)], [sni(19), sni(39)]],
            ]));
        }
        tokens.push(Token::SeqEnd);
        assert_tokens(&serialized_state, &tokens);
    }
}
