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

use block::{Block, BlockLighting, GlobalRenderProperties, LightLevel};
use chunk_cache::ChunkCache;
use hashtable::DefaultBuildHasher;
use math::{self, Dot, Mappable, Reducible};
use quantiles::ckms::CKMS;
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
        let time_per_loop = time::Duration::from_secs(1) / 20;
        let air_block_id = registry.find_block_by_name("voxels:air").unwrap();
        let stone_block_id = registry.find_block_by_name("voxels:stone").unwrap();
        let stone_block = Block::new(
            stone_block_id,
            BlockLighting::new(LightLevel::MAX, LightLevel::MAX, LightLevel::MAX),
        );
        let air_block = Block::new(
            air_block_id,
            BlockLighting::new(LightLevel::MAX, LightLevel::MAX, LightLevel::MAX),
        );
        {
            let size = 20;
            let chunk_size = (size as u32).next_power_of_two();
            for xc in -1..1 {
                for yc in -1..1 {
                    for zc in -1..1 {
                        let chunk_start =
                            math::Vec3::new(xc, yc, zc) * math::Vec3::splat(chunk_size as i32);
                        world_state.set_cube_pow2(
                            &mut world,
                            chunk_start,
                            chunk_size,
                            |position: math::Vec3<u32>, original: Block| {
                                let position = position.map(|v| v as i32) + chunk_start;
                                if position.map(|v| v.abs() > size).reduce(|a, b| a || b) {
                                    return original;
                                }
                                if position.dot(position) >= size * size {
                                    stone_block
                                } else {
                                    air_block
                                }
                            },
                        );
                    }
                }
            }
        }
        let mut angle = 0;
        let angle_step_count = 20;
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
            let _ = elapsed_time;
            //println!("step duration: {:?}", elapsed_time);
            world_state.step(&mut world, 1);
            angle += 1;
            if angle >= angle_step_count {
                angle = 0;
            }
            let angle = angle as f32 * (360.0f32.to_radians() / angle_step_count as f32);
            let solid_transform =
                math::Mat4::<f32>::rotation(angle, math::Vec3::new(1.0, 0.0, 0.0));
            let size = 4;
            let chunk_size = (size as u32).next_power_of_two();
            for xc in -1..1 {
                for yc in -1..1 {
                    for zc in -1..1 {
                        let chunk_start =
                            math::Vec3::new(xc, yc, zc) * math::Vec3::splat(chunk_size as i32);
                        world_state.set_cube_pow2(
                            &mut world,
                            chunk_start,
                            chunk_size,
                            |position: math::Vec3<u32>, original: Block| {
                                let position = position.map(|v| v as i32) + chunk_start;
                                if position.map(|v| v.abs() > size).reduce(|a, b| a || b) {
                                    return original;
                                }
                                if position.dot(position) >= size * size
                                    && (solid_transform * math::Vec4::new(
                                        position.x, position.y, position.z, 1,
                                    ).map(|v| v as f32)).reduce(|a, b| a * b)
                                        > 0.0
                                {
                                    stone_block
                                } else {
                                    air_block
                                }
                            },
                        );
                    }
                }
            }
            for x in -size..=size {
                for y in -size..=size {
                    for z in -size..=size {
                        let position = math::Vec3::new(x, y, z);
                        world_state.set(
                            &mut world,
                            position,
                            if position.dot(position) >= size * size
                                && (solid_transform * math::Vec4::new(
                                    position.x, position.y, position.z, 1,
                                ).map(|v| v as f32)).reduce(|a, b| a * b)
                                    > 0.0
                            {
                                stone_block
                            } else {
                                air_block
                            },
                        );
                    }
                }
            }
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
    last_fps_report_instant: Option<time::Instant>,
    frames_since_last_fps_report: u32,
    last_frame_instant: Option<time::Instant>,
    frame_time_stats: CKMS<f64>,
    max_frame_duration: Option<time::Duration>,
    min_frame_duration: Option<time::Duration>,
}

fn duration_to_f64(v: time::Duration) -> f64 {
    v.as_secs() as f64 + 1e-9 * v.subsec_nanos() as f64
}

fn f64_to_duration(v: f64) -> time::Duration {
    let secs = v.floor();
    let nanos = ((v - secs) * 1e9).round() as u32;
    let secs = v as u64;
    time::Duration::new(secs, nanos)
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
            GlobalRenderProperties::default(),
            registry,
            tiles_image_set,
        );
        Ok(Self {
            device: device,
            game_state: game_state,
            chunk_cache: chunk_cache,
            start_instant: time::Instant::now(),
            last_fps_report_instant: None,
            frames_since_last_fps_report: 0,
            last_frame_instant: None,
            frame_time_stats: CKMS::new(1e-4),
            max_frame_duration: None,
            min_frame_duration: None,
        })
    }
    pub fn print_stats(&self) {
        if let Some(min_frame_duration) = self.min_frame_duration {
            println!("min frame duration: {:?}", min_frame_duration);
        }
        if let Some(max_frame_duration) = self.max_frame_duration {
            println!("max frame duration: {:?}", max_frame_duration);
        }
        for &percentile in &[90, 95, 99] {
            if let Some((_, value)) = self.frame_time_stats.query(percentile as f64 / 100.0) {
                println!(
                    "{}% frame duration: {:?}",
                    percentile,
                    f64_to_duration(value)
                );
            }
        }
    }
    pub fn into_device(self) -> D {
        self.device
    }
    pub fn send_event(&mut self, event: Event) {
        self.game_state.send_event(event)
    }
    pub fn render_frame(&mut self) -> Result<(), D::Error> {
        let current_instant = time::Instant::now();
        match self.last_frame_instant {
            None => {}
            Some(last_frame_instant) => {
                let frame_duration = current_instant.duration_since(last_frame_instant);
                match self.min_frame_duration {
                    Some(min_frame_duration) if min_frame_duration < frame_duration => {}
                    _ => self.min_frame_duration = Some(frame_duration),
                }
                match self.max_frame_duration {
                    Some(max_frame_duration) if max_frame_duration > frame_duration => {}
                    _ => self.max_frame_duration = Some(frame_duration),
                }
                self.frame_time_stats
                    .insert(duration_to_f64(frame_duration));
            }
        }
        self.last_frame_instant = Some(current_instant);
        match self.last_fps_report_instant.take() {
            None => {
                self.frames_since_last_fps_report = 0;
                self.last_fps_report_instant = Some(current_instant);
            }
            Some(last_fps_report_instant) => {
                self.frames_since_last_fps_report += 1;
                let elapsed = current_instant.duration_since(last_fps_report_instant);
                if elapsed >= time::Duration::from_secs(5) {
                    self.last_fps_report_instant = Some(current_instant);
                    let elapsed_per_frame = elapsed / self.frames_since_last_fps_report;
                    self.frames_since_last_fps_report = 0;
                    let fps = 1.0 / duration_to_f64(elapsed_per_frame);
                    println!("FPS: {} elapsed per frame: {:?}", fps, elapsed_per_frame);
                } else {
                    self.last_fps_report_instant = Some(last_fps_report_instant);
                }
            }
        }
        self.chunk_cache
            .set_world_state(self.game_state.get_world_state());
        let elapsed_time = duration_to_f64(current_instant.duration_since(self.start_instant));
        let view_point = math::Vec3::<f32>::new(0.5, 0.5, 0.5);
        let mut view_transform = math::Mat4::<f32>::identity();
        if false {
            view_transform = view_transform.rotate(
                (((elapsed_time * 30.0) % 360.0) as f32).to_radians(),
                math::Vec3::new(1.0, 0.0, 0.0),
            );
        }
        if true {
            view_transform = view_transform.rotate(
                (((elapsed_time * 10.0) % 360.0) as f32).to_radians(),
                math::Vec3::new(0.0, 1.0, 0.0),
            );
        }
        view_transform = view_transform.translate(-view_point);
        let mut loader_command_buffers = self.chunk_cache.get_loader_command_buffers();
        let render_command_buffers = self
            .chunk_cache
            .get_render_command_buffers(math::Vec3::new(0.0, 0.0, 0.0), 128.0)?;
        let dimensions = self.device.get_dimensions().map(|v| v as f32);
        let dimensions = dimensions / math::Vec2::splat(dimensions.x.min(dimensions.y));
        let near = 0.1;
        let far = 100.0;
        let final_transform = math::Mat4::<f32>::perspective_projection(
            -near * dimensions.x,
            near * dimensions.x,
            -near * dimensions.y,
            near * dimensions.y,
            near,
            far,
        ) * view_transform;
        self.device.render_frame(
            math::Vec4::new(0.0, 0.0, 0.0, 1.0),
            &mut loader_command_buffers,
            &[RenderCommandBufferGroup {
                render_command_buffers: &render_command_buffers,
                final_transform: final_transform,
            }],
        )?;
        Ok(())
    }
}
