pub mod resource_manager;
pub mod error;

use crate::{
    core::{
        math::vec2::Vec2,
        visitor::{Visitor, VisitResult, Visit},
    },
    sound::context::Context,
    engine::{resource_manager::ResourceManager, error::EngineError},
    gui::{
        UserInterface,
    },
    renderer::{Renderer, error::RendererError, gl},
    window::{WindowBuilder, Window},
    scene::SceneContainer,
    PossiblyCurrent,
    GlRequest,
    GlProfile,
    WindowedContext,
    NotCurrent,
    Api,
    event_loop::EventLoop,
};
use std::sync::{Arc, Mutex};

pub struct Engine {
    context: glutin::WindowedContext<PossiblyCurrent>,
    pub renderer: Renderer,
    pub user_interface: UserInterface,
    pub sound_context: Arc<Mutex<Context>>,
    pub resource_manager: ResourceManager,
    pub scenes: SceneContainer,
}

impl Engine {
    /// Creates new instance of engine from given window builder and events loop.
    ///
    /// Automatically creates all sub-systems (renderer, sound, ui, etc.).
    ///
    /// # Examples
    ///
    /// ```
    /// use rg3d::engine::Engine;
    /// use rg3d::window::WindowBuilder;
    /// use rg3d::event_loop::EventLoop;
    ///
    /// let evt = EventLoop::new();
    /// let window_builder = WindowBuilder::new()
    ///     .with_title("Test")
    ///     .with_fullscreen(None);
    /// let mut engine = Engine::new(window_builder, &evt).unwrap();
    /// ```
    #[inline]
    pub fn new(window_builder: WindowBuilder, events_loop: &EventLoop<()>) -> Result<Engine, EngineError> {
        let context_wrapper: WindowedContext<NotCurrent> = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_gl_profile(GlProfile::Core)
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .build_windowed(window_builder, events_loop)?;

        let context = unsafe {
            let context = match context_wrapper.make_current() {
                Ok(context) => context,
                Err((_, e)) => return Err(EngineError::from(e)),
            };
            gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            context
        };

        let client_size = context.window().inner_size();

        Ok(Engine {
            context,
            resource_manager: ResourceManager::new(),
            sound_context: Context::new()?,
            scenes: SceneContainer::new(),
            renderer: Renderer::new(client_size.into())?,
            user_interface: UserInterface::new(),
        })
    }

    /// Returns reference to main window.  Could be useful to set fullscreen mode, change
    /// size of window, its title, etc.
    #[inline]
    pub fn get_window(&self) -> &Window {
        self.context.window()
    }

    /// Performs single update tick with given time delta. Engine internally will perform update
    /// of all scenes, sub-systems, user interface, etc. Must be called in order to get engine
    /// functioning.
    pub fn update(&mut self, dt: f32) {
        let client_size = self.context.window().inner_size();
        let aspect_ratio = client_size.width as f32 / client_size.height as f32;

        self.resource_manager.update();

        for scene in self.scenes.iter_mut() {
            scene.update(aspect_ratio, dt);
        }

        self.user_interface.update(Vec2::new(client_size.width as f32, client_size.height as f32), dt);
    }

    pub fn get_ui_mut(&mut self) -> &mut UserInterface {
        &mut self.user_interface
    }

    #[inline]
    pub fn render(&mut self) -> Result<(), RendererError> {
        self.renderer.upload_resources(&mut self.resource_manager);
        self.user_interface.draw();
        self.renderer.render(&self.scenes, &self.user_interface.get_drawing_context(), &self.context)
    }
}

impl Visit for Engine {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        if visitor.is_reading() {
            self.resource_manager.update();
            self.scenes.clear();
        }

        self.resource_manager.visit("ResourceManager", visitor)?;
        self.scenes.visit("Scenes", visitor)?;
        self.sound_context.lock()?.visit("SoundContext", visitor)?;

        if visitor.is_reading() {
            self.resource_manager.reload_resources();
            for scene in self.scenes.iter_mut() {
                scene.resolve();
            }
        }

        visitor.leave_region()
    }
}

