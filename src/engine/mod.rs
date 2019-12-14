use std::env;
use std::path;

use ggez::conf::NumSamples;
use ggez::{conf, event, ContextBuilder};
use specs::{Dispatcher, World};

pub mod camera;
pub mod camera_handler;
pub mod components;
pub mod game_loop;
pub mod render_context;
pub mod resources;
pub mod shape_render;
pub mod systems;

const PHYSICS_UPDATES: usize = 4;

pub fn start<'a>(world: World, schedule: Dispatcher<'a, 'a>) {
    let mut c = conf::Conf::new();
    if cfg!(target_os = "windows") {
        c.window_mode = c.window_mode.dimensions(1600., 900.);
    } else {
        c.window_mode = c.window_mode.dimensions(800., 600.);
    }
    c.window_setup = c.window_setup.vsync(true).samples(NumSamples::Four);

    let mut cb = ContextBuilder::new("Sandbox", "Uriopass").conf(c);

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();

    let mut state = game_loop::EngineState::new(world, schedule, ctx).unwrap();

    state.cam.camera.zoom = 0.5;
    state.cam.camera.position.y = 500.;

    event::run(ctx, event_loop, &mut state).unwrap()
}