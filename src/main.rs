extern crate freetype;
extern crate gl;
extern crate glutin;

mod breakout;
mod collision;
mod fonts;
mod graphics;
mod math;
mod renderer;

use glutin::{
    Api as GlApi,
    ContextBuilder,
    ElementState,
    Event,
    EventsLoop,
    GlContext,
    GlRequest,
    GlWindow,
    KeyboardInput,
    MouseButton,
    VirtualKeyCode,
    WindowBuilder,
    WindowEvent,
};
use std::time::Instant;

use breakout::Game;

fn main() {
    const INITIAL_WIDTH: u32 = 900;
    const INITIAL_HEIGHT: u32 = 900;

    let mut events_loop = EventsLoop::new();
    let window_spec = WindowBuilder::new()
        .with_title("B R E A K O U T ! ! - by Eugene Che")
        .with_dimensions(INITIAL_WIDTH, INITIAL_HEIGHT);
    let context_spec = ContextBuilder::new()
        .with_gl(GlRequest::Specific(GlApi::OpenGl, (3, 3)));
    
    let window = GlWindow::new(window_spec, context_spec, &events_loop).unwrap();

    unsafe {
        window.make_current().unwrap();
        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    }

    let mut game = Game::new(INITIAL_WIDTH as _, INITIAL_WIDTH as _);
    let mut is_running = true;
    let mut last_update = Instant::now();
    while is_running {
        events_loop.poll_events(|event| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => is_running = false,
                    WindowEvent::CursorMoved { position, .. } => {
                        let (x, y) = position;
                        game.on_mouse_motion(x as _, y as _);
                    },
                    WindowEvent::Resized(w, h) => {
                        game.on_viewport_change(w, h);
                    },
                    WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode: Some(keycode), .. }, .. } => {
                        game.on_key(keycode, state);
                    },
                    WindowEvent::MouseInput { button, state, .. } => {
                        game.on_mouse_button(button, state);
                    },
                    _ => (),
                },
                _  => (),
            }
        });

        let duration = last_update.elapsed();
        let dt = duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1_000_000_000.;
        last_update = Instant::now();

        game.step(dt);
        game.render();
        window.swap_buffers().unwrap();
    }
}
