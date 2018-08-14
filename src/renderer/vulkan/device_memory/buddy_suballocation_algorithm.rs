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
use super::Suballocation as GenericSuballocation;
use super::SuballocationAlgorithm as GenericSuballocationAlgorithm;
use super::SuballocationFailed;
use std::cmp;
use std::result;

type Result<T> = result::Result<T, SuballocationFailed>;

#[derive(Debug, Eq, PartialEq)]
pub struct Suballocation {
    index: usize,
    size: u64,
    offset: u64,
}

impl GenericSuballocation for Suballocation {
    fn get_offset(&self) -> u64 {
        self.offset
    }
    fn get_size(&self) -> u64 {
        self.size
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Node {
    Free,
    Allocated,
    Split,
}

#[derive(Debug)]
pub struct BuddySuballocationAlgorithm {
    log2_total_size: u32,
    nodes: Vec<Node>,
    free_lists: Box<[Vec<usize>]>,
}

const ROOT_INDEX: usize = 1;

fn first_child_index(index: usize) -> usize {
    index << 1
}

fn second_child_index(index: usize) -> usize {
    first_child_index(index) + 1
}

fn sibling_index(index: usize) -> usize {
    assert!(index > ROOT_INDEX);
    index ^ 1
}

fn parent_index(index: usize) -> usize {
    assert!(index > ROOT_INDEX);
    index >> 1
}

fn ceil_log2(v: u64) -> u32 {
    assert!(v != 0);
    64 - (v - 1).leading_zeros()
}

fn floor_log2(v: u64) -> u32 {
    assert!(v != 0);
    63 - v.leading_zeros()
}

fn get_offset_from_index(index: usize, log2_total_size: u32) -> u64 {
    assert_ne!(index, 0);
    let log2_index = floor_log2(index as u64);
    assert!(log2_index <= log2_total_size);
    let v = index as u64 & !(1 << log2_index);
    v << (log2_total_size - log2_index)
}

fn get_log2_size_from_index(index: usize, log2_total_size: u32) -> u32 {
    assert_ne!(index, 0);
    let log2_index = floor_log2(index as u64);
    assert!(log2_index <= log2_total_size);
    log2_total_size - log2_index
}

fn insert_index_in_free_list(index: usize, free_list: &mut Vec<usize>) {
    let free_list_index = free_list.binary_search_by(|v| index.cmp(v)).unwrap_err();
    free_list.insert(free_list_index, index);
}

fn remove_index_from_free_list(index: usize, free_list: &mut Vec<usize>) {
    let free_list_index = free_list.binary_search_by(|v| index.cmp(v)).unwrap();
    free_list.remove(free_list_index);
}

impl GenericSuballocationAlgorithm for BuddySuballocationAlgorithm {
    type Suballocation = Suballocation;
    fn new(size: u64) -> Self {
        let log2_size = ceil_log2(size);
        let mut nodes = Vec::new();
        nodes.resize(1 << 6, Node::Free);
        let mut free_lists = Vec::new();
        free_lists.reserve((log2_size + 1) as usize);
        for _ in 0..=log2_size {
            free_lists.push(Vec::new())
        }
        free_lists[log2_size as usize].push(ROOT_INDEX);
        Self {
            log2_total_size: log2_size,
            nodes: nodes,
            free_lists: free_lists.into_boxed_slice(),
        }
    }
    fn allocate(&mut self, size: u64, alignment: u64) -> Result<Suballocation> {
        assert!(size != 0);
        assert!(alignment != 0 && alignment.is_power_of_two());
        let minimum_log2_size = ceil_log2(cmp::max(size, alignment));
        if minimum_log2_size <= self.log2_total_size {
            let required_node_count = 2 << (self.log2_total_size - minimum_log2_size);
            if self.nodes.len() < required_node_count {
                self.nodes.resize(required_node_count, Node::Free);
            }
        }
        for mut block_log2_size in minimum_log2_size..=self.log2_total_size {
            if let Some(mut index) = self.free_lists[block_log2_size as usize].pop() {
                for block_log2_size in ((minimum_log2_size + 1)..=block_log2_size).rev() {
                    self.nodes[index] = Node::Split;
                    self.nodes[second_child_index(index)] = Node::Free;
                    insert_index_in_free_list(
                        second_child_index(index),
                        &mut self.free_lists[(block_log2_size - 1) as usize],
                    );
                    index = first_child_index(index);
                }
                self.nodes[index] = Node::Allocated;
                return Ok(Suballocation {
                    index: index,
                    size: 1 << minimum_log2_size,
                    offset: get_offset_from_index(index, self.log2_total_size),
                });
            }
        }
        Err(SuballocationFailed {})
    }
    fn free(&mut self, suballocation: Suballocation) {
        let Suballocation { mut index, .. } = suballocation;
        assert!(index >= ROOT_INDEX);
        assert_eq!(self.nodes[index], Node::Allocated);
        self.nodes[index] = Node::Free;
        let mut log2_size = get_log2_size_from_index(index, self.log2_total_size);
        while index > ROOT_INDEX {
            let parent_index = parent_index(index);
            assert_eq!(self.nodes[parent_index], Node::Split);
            let sibling_index = sibling_index(index);
            if self.nodes[sibling_index] == Node::Free {
                remove_index_from_free_list(
                    sibling_index,
                    &mut self.free_lists[log2_size as usize],
                );
                index = parent_index;
                log2_size = log2_size + 1;
                self.nodes[index] = Node::Free;
            } else {
                break;
            }
        }
        insert_index_in_free_list(index, &mut self.free_lists[log2_size as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::Node::*;
    use super::*;

    const LOG2_SIZE: usize = 10;
    const SIZE: u64 = 1 << LOG2_SIZE;

    #[test]
    fn test_floor_ceil_log2() {
        for i in 1..=0x1000 {
            assert_eq!(floor_log2(i), (i as f64).log2().floor() as u32);
            assert_eq!(ceil_log2(i), (i as f64).log2().ceil() as u32);
        }
    }

    fn matches_free_lists(
        a: &BuddySuballocationAlgorithm,
        match_lists: &[(usize, &[usize])],
    ) -> bool {
        let mut matched_list_indexes = [false; LOG2_SIZE + 1];
        for &(list_index, indexes) in match_lists {
            assert!(!matched_list_indexes[list_index]);
            matched_list_indexes[list_index] = true;
            if a.free_lists[list_index] != indexes {
                return false;
            }
        }
        for list_index in matched_list_indexes
            .iter()
            .enumerate()
            .filter_map(|(list_index, &matched)| if matched { None } else { Some(list_index) })
        {
            if !a.free_lists[list_index].is_empty() {
                return false;
            }
        }
        true
    }

    fn assert_matches_free_lists(
        a: &BuddySuballocationAlgorithm,
        match_lists: &[(usize, &[usize])],
    ) {
        if !matches_free_lists(a, match_lists) {
            panic!(format!(
                "free lists don't match:\n{:#?}\n{:#?}",
                a.free_lists, match_lists
            ));
        }
    }

    #[test]
    fn test_1() {
        let mut a = BuddySuballocationAlgorithm::new(SIZE);
        assert_matches_free_lists(&a, &[(LOG2_SIZE, &[ROOT_INDEX])]);
        let alloc1 = a.allocate(SIZE, SIZE).expect("");
        assert_eq!(
            alloc1,
            Suballocation {
                index: 1,
                size: SIZE,
                offset: 0
            }
        );
        assert!(match a.nodes[1..2] {
            [Allocated] => true,
            _ => false,
        });
        assert_matches_free_lists(&a, &[]);
        a.free(alloc1);
        assert_matches_free_lists(&a, &[(LOG2_SIZE, &[ROOT_INDEX])]);
        assert!(match a.nodes[1..2] {
            [Free] => true,
            _ => false,
        });
    }

    #[test]
    fn test_2() {
        let mut a = BuddySuballocationAlgorithm::new(SIZE);
        assert_matches_free_lists(&a, &[(LOG2_SIZE, &[ROOT_INDEX])]);
        let alloc1 = a.allocate(SIZE / 2, 1).expect("");
        assert_eq!(
            alloc1,
            Suballocation {
                index: 2,
                size: SIZE / 2,
                offset: 0
            }
        );
        assert!(match a.nodes[1..4] {
            [Split, Allocated, Free] => true,
            _ => false,
        });
        assert_matches_free_lists(&a, &[(LOG2_SIZE - 1, &[3])]);
        let alloc2 = a.allocate(SIZE / 2, 1).expect("");
        assert_eq!(
            alloc2,
            Suballocation {
                index: 3,
                size: SIZE / 2,
                offset: SIZE / 2
            }
        );
        assert!(match a.nodes[1..4] {
            [Split, Allocated, Allocated] => true,
            _ => false,
        });
        assert_matches_free_lists(&a, &[]);
        a.free(alloc1);
        assert_matches_free_lists(&a, &[(LOG2_SIZE - 1, &[2])]);
        assert!(match a.nodes[1..4] {
            [Split, Free, Allocated] => true,
            _ => false,
        });
        a.free(alloc2);
        assert_matches_free_lists(&a, &[(LOG2_SIZE, &[1])]);
        assert!(match a.nodes[1..2] {
            [Free] => true,
            _ => false,
        });
    }

    #[test]
    fn test_3() {
        let mut a = BuddySuballocationAlgorithm::new(SIZE);
        assert_matches_free_lists(&a, &[(LOG2_SIZE, &[ROOT_INDEX])]);
        let alloc1 = a.allocate(SIZE / 2, 1).expect("");
        assert_eq!(
            alloc1,
            Suballocation {
                index: 2,
                size: SIZE / 2,
                offset: 0
            }
        );
        assert!(match a.nodes[1..4] {
            [Split, Allocated, Free] => true,
            _ => false,
        });
        assert_matches_free_lists(&a, &[(LOG2_SIZE - 1, &[3])]);
        let alloc2 = a.allocate(SIZE / 4, 1).expect("");
        assert_eq!(
            alloc2,
            Suballocation {
                index: 6,
                size: SIZE / 4,
                offset: SIZE / 2
            }
        );
        assert!(match a.nodes[1..8] {
            [Split, Allocated, Split, _, _, Allocated, Free] => true,
            _ => false,
        });
        assert_matches_free_lists(&a, &[(LOG2_SIZE - 2, &[7])]);
        let alloc3 = a.allocate(SIZE / 4, 1).expect("");
        assert_eq!(
            alloc3,
            Suballocation {
                index: 7,
                size: SIZE / 4,
                offset: SIZE * 3 / 4
            }
        );
        assert!(match a.nodes[1..8] {
            [Split, Allocated, Split, _, _, Allocated, Allocated] => true,
            _ => false,
        });
        assert_matches_free_lists(&a, &[]);
    }
}
