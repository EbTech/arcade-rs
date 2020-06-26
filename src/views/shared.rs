use crate::phi::data::Rectangle;
use crate::phi::gfx::{CopySprite, Sprite};
use sdl2::render::WindowCanvas;

#[derive(Clone)]
pub struct Background<'r> {
    pub pos: f64,
    pub vel: f64,
    pub sprite: Sprite<'r>,
}

impl<'r> Background<'r> {
    /// Move the background proportionally to the elapsed time since the last
    /// frame and the background's velocity.
    pub fn update(&mut self, elapsed: f64) {
        // We define a logical position as depending solely on the time and the
        // dimensions of the image, not on the screen's size.
        let size = self.sprite.size();
        self.pos += self.vel * elapsed;
        if self.pos > size.0 {
            self.pos -= size.0;
        }
    }

    /// Render the background at its current position, and as many times as
    /// required to fill the screen.
    pub fn render(&self, renderer: &mut WindowCanvas) {
        // We determine the scale ratio of the window to the sprite.
        let size = self.sprite.size();
        let (win_w, win_h) = renderer.output_size().unwrap();
        let scale = win_h as f64 / size.1;

        // We render as many copies of the background as necessary to fill
        // the screen.
        let mut physical_left = -self.pos * scale;

        while physical_left < win_w as f64 {
            renderer.copy_sprite(&self.sprite, Rectangle {
                x: physical_left,
                y: 0.0,
                w: size.0 * scale,
                h: win_h as f64,
            });

            physical_left += size.0 * scale;
        }
    }
}

#[derive(Clone)]
pub struct BgSet<'r> {
    pub back: Background<'r>,
    pub middle: Background<'r>,
    pub front: Background<'r>,
}

impl<'r> BgSet<'r> {
    pub fn new(renderer: &mut WindowCanvas) -> BgSet<'r> {
        BgSet {
            back: Background {
                pos: 0.0,
                vel: 20.0,
                sprite: Sprite::load(renderer, "assets/starBG.png").unwrap(),
            },
            middle: Background {
                pos: 0.0,
                vel: 40.0,
                sprite: Sprite::load(renderer, "assets/starMG.png").unwrap(),
            },
            front: Background {
                pos: 0.0,
                vel: 80.0,
                sprite: Sprite::load(renderer, "assets/starFG.png").unwrap(),
            },
        }
    }
}
