#[macro_use]
mod events;
pub mod data;
pub mod gfx;

use self::gfx::Sprite;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::ttf::Sdl2TtfContext;
use std::collections::HashMap;
use std::path::Path;


struct_events! {
    keyboard: {
        key_escape: Escape,
        key_up: Up,
        key_down: Down,
        key_left: Left,
        key_right: Right,
        key_space: Space,
        key_return: Return,
        
        key_1: Num1,
        key_2: Num2,
        key_3: Num3
    },
    else: {
        quit: Quit { .. }
    }
}

/// Bundles the Phi abstractions in a single structure which
/// can be passed easily between functions.
pub struct Phi<'ttf, 'r> {
    pub events: Events,
    pub renderer: WindowCanvas,
    
    ttf_context: Sdl2TtfContext,
    allocated_channels: i32,
    cached_fonts: HashMap<(&'static str, u16), sdl2::ttf::Font<'ttf, 'r>>,
}

impl<'ttf, 'r> Phi<'ttf, 'r> {
    fn new(events: Events, renderer: WindowCanvas, ttf_context: Sdl2TtfContext) -> Phi<'ttf, 'r> {
        // We start with 32 mixer channels, which we may grow if necessary.
        let allocated_channels = 32;
        sdl2::mixer::allocate_channels(allocated_channels);
        
        Phi {
            events,
            renderer,
            ttf_context,
            allocated_channels,
            cached_fonts: HashMap::new(),
        }
    }
    
    pub fn output_size(&self) -> (f64, f64) {
        let (w, h) = self.renderer.output_size().unwrap();
        (w as f64, h as f64)
    }
    
    /// Renders a string of text as a sprite using the provided parameters.
    pub fn ttf_str_sprite(&mut self, text: &str, font_path: &'static str, size: u16, color: Color) -> Option<Sprite> {
        //? First, we verify whether the font is already cached. If this is the
        //? case, we use it to render the text.
        if let Some(font) = self.cached_fonts.get(&(font_path, size)) {
            return font.render(text).blended(color).ok()
                .and_then(|surface| self.renderer.texture_creator().create_texture_from_surface(&surface).ok())
                .map(Sprite::new)
        }

        //? Otherwise, we start by trying to load the requested font.
        self.ttf_context.load_font(Path::new(font_path), size).ok()
            .and_then(|font| {
                //? If this works, we cache the font we acquired.
                self.cached_fonts.insert((font_path, size), font);
                //? Then, we call the method recursively. Because we know that
                //? the font has been cached, the `if` block will be executed
                //? and the sprite will be appropriately rendered.
                self.ttf_str_sprite(text, font_path, size, color)
            })
    }
    
    /// Play a sound once, and allocate new channels if this is necessary.
    pub fn play_sound(&mut self, sound: &::sdl2::mixer::Chunk) {
        // Attempt to play the sound once.
        match sdl2::mixer::Channel::all().play(sound, 0) {
            Err(_) => {
                // If there weren't enough channels allocated, then we double
                // that number and try again.
                self.allocated_channels *= 2;
                sdl2::mixer::allocate_channels(self.allocated_channels);
                self.play_sound(sound);
                println!("Allocating {} channels.", self.allocated_channels);
            },

            _ => { /* Everything's Alright! */ }
        }
    }
}

/// A `ViewAction` is a way for the currently executed view to communicate with
/// the game loop. It specifies whether an action should be executed before the
/// next rendering.
pub enum ViewAction {
    Render(Box<dyn View>),
    Quit,
}

pub trait View {
    /// Called on every frame to take care of the logic of the program. From
    /// user inputs and the instance's internal state, determine whether to
    /// render itself or another view, close the window, etc.
    ///
    /// `elapsed` is expressed in seconds.
    fn update(self: Box<Self>, context: &mut Phi, elapsed: f64) -> ViewAction;

    /// Called on every frame to take care rendering the current view. It
    /// disallows mutating the object by default, although you may still do it
    /// through a `RefCell` if you need to.
    fn render(&self, context: &mut Phi);
}

pub fn spawn<F>(title: &str, init: F)
where F: Fn(&mut Phi) -> Box<dyn View> {
    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();
    let mut timer = sdl_context.timer().unwrap();
    let _image_context = sdl2::image::init(sdl2::image::InitFlag::PNG).unwrap();
    let _ttf_context = sdl2::ttf::init().unwrap();
    
    // Initialize audio plugin
    let _mixer_context = sdl2::mixer::init(sdl2::mixer::InitFlag::OGG).unwrap();
    sdl2::mixer::open_audio(44100, sdl2::mixer::AUDIO_S16LSB, 2, 1024).unwrap();
    
    // Create the window
    let window = video.window(title, 800, 600)
        .position_centered().opengl().resizable()
        .build().unwrap();

    // Create the context
    let mut context = Phi::new(
        Events::new(sdl_context.event_pump().unwrap()),
        window.into_canvas().accelerated().build().unwrap(),
        _ttf_context);

    // Create the default view
    let mut current_view = init(&mut context);


    // Frame timing

    let interval = 1_000 / 60;
    let mut before = timer.ticks();
    let mut last_second = timer.ticks();
    let mut fps = 0u16;

    loop {
        // Frame timing (bis)

        let now = timer.ticks();
        let dt = now - before;
        let elapsed = dt as f64 / 1_000.0;

        // If the time elapsed since the last frame is too small, wait out the
        // difference and try again.
        if dt < interval {
            timer.delay(interval - dt);
            continue;
        }

        before = now;
        fps += 1;

        if now - last_second > 1_000 {
            println!("FPS: {}", fps);
            last_second = now;
            fps = 0;
        }

        // Logic & rendering

        context.events.pump(&mut context.renderer);

        match current_view.update(&mut context, elapsed) {
            ViewAction::Render(view) => {
                current_view = view;
                current_view.render(&mut context);
                context.renderer.present();
            },

            ViewAction::Quit =>
                break,
        }
    }
}
