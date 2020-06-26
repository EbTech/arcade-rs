mod phi;
mod views;


fn main() {
    phi::spawn("ArcadeRS Shooter", |phi| {
        Box::new(views::main_menu::MainMenuView::new(phi))
    });
}
