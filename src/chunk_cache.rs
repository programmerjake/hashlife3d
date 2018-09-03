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

use block::Block;
use geometry::Mesh;
use hashtable::DefaultBuildHasher;
use math::{self, Dot, Mappable};
use registry::Registry;
use renderer::*;
use std::cmp;
use std::collections::{
    hash_map::{Entry, HashMap, RandomState},
    BinaryHeap,
};
use std::hash::*;
use std::sync::*;
use std::thread;
use world3d::{State, Substate};

struct Chunk<DR: DeviceReference> {
    position: math::Vec3<i32>,
    index_buffer: DR::DeviceIndexBuffer,
    vertex_buffer: DR::DeviceVertexBuffer,
}

struct LoaderCommandBufferQueueEntry<DR: DeviceReference> {
    command_buffer: DR::LoaderCommandBuffer,
    chunk: Chunk<DR>,
}

impl<DR: DeviceReference> LoaderCommandBufferQueueEntry<DR> {
    fn on_receive(self, chunk_cache: &mut ChunkCache<DR>) -> DR::LoaderCommandBuffer {
        chunk_cache.chunks.insert(self.chunk.position, self.chunk);
        self.command_buffer
    }
}

#[derive(Debug, Copy, Clone)]
struct ViewState {
    view_point: math::Vec3<f32>,
    view_distance: f32,
}

enum GenerateThreadMessage {
    SetState(State<Block, DefaultBuildHasher>),
    SetView(ViewState),
}

struct GenerateThreadArgs<DR: DeviceReference> {
    loader_command_buffers_sender: mpsc::Sender<LoaderCommandBufferQueueEntry<DR>>,
    renderer_errors_sender: mpsc::Sender<DR::Error>,
    message_receiver: mpsc::Receiver<GenerateThreadMessage>,
    world_state: State<Block, DefaultBuildHasher>,
    device: DR,
    registry: Registry,
}

const CHUNK_SIZE_SHIFT: u32 = 3;
const CHUNK_SIZE: u32 = 1 << CHUNK_SIZE_SHIFT;
const CHUNK_MOD_MASK: i32 = CHUNK_SIZE as i32 - 1;
const CHUNK_FLOOR_MASK: i32 = !CHUNK_MOD_MASK;
const NEIGHBORHOOD_SIZE: usize = 3;

struct Blocks(Box<[Block]>);

impl Blocks {
    const SIZE: usize = CHUNK_SIZE as usize * NEIGHBORHOOD_SIZE;
    fn new() -> Self {
        let blocks_len = Self::SIZE * Self::SIZE * Self::SIZE;
        let mut blocks = Vec::with_capacity(blocks_len);
        for _ in 0..blocks_len {
            blocks.push(Default::default());
        }
        Blocks(blocks.into_boxed_slice())
    }
    fn stride() -> math::Vec3<usize> {
        math::Vec3::new(1, Self::SIZE, Self::SIZE * Self::SIZE)
    }
    fn get_index(position: math::Vec3<u32>) -> usize {
        position
            .map(|v| {
                assert!(v < Self::SIZE as u32);
                v as usize
            }).dot(Self::stride())
    }
    fn get(&self, position: math::Vec3<u32>) -> Block {
        self.0[Self::get_index(position)]
    }
}

struct GenerateThreadChunk {
    neighborhood: [[[Substate<Block, DefaultBuildHasher>; 3]; 3]; 3],
}

fn make_neighborhood<T, F: FnMut(usize, usize, usize) -> T>(
    mut f: F,
) -> [[[T; NEIGHBORHOOD_SIZE]; NEIGHBORHOOD_SIZE]; NEIGHBORHOOD_SIZE] {
    fn make_array<T, F: FnMut(usize) -> T>(mut f: F) -> [T; NEIGHBORHOOD_SIZE] {
        [f(0), f(1), f(2)]
    }
    make_array(|xi| make_array(|yi| make_array(|zi| f(xi, yi, zi))))
}

fn render_chunk<DR: DeviceReference>(
    neighborhood: [[[Substate<Block, DefaultBuildHasher>; 3]; 3]; 3],
    device: &DR,
    chunk_position: math::Vec3<i32>,
    blocks: &mut Blocks,
    loader_command_buffers_sender: &mpsc::Sender<LoaderCommandBufferQueueEntry<DR>>,
    registry: &Registry,
) -> Result<GenerateThreadChunk, DR::Error> {
    println!("rendering chunk: {:?}", chunk_position);
    for xi in 0..NEIGHBORHOOD_SIZE {
        for yi in 0..NEIGHBORHOOD_SIZE {
            for zi in 0..NEIGHBORHOOD_SIZE {
                neighborhood[xi][yi][zi].get_cube_pow2(
                    math::Vec3::splat(0),
                    CHUNK_SIZE,
                    Blocks::stride(),
                    &mut blocks.0[Blocks::get_index(
                        math::Vec3::new(xi, yi, zi).map(|v| v as u32)
                            * math::Vec3::splat(CHUNK_SIZE),
                    )..]
                        [..CHUNK_SIZE as usize * CHUNK_SIZE as usize * CHUNK_SIZE as usize],
                );
            }
        }
    }
    let mut mesh = Mesh::new();
    const OFFSET: u32 = CHUNK_SIZE;
    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let neighborhood = make_neighborhood(|x2, y2, z2| {
                    blocks.get(
                        math::Vec3::new(x, y, z)
                            + math::Vec3::splat(OFFSET - 1)
                            + math::Vec3::new(x2, y2, z2).map(|v| v as u32),
                    )
                });
                let block = neighborhood[1][1][1];
                if block.id() != Default::default() {
                    println!("rendering block: {:?}", registry.get_block(block.id()))
                }
                registry.get_block(block.id()).render(
                    neighborhood,
                    &mut mesh,
                    chunk_position + math::Vec3::new(x, y, z).map(|v| v as i32),
                );
            }
        }
    }
    let (staging_vertex_buffer, staging_index_buffer) = mesh.create_staging_buffers(device)?;
    let device_vertex_buffer = device.create_device_vertex_buffer_like(&staging_vertex_buffer)?;
    let device_index_buffer = device.create_device_index_buffer_like(&staging_index_buffer)?;
    let mut loader_command_buffer = device.create_loader_command_buffer_builder()?;
    let device_vertex_buffer = loader_command_buffer
        .initialize_vertex_buffer(staging_vertex_buffer.slice_ref(..), device_vertex_buffer)?;
    let device_index_buffer = loader_command_buffer
        .initialize_index_buffer(staging_index_buffer.slice_ref(..), device_index_buffer)?;
    let loader_command_buffer = loader_command_buffer.finish()?;
    let rendered_chunk = Chunk {
        position: chunk_position,
        vertex_buffer: device_vertex_buffer,
        index_buffer: device_index_buffer,
    };
    loader_command_buffers_sender
        .send(LoaderCommandBufferQueueEntry {
            command_buffer: loader_command_buffer,
            chunk: rendered_chunk,
        }).unwrap();
    Ok(GenerateThreadChunk {
        neighborhood: neighborhood,
    })
}

fn for_all_chunks_in_view<E, F: FnMut(math::Vec3<i32>) -> Result<(), E>>(
    view_state: ViewState,
    mut f: F,
) -> Result<(), E> {
    let center_chunk = view_state
        .view_point
        .map(|v| v.floor() as i32 & CHUNK_FLOOR_MASK);
    let view_distance_in_chunks =
        (view_state.view_distance.ceil() as i32 + CHUNK_SIZE as i32 - 1) >> CHUNK_SIZE_SHIFT;
    for dx_in_chunks in -view_distance_in_chunks..=view_distance_in_chunks {
        for dy_in_chunks in -view_distance_in_chunks..=view_distance_in_chunks {
            for dz_in_chunks in -view_distance_in_chunks..=view_distance_in_chunks {
                let chunk_position = math::Vec3::new(dx_in_chunks, dy_in_chunks, dz_in_chunks)
                    * math::Vec3::splat(CHUNK_SIZE as i32)
                    + center_chunk;
                f(chunk_position)?;
            }
        }
    }
    Ok(())
}

fn generate_thread_fn<DR: DeviceReference>(args: GenerateThreadArgs<DR>) {
    let GenerateThreadArgs {
        loader_command_buffers_sender,
        renderer_errors_sender,
        message_receiver,
        mut world_state,
        device,
        registry,
    } = args;
    let chunks_reduce_cache_size_hasher_builder = RandomState::new();
    let mut view_state = ViewState {
        view_point: math::Vec3::splat(0.0f32),
        view_distance: 0.0f32,
    };
    fn receive_message_cluster(
        message_receiver: &mpsc::Receiver<GenerateThreadMessage>,
        mut block: bool,
    ) -> Result<(Option<State<Block, DefaultBuildHasher>>, Option<ViewState>), mpsc::RecvError>
    {
        let mut returned_world_state = None;
        let mut returned_view_state = None;
        loop {
            let recv_result;
            if block {
                block = false;
                recv_result = message_receiver.recv()?;
            } else {
                recv_result = match message_receiver.try_recv() {
                    Err(mpsc::TryRecvError::Empty) => {
                        return Ok((returned_world_state, returned_view_state))
                    }
                    Err(mpsc::TryRecvError::Disconnected) => return Err(mpsc::RecvError),
                    Ok(result) => result,
                };
            }
            match recv_result {
                GenerateThreadMessage::SetView(view) => returned_view_state = Some(view),
                GenerateThreadMessage::SetState(world_state) => {
                    returned_world_state = Some(world_state)
                }
            }
        }
    }
    const MAX_CACHE_SIZE: usize = 1 << 13;
    let mut chunks: HashMap<math::Vec3<i32>, GenerateThreadChunk> = HashMap::new();
    let mut chunks_size = 0;
    #[derive(Debug, Copy, Clone)]
    struct WorkListItem {
        priority: f32,
        chunk_position: math::Vec3<i32>,
    }
    impl Eq for WorkListItem {}
    impl Ord for WorkListItem {
        fn cmp(&self, rhs: &Self) -> cmp::Ordering {
            self.priority.partial_cmp(&rhs.priority).unwrap()
        }
    }
    impl PartialOrd for WorkListItem {
        fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
            Some(self.cmp(rhs))
        }
    }
    impl PartialEq for WorkListItem {
        fn eq(&self, rhs: &Self) -> bool {
            self.cmp(rhs) == cmp::Ordering::Equal
        }
    }
    let mut work_list: BinaryHeap<WorkListItem> = BinaryHeap::new();
    let mut chunks_reduce_cache_size_kept_result = 0;
    let mut blocks = Blocks::new();
    loop {
        match receive_message_cluster(&message_receiver, work_list.peek().is_none()) {
            Ok((new_world_state, new_view_point)) => {
                let mut regenerate_work_list = false;
                if let Some(new_world_state) = new_world_state {
                    world_state = new_world_state;
                    regenerate_work_list = true;
                }
                if let Some(new_view_point) = new_view_point {
                    view_state = new_view_point;
                    regenerate_work_list = true;
                }
                if regenerate_work_list {
                    let mut work_list_vec = work_list.into_vec();
                    for_all_chunks_in_view(view_state, |chunk_position| -> Result<(), ()> {
                        let chunk_center = chunk_position.map(|v| v as f32)
                            + math::Vec3::splat(CHUNK_SIZE as f32 / 2.0);
                        let displacement = view_state.view_point - chunk_center;
                        let priority = -displacement.dot(displacement);
                        work_list_vec.push(WorkListItem {
                            priority: priority,
                            chunk_position: chunk_position,
                        });
                        Ok(())
                    }).unwrap();
                    work_list = work_list_vec.into();
                }
            }
            Err(mpsc::RecvError) => break,
        }
        if chunks_size > MAX_CACHE_SIZE {
            chunks_reduce_cache_size_kept_result = (chunks_reduce_cache_size_kept_result + 1) & 0x1;
            chunks.retain(|chunk_position, _| {
                let mut hasher = chunks_reduce_cache_size_hasher_builder.build_hasher();
                chunk_position.hash(&mut hasher);
                if (hasher.finish() & 0x1) == chunks_reduce_cache_size_kept_result {
                    true
                } else {
                    chunks_size -= 1;
                    false
                }
            });
        }
        let WorkListItem {
            priority: _,
            chunk_position,
        } = match work_list.pop() {
            None => continue,
            Some(chunk) => chunk,
        };
        let neighborhood = make_neighborhood(|xi, yi, zi| {
            world_state.get_substate(
                math::Vec3::new(xi as i32, yi as i32, zi as i32)
                    * math::Vec3::splat(CHUNK_SIZE as i32)
                    + chunk_position,
                CHUNK_SIZE,
            )
        });
        match chunks.entry(chunk_position) {
            Entry::Occupied(ref entry) if entry.get().neighborhood == neighborhood => continue,
            Entry::Vacant(entry) => {
                entry.insert(match render_chunk(
                    neighborhood,
                    &device,
                    chunk_position,
                    &mut blocks,
                    &loader_command_buffers_sender,
                    &registry,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        let _ = renderer_errors_sender.send(error);
                        return;
                    }
                });
                chunks_size += 1;
            }
            Entry::Occupied(mut entry) => {
                entry.insert(match render_chunk(
                    neighborhood,
                    &device,
                    chunk_position,
                    &mut blocks,
                    &loader_command_buffers_sender,
                    &registry,
                ) {
                    Ok(result) => result,
                    Err(error) => {
                        let _ = renderer_errors_sender.send(error);
                        return;
                    }
                });
            }
        }
    }
}

pub struct ChunkCache<DR: DeviceReference> {
    device: DR,
    generate_thread: Option<thread::JoinHandle<()>>,
    chunks: HashMap<math::Vec3<i32>, Chunk<DR>>,
    world_state: State<Block, DefaultBuildHasher>,
    loader_command_buffers_receiver: mpsc::Receiver<LoaderCommandBufferQueueEntry<DR>>,
    renderer_errors_receiver: mpsc::Receiver<DR::Error>,
    message_sender: Option<mpsc::Sender<GenerateThreadMessage>>,
    returned_loader_command_buffers: Vec<DR::LoaderCommandBuffer>,
    tiles_image_set: Arc<DR::DeviceImageSet>,
}

impl<DR: DeviceReference> Drop for ChunkCache<DR> {
    fn drop(&mut self) {
        self.message_sender = None;
        self.generate_thread.take().unwrap().join().unwrap();
    }
}

impl<DR: DeviceReference> ChunkCache<DR> {
    pub fn new(
        device: DR,
        world_state: State<Block, DefaultBuildHasher>,
        registry: Registry,
        tiles_image_set: Arc<DR::DeviceImageSet>,
    ) -> Self {
        let (loader_command_buffers_sender, loader_command_buffers_receiver) = mpsc::channel();
        let (renderer_errors_sender, renderer_errors_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();
        let generate_thread_args = GenerateThreadArgs {
            loader_command_buffers_sender: loader_command_buffers_sender,
            renderer_errors_sender: renderer_errors_sender,
            message_receiver: message_receiver,
            world_state: world_state.clone(),
            device: device.clone(),
            registry: registry,
        };
        let generate_thread = thread::spawn(move || generate_thread_fn(generate_thread_args));
        Self {
            device: device,
            generate_thread: Some(generate_thread),
            chunks: HashMap::new(),
            world_state: world_state,
            loader_command_buffers_receiver: loader_command_buffers_receiver,
            renderer_errors_receiver: renderer_errors_receiver,
            message_sender: Some(message_sender),
            returned_loader_command_buffers: Vec::new(),
            tiles_image_set: tiles_image_set,
        }
    }
    pub fn set_world_state(&mut self, world_state: State<Block, DefaultBuildHasher>) {
        if world_state != self.world_state {
            self.message_sender
                .as_ref()
                .unwrap()
                .send(GenerateThreadMessage::SetState(world_state.clone()))
                .unwrap();
            self.world_state = world_state;
        }
    }
    pub fn get_loader_command_buffers(&mut self) -> &mut Vec<DR::LoaderCommandBuffer> {
        while let Ok(command_buffer) = self.loader_command_buffers_receiver.try_recv() {
            let command_buffer = command_buffer.on_receive(self);
            self.returned_loader_command_buffers.push(command_buffer);
        }
        &mut self.returned_loader_command_buffers
    }
    pub fn get_render_command_buffers(
        &mut self,
        view_point: math::Vec3<f32>,
        view_distance: f32,
    ) -> Result<Vec<DR::RenderCommandBuffer>, DR::Error> {
        if let Ok(error) = self.renderer_errors_receiver.try_recv() {
            while let Ok(_) = self.renderer_errors_receiver.try_recv() {}
            return Err(error);
        }
        let view_state = ViewState {
            view_point: view_point,
            view_distance: view_distance,
        };
        self.message_sender
            .as_ref()
            .unwrap()
            .send(GenerateThreadMessage::SetView(view_state))
            .unwrap();
        let mut retval = self.device.create_render_command_buffer_builder()?;
        retval.set_image_set(&*self.tiles_image_set);
        for_all_chunks_in_view(view_state, |chunk_position| {
            if let Some(chunk) = self.chunks.get(&chunk_position) {
                retval.draw(
                    chunk.vertex_buffer.slice_ref(..),
                    chunk.index_buffer.slice_ref(..),
                );
            }
            Ok(())
        })?;
        Ok(vec![retval.finish()?])
    }
}
