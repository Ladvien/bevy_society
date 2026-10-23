//! **The headless recorder the capture example drives.**
//!
//! Rendering a demo to PNGs needs the same awkward things every time: `DefaultPlugins` with no window
//! and no winit, shaders compiled synchronously so frame 0 is not empty, a render target that is not
//! a surface, a camera pointed at it, a hand-pumped update loop, a device poll so a screenshot does
//! not read a half-drawn frame, and a few extra pumps at the end because screenshot readback lands a
//! frame or two late. None of that is interesting and all of it is easy to get subtly wrong.
//!
//! The scene and the script belong to the caller: what stands there, what happens to it, and on which
//! frame. That is the part worth reading.
//!
//! Simulated time is pinned with [`TimeUpdateStrategy::ManualDuration`], so a frame is exactly
//! `1/30 s` of game time whatever the host's real clock does. Without it a capture on a busy machine
//! records a different number of society ticks than the same capture on an idle one, and the clip no
//! longer shows the run it claims to.

use bevy::{
    app::SubApps,
    asset::RenderAssetUsages,
    camera::RenderTarget,
    prelude::*,
    render::{
        RenderPlugin,
        render_resource::{Extent3d, PollType, TextureDimension, TextureFormat, TextureUsages},
        renderer::RenderDevice,
        view::screenshot::{Screenshot, save_to_disk},
    },
    time::TimeUpdateStrategy,
    window::ExitCondition,
    winit::WinitPlugin,
};
use std::time::Duration;

/// Seconds of simulated time one recorded frame advances.
pub const FRAME_SECONDS: f64 = 1.0 / 30.0;

/// A headless Bevy app that writes one PNG per frame.
pub struct Recorder {
    app: SubApps,
    target: Handle<Image>,
    out: String,
    frame: u32,
}

impl Recorder {
    /// Builds the app, its off-screen target and a camera looking through `camera`.
    ///
    /// `None`, with an `error!` naming the path, when the output directory cannot be created: a
    /// recorder that cannot write is refused at the door rather than 100 frames in.
    pub fn new(width: u32, height: u32, camera: Transform, out: &str) -> Option<Recorder> {
        Recorder::new_with(width, height, camera, out, |_| {})
    }

    /// [`Recorder::new`], plus a chance to add plugins before `finish`/`cleanup`.
    ///
    /// After `cleanup` an `App` will not accept another plugin, and this recorder calls both itself
    /// because it never calls `run()`. That makes this the only window in which a caller can add one.
    pub fn new_with(
        width: u32,
        height: u32,
        camera: Transform,
        out: &str,
        add_plugins: impl FnOnce(&mut App),
    ) -> Option<Recorder> {
        if let Err(e) = std::fs::create_dir_all(out) {
            error!("capture: cannot create {out}: {e}");
            return None;
        }

        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                // Every shader must be ready before the first frame, or frame 0 renders empty.
                .set(RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                .disable::<WinitPlugin>(),
        );
        add_plugins(&mut app);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            FRAME_SECONDS,
        )));
        // `run()` is never called, so the two things the runner would have done are done here.
        app.finish();
        app.cleanup();
        let mut app = std::mem::take(app.sub_apps_mut());

        let mut image = Image::new_uninit(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
        let target = app
            .main
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(image);

        app.main
            .world_mut()
            .spawn((Camera3d::default(), RenderTarget::from(target.clone()), camera));

        Some(Recorder {
            app,
            target,
            out: out.to_owned(),
            frame: 0,
        })
    }

    /// The main world, for building a scene or changing it between frames.
    pub fn world(&mut self) -> &mut World {
        self.app.main.world_mut()
    }

    /// The sub-apps, for the one thing `world()` cannot do: adding a system after `cleanup`.
    pub fn app(&mut self) -> &mut SubApps {
        &mut self.app
    }

    /// Screenshots the world as it stands, then advances it one frame.
    pub fn shoot(&mut self) {
        let path = format!("{}/frame{:04}.png", self.out, self.frame);
        self.app
            .main
            .world_mut()
            .spawn(Screenshot::image(self.target.clone()))
            .observe(save_to_disk(path));
        self.frame += 1;
        self.step();
    }

    /// Draws and discards `frames` frames, so the first *recorded* one is not the blank still a GIF
    /// viewer sees first. Call it after the scene exists and before the recording loop.
    pub fn warm_up(&mut self, frames: u32) {
        for _ in 0..frames {
            self.step();
        }
    }

    /// Advances one frame without recording it.
    pub fn step(&mut self) {
        self.app.update();
        // Without the wait, a screenshot can read back a half-drawn target.
        let device = self
            .app
            .main
            .world()
            .resource::<RenderDevice>()
            .wgpu_device()
            .clone();
        if let Err(e) = device.poll(PollType::Wait {
            submission_index: None,
            timeout: None,
        }) {
            warn!("capture: device poll failed, frame may be torn: {e:?}");
        }
    }

    /// Pumps past the last screenshot so its readback lands, and reports how many frames were written.
    pub fn finish(mut self) -> u32 {
        for _ in 0..4 {
            self.step();
        }
        self.frame
    }
}
