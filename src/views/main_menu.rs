use crate::phi::{Phi, View, ViewAction};
use crate::phi::data::Rectangle;
use crate::phi::gfx::{CopySprite, Sprite};
use crate::views::shared::BgSet;
use sdl2::pixels::Color;


const ACTION_FONT: &'static str = "assets/belligerent.ttf";

struct Action<'r> {
    func: Box<dyn Fn(&mut Phi, BgSet) -> ViewAction>,

    idle_sprite: Sprite<'r>,
    hover_sprite: Sprite<'r>,
}

impl<'r> Action<'r> {
    fn new(phi: &mut Phi, label: &'static str, func: Box<dyn Fn(&mut Phi, BgSet) -> ViewAction>) -> Action<'r> {
        Action {
            func: func,
            idle_sprite: phi.ttf_str_sprite(label, ACTION_FONT, 32, Color::RGB(220, 220, 220)).unwrap(),
            hover_sprite: phi.ttf_str_sprite(label, ACTION_FONT, 42, Color::RGB(255, 255, 255)).unwrap(),
        }
    }
}

pub struct MainMenuView<'r> {
    actions: Vec<Action<'r>>,
    selected: i8,
    bg: BgSet<'r>,
}

impl<'r> MainMenuView<'r> {
    pub fn new(phi: &mut Phi) -> MainMenuView<'r> {
        let bg = BgSet::new(&mut phi.renderer);
        MainMenuView::with_backgrounds(phi, bg)
    }
    
    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet<'r>) -> MainMenuView<'r> {
        MainMenuView {
            actions: vec![
                Action::new(phi, "New Game", Box::new(|phi, bg| {
                    ViewAction::Render(Box::new(
                        crate::views::game::GameView::with_backgrounds(phi, bg)))
                })),
                Action::new(phi, "Quit", Box::new(|_, _| {
                    ViewAction::Quit
                })),
            ],
            selected: 0,
            bg: bg,
        }
    }
}

impl<'r> View for MainMenuView<'r> {
    fn update(mut self : Box<Self>, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
            return ViewAction::Quit;
        }

        // Execute the currently selected action if requested
        if phi.events.now.key_space == Some(true) ||
            phi.events.now.key_return == Some(true) {
            let bg = self.bg.clone();
            return (self.actions[self.selected as usize].func)(phi, bg);
        }

        // Change the selected action using the keyboard
        if phi.events.now.key_up == Some(true) {
            self.selected -= 1;
            if self.selected < 0 {
                self.selected = self.actions.len() as i8 - 1;
            }
        }

        if phi.events.now.key_down == Some(true) {
            self.selected += 1;
            if self.selected >= self.actions.len() as i8 {
                self.selected = 0;
            }
        }
        
        // Update the backgrounds
        self.bg.back.update(elapsed);
        self.bg.middle.update(elapsed);
        self.bg.front.update(elapsed);

        ViewAction::Render(self)
    }
    
    fn render(&self, phi: &mut Phi) {
        // Clear the screen.
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();
        
        // Render the backgrounds
        self.bg.back.render(&mut phi.renderer);
        self.bg.middle.render(&mut phi.renderer);
        self.bg.front.render(&mut phi.renderer);

        // Definitions for the menu's layout
        let (win_w, win_h) = phi.output_size();
        let label_h = 50.0;
        let border_width = 3.0;
        let box_w = 360.0;
        let box_h = self.actions.len() as f64 * label_h;
        let margin_h = 10.0;

        // Render the border of the colored box which holds the labels
        phi.renderer.set_draw_color(Color::RGB(70, 15, 70));
        phi.renderer.fill_rect(
            Rectangle::with_size(box_w + border_width * 2.0, box_h + border_width * 2.0 + margin_h * 2.0)
            .center_at((win_w / 2.0, win_h / 2.0)).to_sdl().unwrap());

        // Render the colored box which holds the labels
        phi.renderer.set_draw_color(Color::RGB(140, 30, 140));
        phi.renderer.fill_rect(
            Rectangle::with_size(box_w, box_h + margin_h * 2.0)
            .center_at((win_w / 2.0, win_h / 2.0)).to_sdl().unwrap());

        // Render the labels in the menu
        for (i, action) in self.actions.iter().enumerate() {
            if self.selected as usize == i {
                let (w, h) = action.hover_sprite.size();
                phi.renderer.copy_sprite(&action.hover_sprite,
                    Rectangle::with_size(w, h)
                    .center_at((win_w / 2.0, (win_h - box_h + label_h) / 2.0 + label_h * i as f64)));
            } else {
                let (w, h) = action.idle_sprite.size();
                phi.renderer.copy_sprite(&action.idle_sprite,
                    Rectangle::with_size(w, h)
                    .center_at((win_w / 2.0, (win_h - box_h + label_h) / 2.0 + label_h * i as f64)));
            }
        }
    }
}
