use std::hash::Hasher;

use iced::{Application, executor, Theme, Command, Renderer, widget::{container, column, text_input, image}, futures::{StreamExt, stream::BoxStream}, Event};
use shared::primitive::Size;
use crate::state::{browser::{Browser, BrowserHandler}, browser_tab::TabEvent};

pub struct Moon {
    browser: BrowserHandler,
    url_input_content: String,
    content_width: u32,
    content_height: u32,
    content_data: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum Message {
    URLInputContentChanged(String),
    URLNavigationTriggered,
    ContentDataChanged(Vec<u8>),
    WindowResized(u32, u32),
    MouseScrolled(f32, f32),
    NoOp
}

impl Application for Moon {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Self::Message>) {
        let browser = Browser::new();
        let handler = browser.handler();
        std::thread::spawn(move || {
            browser.run().expect("Browser panic");
        });

        let instance = Moon {
            browser: handler,
            url_input_content: String::new(),
            content_width: 0,
            content_height: 0,
            content_data: Vec::new()
        };
        (instance, Command::none())
    }

    fn title(&self) -> String {
        String::from("Moon browser")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
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
                self.content_height = height - 30;
                self.browser.resize(Size::new(width as f32, height as f32));
            }
            Message::MouseScrolled(_, y) => {
                self.browser.scroll(-y);
            }
            Message::NoOp => {}
        }
        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        struct BrowserSub(BrowserHandler);
        impl<H: Hasher, M> iced::subscription::Recipe<H, M> for BrowserSub {
            type Output = (usize, TabEvent);

            fn hash(&self, _: &mut H) {
                // TODO: implement this
            }

            fn stream(
                self: Box<Self>,
                _: BoxStream<M>,
            ) -> BoxStream<Self::Output> {
                self.0.events().into_stream().boxed()
            }
        }
        let browser_sub = iced::Subscription::from_recipe(BrowserSub(self.browser.clone()))
            .map(|(_, tab_event)| match tab_event {
                TabEvent::FrameReceived(data) => Message::ContentDataChanged(data),
                _ => Message::NoOp
            });

        let events_sub = iced::subscription::events()
            .map(|event| match event {
                Event::Window(iced::window::Event::Resized { width, height }) => {
                    Message::WindowResized(width, height)
                }
                Event::Mouse(iced::mouse::Event::WheelScrolled { delta }) => match delta {
                    iced::mouse::ScrollDelta::Pixels { x, y } => {
                        Message::MouseScrolled(x, y)
                    }
                    _ => Message::NoOp
                }
                _ => Message::NoOp
            });

        let subs = vec![browser_sub, events_sub];
        iced::Subscription::batch(subs)
    }

    fn view(&self) -> iced::Element<Self::Message, Renderer<Self::Theme>> {
        let content = column![
            primary_bar(&self.url_input_content),
            content_area(self.content_width, self.content_height, self.content_data.clone()),
        ];
        container(content).into()
    }
}

fn primary_bar(
    url_content: &str
) -> iced::Element<'static, Message> {
    text_input("Go to...", url_content, Message::URLInputContentChanged)
        .on_submit(Message::URLNavigationTriggered)
        .into()
}

fn content_area(
    width: u32,
    height: u32,
    content: Vec<u8>
) -> iced::Element<'static, Message> {
    let image_handle = image::Handle::from_pixels(width, height, content);
    let content_image = iced::widget::image(image_handle)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill);
    
    content_image.into()
}
