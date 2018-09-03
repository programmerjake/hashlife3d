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

use block::{Block, LightLevel};
use chunk_cache::ChunkCache;
use hashtable::DefaultBuildHasher;
use math;
use registry::Registry;
use renderer::*;
use resources::images::tiles;
use sdl::event::Event;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time;
use world3d;

struct StepFn;

impl world3d::StepFn<Block> for StepFn {
    fn step(&self, neighborhood: &[[[Block; 3]; 3]; 3]) -> Block {
        // FIXME: finish implementing
        neighborhood[1][1][1]
    }
}

pub struct GameState {
    game_thread: Option<thread::JoinHandle<()>>,
    current_state: world3d::State<Block, DefaultBuildHasher>,
    game_state_receiver: Option<mpsc::Receiver<world3d::State<Block, DefaultBuildHasher>>>,
    event_sender: mpsc::Sender<Event>,
}

impl GameState {
    fn game_thread(
        mut world: world3d::World<Block, StepFn, DefaultBuildHasher>,
        mut world_state: world3d::State<Block, DefaultBuildHasher>,
        game_state_sender: mpsc::SyncSender<world3d::State<Block, DefaultBuildHasher>>,
        event_receiver: mpsc::Receiver<Event>,
        registry: Registry,
    ) {
        let mut last_time = time::Instant::now();
        // FIXME: change time_per_loop back to 1/20 second
        let time_per_loop = time::Duration::from_secs(1) / 2;
        let mut block_enabled: bool = false;
        let air_block_id = registry.find_block_by_name("voxels:air").unwrap();
        let stone_block_id = registry.find_block_by_name("voxels:stone").unwrap();
        loop {
            while let Ok(event) = event_receiver.try_recv() {
                println!("event: {:?}", event);
            }
            let current_time = time::Instant::now();
            let next_loop_time = last_time + time_per_loop;
            let elapsed_time;
            if next_loop_time > current_time {
                thread::sleep(next_loop_time - current_time);
                elapsed_time = next_loop_time - last_time;
                last_time = next_loop_time;
            } else {
                elapsed_time = current_time - last_time;
                last_time = current_time;
            }
            println!("step duration: {:?}", elapsed_time);
            world_state.step(&mut world, 1);
            world_state.set(
                &mut world,
                math::Vec3::new(1, 1, 1),
                Block::new(
                    if block_enabled {
                        stone_block_id
                    } else {
                        air_block_id
                    },
                    LightLevel::MAX,
                    LightLevel::MAX,
                    LightLevel::MAX,
                ),
            );
            block_enabled = !block_enabled;
            world.gc();
            match game_state_sender.send(world_state.clone()) {
                Ok(_) => {}
                Err(_) => break,
            }
        }
    }
    pub fn new(registry: Registry) -> Self {
        let mut world = world3d::World::new(StepFn, Default::default());
        let current_state = world3d::State::create_empty(&mut world);
        let game_thread_world_state = current_state.clone();
        let (game_state_sender, game_state_receiver) = mpsc::sync_channel(1);
        let (event_sender, event_receiver) = mpsc::channel();
        let game_thread = thread::spawn(move || {
            Self::game_thread(
                world,
                game_thread_world_state,
                game_state_sender,
                event_receiver,
                registry,
            )
        });
        Self {
            game_thread: Some(game_thread),
            current_state: current_state,
            game_state_receiver: Some(game_state_receiver),
            event_sender: event_sender,
        }
    }
    pub fn get_world_state(&mut self) -> world3d::State<Block, DefaultBuildHasher> {
        loop {
            match self.game_state_receiver.as_ref().unwrap().try_recv() {
                Ok(state) => {
                    self.current_state = state;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(error) => panic!(error),
            }
        }
        self.current_state.clone()
    }
    pub fn send_event(&mut self, event: Event) {
        let _ = self.event_sender.send(event);
    }
}

impl Drop for GameState {
    fn drop(&mut self) {
        self.game_state_receiver = None;
        self.game_thread.take().unwrap().join().unwrap();
    }
}

pub struct RenderState<'a, D: Device> {
    device: D,
    game_state: &'a mut GameState,
    chunk_cache: ChunkCache<D::Reference>,
    start_instant: time::Instant,
}

impl<'a, D: Device> RenderState<'a, D> {
    pub fn new(
        mut device: D,
        game_state: &'a mut GameState,
        registry: Registry,
    ) -> Result<Self, D::Error> {
        let staging_tiles_image_set = tiles::create_tiles_image_set(device.get_device_ref())?;
        let device_tiles_image_set =
            device.create_device_image_set_like(&staging_tiles_image_set)?;
        let mut loader_command_buffer = device.create_loader_command_buffer_builder()?;
        let device_tiles_image_set = loader_command_buffer.initialize_image_set(
            staging_tiles_image_set.slice_ref(..),
            device_tiles_image_set,
        )?;
        let loader_command_buffer = loader_command_buffer.finish()?;
        device
            .submit_loader_command_buffers(&mut vec![loader_command_buffer])?
            .wait()?;
        let tiles_image_set = Arc::new(device_tiles_image_set);
        let chunk_cache = ChunkCache::new(
            device.get_device_ref().clone(),
            game_state.get_world_state(),
            registry,
            tiles_image_set,
        );
        Ok(Self {
            device: device,
            game_state: game_state,
            chunk_cache: chunk_cache,
            start_instant: time::Instant::now(),
        })
    }
    pub fn into_device(self) -> D {
        self.device
    }
    pub fn send_event(&mut self, event: Event) {
        self.game_state.send_event(event)
    }
    pub fn render_frame(&mut self) -> Result<(), D::Error> {
        self.chunk_cache
            .set_world_state(self.game_state.get_world_state());
        let elapsed_time = self.start_instant.elapsed();
        let elapsed_time = elapsed_time.subsec_nanos() as f32 / 1e9 + elapsed_time.as_secs() as f32;
        let view_point = math::Vec3::<f32>::new(0.0, 0.0, 0.0);
        let view_transform = math::Mat4::translation(-view_point)
            .rotate(
                (elapsed_time * 30.0).to_radians(),
                math::Vec3::new(1.0, 0.5, 1.0f32).normalize().unwrap(),
            ).rotate(
                (elapsed_time * 60.0).to_radians(),
                math::Vec3::new(1.0, -0.5, 1.0f32).normalize().unwrap(),
            );
        let render_command_buffers = self
            .chunk_cache
            .get_render_command_buffers(math::Vec3::new(0.0, 0.0, 0.0), 64.0)?;
        let loader_command_buffers = self.chunk_cache.get_loader_command_buffers();
        let near = 0.1;
        let far = 100.0;
        let final_transform =
            math::Mat4::<f32>::perspective_projection(-1.0, 1.0, -1.0, 1.0, near, far)
                * view_transform;
        self.device.render_frame(
            math::Vec4::new(0.0, 0.0, 0.0, 1.0),
            loader_command_buffers,
            &[RenderCommandBufferGroup {
                render_command_buffers: &render_command_buffers,
                final_transform: final_transform,
            }],
        )?;
        Ok(())
    }
}
