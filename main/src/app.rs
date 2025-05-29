use crate::state::{
    browser::{Browser, BrowserHandler},
    browser_tab::TabEvent,
};
use flume::Receiver;
use iced::{
    futures::{stream, SinkExt, Stream, StreamExt},
    keyboard::{Key, Modifiers},
    widget::{button, canvas, column, container, image, row, text, text_input},
    Event, Font, Renderer,
};
use shared::primitive::{Point, Size};

pub struct Moon {
    browser: BrowserHandler,
    url_input_content: String,
    content_width: f32,
    content_height: f32,
    content_data: Vec<u8>,
    title: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    URLInputContentChanged(String),
    URLNavigationTriggered,
    ContentDataChanged(Vec<u8>),
    WindowResized(f32, f32),
    MouseScrolled(f32, f32),
    MouseMoved(f32, f32),
    KeyPressed(Key, Modifiers),
    TitleChanged(String),
    ReloadTriggered,
    NoOp,
}

impl Default for Moon {
    fn default() -> Self {
        let browser = Browser::new();
        let handler = browser.handler();
        std::thread::spawn(move || {
            browser.run().expect("Browser panic");
        });

        Self {
            browser: handler,
            url_input_content: String::new(),
            content_width: 0.,
            content_height: 0.,
            content_data: Vec::new(),
            title: String::new(),
        }
    }
}

impl Moon {
    fn title(&self) -> String {
        self.title.clone()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::URLNavigationTriggered => {
                let url = self.url_input_content.clone();
                self.browser.goto(url);
            }
            Message::URLInputContentChanged(url) => {
                self.url_input_content = url;
            }
            Message::ContentDataChanged(data) => {
                self.content_data = data;
            }
            Message::WindowResized(width, height) => {
                self.content_width = width;
                self.content_height = height - 40.;
                self.browser.resize(Size::new(width, height));
            }
            Message::MouseScrolled(_, y) => {
                self.browser.scroll(-y);
            }
            Message::MouseMoved(x, y) => {
                self.browser.handle_mouse_move(Point::new(x, y));
            }
            Message::KeyPressed(Key::Named(iced::keyboard::key::Named::F5), _)
            | Message::ReloadTriggered => {
                self.browser.reload();
            }
            Message::KeyPressed(_, Modifiers::CTRL) => {
                self.browser.view_source_current_tab();
            }
            Message::TitleChanged(new_title) => {
                self.title = new_title;
            }
            Message::KeyPressed(_, _) => {}
            Message::NoOp => {}
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let browser_events = self.browser.events();
        fn browser_events_handler(
            browser_events: Receiver<(usize, TabEvent)>,
        ) -> impl Stream<Item = Message> {
            iced::stream::channel(1, move |mut output| async move {
                loop {
                    let (_, tab_event) = browser_events.recv().expect("Tab disconected!");
                    let message = match tab_event {
                        TabEvent::FrameReceived(data) => Message::ContentDataChanged(data),
                        TabEvent::TitleChanged(new_title) => Message::TitleChanged(new_title),
                        TabEvent::URLChanged(new_url) => {
                            Message::URLInputContentChanged(new_url.as_str())
                        }
                        _ => Message::NoOp,
                    };
                    output.send(message).await.unwrap();
                }
            })
        }
        let browser_sub = iced::Subscription::run_with_id(
            1,
            browser_events_handler(browser_events).map(|message| message),
        );

        let events_sub = iced::event::listen().map(|event| match event {
            Event::Window(iced::window::Event::Resized(size)) => {
                Message::WindowResized(size.width, size.height)
            }
            Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => match delta {
                iced::mouse::ScrollDelta::Pixels { x, y } => Message::MouseScrolled(x, y),
                _ => Message::NoOp,
            },
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                Message::MouseMoved(position.x, position.y)
            }
            Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::NoOp,
        });

        let subs = vec![browser_sub, events_sub];
        iced::Subscription::batch(subs)
    }

    pub fn view(&self) -> iced::Element<Message> {
        let content = column![
            row![reload_button(), primary_bar(&self.url_input_content),],
            content_area(
                self.content_width,
                self.content_height,
                self.content_data.clone()
            ),
        ];
        container(content).into()
    }
}

fn reload_button() -> iced::Element<'static, Message> {
    let icon = text('\u{ec7f}')
        .font(Font::with_name("IcoFont"))
        .align_y(iced::alignment::Vertical::Center)
        .align_x(iced::alignment::Horizontal::Center);
    button(icon)
        .width(iced::Length::Fixed(40.))
        .height(iced::Length::Fixed(40.))
        .on_press(Message::ReloadTriggered)
        .into()
}

fn primary_bar(url_content: &str) -> iced::Element<'static, Message> {
    text_input("Go to...", url_content)
        .on_input(Message::URLInputContentChanged)
        .on_submit(Message::URLNavigationTriggered)
        .icon(text_input::Icon {
            font: Font::with_name("IcoFont"),
            side: text_input::Side::Left,
            code_point: '\u{ed11}',
            size: Some(iced::Pixels(16.)),
            spacing: 10.,
        })
        .padding(10)
        .into()
}

fn content_area(width: f32, height: f32, content: Vec<u8>) -> iced::Element<'static, Message> {
    let image_handle = image::Handle::from_bytes(content);
    let content_image = iced::widget::image(image_handle)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);

    content_image.into()
}
