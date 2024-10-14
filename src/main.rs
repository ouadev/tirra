use std::borrow::Cow;

use iced::highlighter::{self};
use iced::theme::Theme;
use iced::time::{self, every};
use iced::{event, executor, Event};
use iced::{keyboard, window};
use iced::{Application, Command, Element, Settings, Subscription};

use crate::gui::pages::editor::{self, EditorPage};
use crate::gui::pages::login::{self, LoginPage};
use crate::gui::styles::style_constants::FONT_DEJAVU_SANS_MONO;
use crate::gui::styles::style_constants::FONT_DEJAVU_SANS_MONO_BYTES;

mod db;
mod gui;
mod tirracrypto;

// Constants
const TIRRA_DB_PATH: &str = "./tirra.db";

pub fn main() -> iced::Result {
    TirraIced::run(Settings {
        fonts: vec![Cow::Borrowed(FONT_DEJAVU_SANS_MONO_BYTES)],
        default_font: FONT_DEJAVU_SANS_MONO,
        window: window::Settings {
            icon: None,
            ..Default::default()
        },
        ..Settings::default()
    })
}

struct TirraIced {
    theme: highlighter::Theme,
    login_page: LoginPage,
    editor_page: Option<EditorPage>,
    logged_in: bool,
}

#[derive(Debug, Clone)]
enum Message {
    PeriodicTick,
    CtrlS,
    Editor(editor::Message),
    Login(login::Message),
    IgnoredEvent(Event),
}

impl Application for TirraIced {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        //let (editor_page, command) = EditorPage::new(TIRRA_DB_PATH);

        let (login_page, _login_cmd) = LoginPage::new(TIRRA_DB_PATH);

        //return
        (
            Self {
                theme: highlighter::Theme::InspiredGitHub,
                editor_page: None,
                login_page: login_page,
                logged_in: false,
            },
            //command.map(Message::Editor),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        if self.logged_in {
            match &self.editor_page {
                Some(e) => e.title(),
                _ => String::from("Unknown Tirra Page !"),
            }
        } else {
            self.login_page.title()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        if self.logged_in {
            match message {
                Message::PeriodicTick | Message::CtrlS => self
                    .editor_page
                    .as_mut()
                    .unwrap()
                    .update(editor::Message::SaveFile)
                    .map(Message::Editor),
                Message::Editor(msg) => self
                    .editor_page
                    .as_mut()
                    .unwrap()
                    .update(msg)
                    .map(Message::Editor),
                _ => Command::none(),
            }
        } else {
            match message {
                Message::Login(loginmsg) => {
                    //self.login_page.update(loginmsg).map(Message::Login)
                    match self.login_page.update(loginmsg) {
                        Some(login_msg) => {
                            // Login is successful
                            match login_msg {
                                login::Message::LoginSuccess => {
                                    self.logged_in = true;
                                    let (editor_page, command) = EditorPage::new(TIRRA_DB_PATH);
                                    self.editor_page = Some(editor_page);
                                    command.map(Message::Editor)
                                }
                                _ => Command::none(),
                            }
                        }
                        _ => Command::none(),
                    }
                }
                Message::IgnoredEvent(event) => {
                    match event {
                        Event::Window(_id, _ev) => {
                            //if let window::Event::Opened { position, size } = ev {
                            //    println!("Window Opened");
                            //}
                        }
                        _ => {}
                    }
                    //println!("other event");
                    Command::none()
                }
                _ => Command::none(),
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let kb_event = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::CtrlS),
            _ => None,
        });

        // Configure periodical save tick
        let tick_event = every(time::Duration::new(5, 0)).map(|_| Message::PeriodicTick);

        // Other application events
        let other_events = event::listen().map(Message::IgnoredEvent);

        Subscription::batch(vec![tick_event, kb_event, other_events])
    }

    fn view(&self) -> Element<Message> {
        if self.logged_in {
            self.editor_page
                .as_ref()
                .unwrap()
                .view()
                .map(Message::Editor)
        } else {
            self.login_page.view().map(Message::Login)
        }
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
