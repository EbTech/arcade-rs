use crate::phi::{Phi, View, ViewAction};
use crate::phi::data::{MaybeAlive, Rectangle};
use crate::phi::gfx::{AnimatedSprite, AnimatedSpriteDescr, CopySprite, Sprite};
use crate::views::shared::BgSet;
use crate::views::bullets::*;
use sdl2::pixels::Color;
use sdl2::mixer::{Chunk, Music};
use std::path::Path;

const DEBUG: bool = false;

const PLAYER_SPEED: f64 = 180.0;
const PLAYER_PATH: &'static str = "assets/spaceship.png";
const PLAYER_W: f64 = 43.0;
const PLAYER_H: f64 = 39.0;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const EXPLOSION_PATH: &'static str = "assets/explosion.png";
const EXPLOSIONS_WIDE: usize = 5;
const EXPLOSIONS_HIGH: usize = 4;
const EXPLOSIONS_TOTAL: usize = 17;
const EXPLOSION_SIDE: f64 = 96.0;
const EXPLOSION_FPS: f64 = 16.0;
const EXPLOSION_DURATION: f64 = 1.0 / EXPLOSION_FPS * EXPLOSIONS_TOTAL as f64;

/// The different states our ship might be in. In the image, they're ordered
/// from left to right, then from top to bottom.
#[derive(Clone, Copy)]
enum PlayerFrame {
    UpNorm   = 0,
    UpFast   = 1,
    UpSlow   = 2,
    MidNorm  = 3,
    MidFast  = 4,
    MidSlow  = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8
}

struct Player<'r> {
    rect: Rectangle,
    sprites: Vec<Sprite<'r>>,
    current: PlayerFrame,
    cannon: CannonType,
}

impl<'r> Player<'r> {
    pub fn new(phi: &mut Phi) -> Player<'r> {
        // Get the spaceship's sprites
        let spritesheet = Sprite::load(&mut phi.renderer, PLAYER_PATH).unwrap();
        let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    w: PLAYER_W,
                    h: PLAYER_H,
                    x: PLAYER_W * x as f64,
                    y: PLAYER_H * y as f64,
                }).unwrap());
            }
        }

        Player {
            // Spawn the player at the center of the screen, vertically.
            rect: Rectangle {
                x: 64.0,
                y: (phi.output_size().1 - PLAYER_H) / 2.0,
                w: PLAYER_W,
                h: PLAYER_H,
            },
            sprites: sprites,
            current: PlayerFrame::MidNorm,
            cannon: CannonType::RectBullet,
        }
    }
    
    pub fn update(&mut self, phi: &mut Phi, elapsed: f64) {
        // Change the player's cannons

        if phi.events.now.key_1 == Some(true) {
            self.cannon = CannonType::RectBullet;
        }

        if phi.events.now.key_2 == Some(true) {
            self.cannon = CannonType::SineBullet {
                amplitude: 10.0,
                angular_vel: 15.0,
            };
        }

        if phi.events.now.key_3 == Some(true) {
            self.cannon = CannonType::DivergentBullet {
                a: 100.0,
                b: 1.2,
            };
        }

        // Move the player's ship

        let diagonal =
            (phi.events.key_up ^ phi.events.key_down) &&
            (phi.events.key_left ^ phi.events.key_right);

        let moved =
            if diagonal { 0.5f64.sqrt() }
            else { 1.0 } * PLAYER_SPEED * elapsed;

        let dx = match (phi.events.key_left, phi.events.key_right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        let dy = match (phi.events.key_up, phi.events.key_down) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        self.rect.x += dx;
        self.rect.y += dy;

        // The movable region spans the entire height of the window and 70% of its
        // width. This way, the player cannot get to the far right of the screen, where
        // we will spawn the asteroids, and get immediately eliminated.
        //
        // We restrain the width because most screens are wider than they are high.
        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 as f64 * 0.70,
            h: phi.output_size().1 as f64,
        };

        // If the player cannot fit in the screen, then there is a problem and
        // the game should be promptly aborted.
        self.rect = self.rect.move_inside(movable_region).unwrap();

        // Select the appropriate sprite of the ship to show.
        self.current =
            if dx == 0.0 && dy < 0.0       { PlayerFrame::UpNorm }
            else if dx > 0.0 && dy < 0.0   { PlayerFrame::UpFast }
            else if dx < 0.0 && dy < 0.0   { PlayerFrame::UpSlow }
            else if dx == 0.0 && dy == 0.0 { PlayerFrame::MidNorm }
            else if dx > 0.0 && dy == 0.0  { PlayerFrame::MidFast }
            else if dx < 0.0 && dy == 0.0  { PlayerFrame::MidSlow }
            else if dx == 0.0 && dy > 0.0  { PlayerFrame::DownNorm }
            else if dx > 0.0 && dy > 0.0   { PlayerFrame::DownFast }
            else if dx < 0.0 && dy > 0.0   { PlayerFrame::DownSlow }
            else { unreachable!() };
    }
    
    pub fn render(&self, phi: &mut Phi) {
        // Render the bounding box (for debugging purposes)
        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
        }

        // Render the ship's current sprite.
        phi.renderer.copy_sprite(
            &self.sprites[self.current as usize],
            self.rect);
    }
    
    pub fn spawn_bullets(&self) -> Vec<Box<dyn Bullet>> {
        let cannons_x = self.rect.x + 30.0;
        let cannon1_y = self.rect.y + 6.0;
        let cannon2_y = self.rect.y + PLAYER_H - 10.0;

        spawn_bullets(self.cannon, cannons_x, cannon1_y, cannon2_y)
    }
}

struct Asteroid<'r> {
    sprite: AnimatedSprite<'r>,
    rect: Rectangle,
    vel: f64,
}

impl<'r> Asteroid<'r> {
    fn factory(phi: &mut Phi) -> AsteroidFactory<'r> {
        AsteroidFactory {
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    image_path: ASTEROID_PATH,
                    total_frames: ASTEROIDS_TOTAL,
                    frames_high: ASTEROIDS_HIGH,
                    frames_wide: ASTEROIDS_WIDE,
                    frame_w: ASTEROID_SIDE,
                    frame_h: ASTEROID_SIDE,
                }), 1.0),
        }
    }
    
    fn update(mut self, phi: &mut Phi, dt: f64) -> Option<Asteroid<'r>> {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROID_SIDE {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        if DEBUG {
            // Render the bounding box
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
        }
        
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }
    
    fn rect(&self) -> Rectangle {
        self.rect
    }
}

struct AsteroidFactory<'r> {
    sprite: AnimatedSprite<'r>,
}

impl<'r> AsteroidFactory<'r> {
    fn random(&self, phi: &mut Phi) -> Asteroid<'r> {
        let (w, h) = phi.output_size();

        // FPS in [10.0, 30.0)
        let mut sprite = self.sprite.clone();
        sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        Asteroid {
            sprite: sprite,

            // In the screen vertically, and over the right of the screen
            // horizontally.
            rect: Rectangle {
                w: ASTEROID_SIDE,
                h: ASTEROID_SIDE,
                x: w,
                y: rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
            },

            // vel in [50.0, 150.0)
            vel: rand::random::<f64>().abs() * 100.0 + 50.0,
        }
    }
}

struct Explosion<'r> {
    sprite: AnimatedSprite<'r>,
    rect: Rectangle,

    //? Keep how long its been arrived, so that we destroy the explosion once
    //? its animation is finished.
    alive_since: f64,
}

impl<'r> Explosion<'r> {
    fn factory(phi: &mut Phi) -> ExplosionFactory<'r> {
        ExplosionFactory {
            sprite: AnimatedSprite::with_fps(
                AnimatedSprite::load_frames(phi, AnimatedSpriteDescr {
                    image_path: EXPLOSION_PATH,
                    total_frames: EXPLOSIONS_TOTAL,
                    frames_high: EXPLOSIONS_HIGH,
                    frames_wide: EXPLOSIONS_WIDE,
                    frame_w: EXPLOSION_SIDE,
                    frame_h: EXPLOSION_SIDE,
                }), EXPLOSION_FPS),
        }
    }
    
    fn update(mut self, dt: f64) -> Option<Explosion<'r>> {
        self.alive_since += dt;
        self.sprite.add_time(dt);

        if self.alive_since >= EXPLOSION_DURATION {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }
}


struct ExplosionFactory<'r> {
    sprite: AnimatedSprite<'r>,
}

impl<'r> ExplosionFactory<'r> {
    fn at_center(&self, center: (f64, f64)) -> Explosion<'r> {
        let mut sprite = self.sprite.clone();

        Explosion {
            sprite: sprite,

            // In the screen vertically, and over the right of the screen
            // horizontally.
            rect: Rectangle::with_size(EXPLOSION_SIDE, EXPLOSION_SIDE)
                    .center_at(center),

            alive_since: 0.0,
        }
    }
}

pub struct GameView<'a> {
    player: Player<'a>,
    bullets: Vec<Box<dyn Bullet>>,
    asteroids: Vec<Asteroid<'a>>,
    asteroid_factory: AsteroidFactory<'a>,
    explosions: Vec<Explosion<'a>>,
    explosion_factory: ExplosionFactory<'a>,
    bg: BgSet<'a>,
    music: Music<'a>,
    bullet_sound: Chunk,
    explosion_sound: Chunk,
}

impl<'a> GameView<'a> {
    /*pub fn new(phi: &mut Phi) -> GameView {
        let bg = BgSet::new(&mut phi.renderer);
        GameView::with_backgrounds(phi, bg)
    }*/
    
    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> GameView<'a> {
        let music =
            Music::from_file(Path::new("assets/mdk_phoenix_orchestral.ogg"))
            .unwrap();
        
        music.play(-1).unwrap();
        
        let bullet_sound =
            Chunk::from_file(Path::new("assets/bullet.ogg"))
            .unwrap();

        let explosion_sound =
            Chunk::from_file(Path::new("assets/explosion.ogg"))
            .unwrap();
        
        GameView {
            // Entities
            player: Player::new(phi),
            
            bullets: vec![],
            asteroids: vec![],
            asteroid_factory: Asteroid::factory(phi),
            explosions: vec![],
            explosion_factory: Explosion::factory(phi),
            
            // Scenery
            bg: bg,
            
            // Audio
            music: music,
            bullet_sound: bullet_sound,
            explosion_sound: explosion_sound,
        }
    }
}

impl<'a> View for GameView<'a> {
    fn update(mut self: Box<Self>, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit {
            return ViewAction::Quit;
        }
        
        if phi.events.now.key_escape == Some(true) {
            let bg = self.bg.clone();
            return ViewAction::Render(Box::new(
                crate::views::main_menu::MainMenuView::with_backgrounds(phi, bg)))
            // TODO PauseView
            //return ViewAction::Render(Box::new(
            //    crate::views::main_menu::MainMenuView::with_game_state(self)));
        }
        
        { // begin reference scope
        // Take a reference to the content of the box (i.e. the view itself)
        let game = &mut *self;
        
        // Update the player
        game.player.update(phi, elapsed);
        
        // Update the bullets
        game.bullets =
            std::mem::replace(&mut game.bullets, vec![])
            .into_iter()
            .filter_map(|bullet| bullet.update(phi, elapsed))
            .collect();

        
        // Update the asteroids
        game.asteroids =
            std::mem::replace(&mut game.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| asteroid.update(phi, elapsed))
            .collect();
        
        // Update the explosions
        game.explosions =
            std::mem::replace(&mut game.explosions, vec![])
            .into_iter()
            .filter_map(|explosion| explosion.update(elapsed))
            .collect();
        
        // Collision detection
        
        let mut player_alive = true;
        
        let mut transition_bullets: Vec<_> =
            std::mem::replace(&mut game.bullets, vec![])
            .into_iter()
            .map(|bullet| MaybeAlive { alive: true, value: bullet })
            .collect();
        
        game.asteroids =
            std::mem::replace(&mut game.asteroids, vec![])
            .into_iter()
            .filter_map(|asteroid| {
                // By default, the asteroid has not been in a collision.
                let mut asteroid_alive = true;

                for bullet in &mut transition_bullets {
                    if asteroid.rect().overlaps(bullet.value.rect()) {
                        asteroid_alive = false;
                        bullet.alive = false;
                    }
                }

                // The player's ship is destroyed if it is hit by an asteroid.
                // In which case, the asteroid is also destroyed.
                if asteroid.rect().overlaps(game.player.rect) {
                    asteroid_alive = false;
                    player_alive = false;
                }
                
                if asteroid_alive {
                    Some(asteroid)
                } else {
                    // Spawn an explosive wherever an asteroid was destroyed.
                    game.explosions.push(
                        game.explosion_factory.at_center(
                            asteroid.rect().center()));
                    
                    phi.play_sound(&game.explosion_sound);
                    
                    None
                }
            })
            .collect();
        
        game.bullets = transition_bullets.into_iter()
            .filter_map(MaybeAlive::as_option)
            .collect();
        
        // TODO
        // For the moment, we won't do anything about the player dying. This will be
        // the subject of a future episode.
        if !player_alive {
            println!("The player's ship has been destroyed.");
        }
        
        // Allow the player to shoot after the bullets are updated, so that,
        // when rendered for the first time, they are drawn wherever they
        // spawned.
        if phi.events.now.key_space == Some(true) {
            game.bullets.append(&mut game.player.spawn_bullets());
            phi.play_sound(&game.bullet_sound);
        }
        
        // Randomly create an asteroid about once every 100 frames, that is,
        // a bit more often than once every two seconds.
        if rand::random::<usize>() % 100 == 0 {
            game.asteroids.push(game.asteroid_factory.random(phi));
        }
        
        // Update the backgrounds
        game.bg.back.update(elapsed);
        game.bg.middle.update(elapsed);
        game.bg.front.update(elapsed);
        } // end reference scope
        
        ViewAction::Render(self)
    }
    
    fn render(&self, phi: &mut Phi) {
        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();
        
        // Render the Backgrounds
        self.bg.back.render(&mut phi.renderer);
        self.bg.middle.render(&mut phi.renderer);
        
        // Render the entities

        self.player.render(phi);
        
        for bullet in &self.bullets {
            bullet.render(phi);
        }
        
        for asteroid in &self.asteroids {
            asteroid.render(phi);
        }
        
        for explosion in &self.explosions {
            explosion.render(phi);
        }
        
        // Render the foreground
        self.bg.front.render(&mut phi.renderer);
    }
}
