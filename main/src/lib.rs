use app::Moon;
use iced::Font;

mod app;
mod render_client;
mod state;

pub fn start_main() -> iced::Result {
    iced::application("Moon", Moon::update, Moon::view)
        .subscription(Moon::subscription)
        .font(include_bytes!("../fonts/icofont.ttf").as_slice())
        .default_font(Font::MONOSPACE)
        .run()
}
