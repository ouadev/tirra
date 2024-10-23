use std::borrow::Cow;
use std::env;

use crate::widget::text_input;
use iced::highlighter::{self};
use iced::theme::Theme;
use iced::time::{self, every};
use iced::window::settings::PlatformSpecific;
use iced::{event, executor, widget, Event, Pixels};
use iced::{keyboard, window};
use iced::{Application, Command, Element, Settings, Subscription};

extern crate tirra;
use tirra::gui::pages::editor::{self, EditorPage};
use tirra::gui::pages::login::{self, LoginPage};
use tirra::gui::styles::style_constants::FONT_EDITOR;
use tirra::gui::styles::style_constants::FONT_EXTERNAL_BYTES;

// Constants
const TIRRA_DB_PATH_TESTING: &str = "stuff/dbs/test.db.enc";

// String : database (plaintext) location

pub fn main() -> iced::Result {
    let mut db_to_use = String::from(TIRRA_DB_PATH_TESTING);
    // check arguments
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        db_to_use = String::from(&args[1]);
    }

    // Run ICED
    TirraIced::run(Settings::<String> {
        id: Some(String::from("win-tirra")),
        flags: db_to_use,
        fonts: vec![Cow::Borrowed(FONT_EXTERNAL_BYTES)],
        default_font: FONT_EDITOR,
        default_text_size: Pixels(25.0),
        window: window::Settings {
            icon: None,
            exit_on_close_request: false,
            platform_specific: PlatformSpecific {
                application_id: String::from("win-tirra-lnx"),
            },
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
    db_location: String,
}

#[derive(Debug, Clone)]
enum Message {
    PeriodicTick,
    CtrlS,
    CtrlP,
    Editor(editor::Message),
    Login(login::Message),
    IgnoredEvent(Event),
}

impl Application for TirraIced {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = String;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let (login_page, _login_cmd) = LoginPage::new(&flags);

        //return
        (
            Self {
                theme: highlighter::Theme::InspiredGitHub,
                editor_page: None,
                login_page: login_page,
                logged_in: false,
                db_location: flags,
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
        match message {
            Message::PeriodicTick | Message::CtrlS => {
                if self.logged_in {
                    self.editor_page
                        .as_mut()
                        .unwrap()
                        .update(editor::Message::SaveFile)
                        .map(Message::Editor)
                } else {
                    Command::none()
                }
            }
            Message::CtrlP => {
                if self.logged_in {
                    self.editor_page
                        .as_mut()
                        .unwrap()
                        .update(editor::Message::ShowCommandLine)
                        .map(Message::Editor)
                } else {
                    Command::none()
                }
            }
            Message::Editor(msg) => {
                if self.logged_in {
                    self.editor_page
                        .as_mut()
                        .unwrap()
                        .update(msg)
                        .map(Message::Editor)
                } else {
                    Command::none()
                }
            }
            Message::Login(loginmsg) => {
                //self.login_page.update(loginmsg).map(Message::Login)
                match self.login_page.update(loginmsg) {
                    Some(login_msg) => {
                        // Login is successful
                        match login_msg {
                            login::Message::LoginSuccess => {
                                self.logged_in = true;
                                let (editor_page, command) = EditorPage::new(
                                    &self.db_location,
                                    self.login_page.password.as_bytes(),
                                );
                                self.editor_page = Some(editor_page);
                                command.map(Message::Editor)
                            }
                            _ => Command::none(),
                        }
                    }
                    _ => Command::none(),
                }
            }
            Message::IgnoredEvent(event) => match event {
                Event::Window(_id, win_ev) => match win_ev {
                    window::Event::Focused => {
                        if self.logged_in == false {
                            text_input::focus(text_input::Id::new(
                                login::LoginPage::text_input_id_to_focus(),
                            ))
                        } else {
                            Command::none()
                        }
                    }
                    window::Event::CloseRequested => {
                        if self.logged_in {
                            let _ = self
                                .editor_page
                                .as_mut()
                                .unwrap()
                                .update(editor::Message::SaveFile)
                                .map(Message::Editor);
                        }
                        window::close(window::Id::MAIN)
                    }
                    _ => Command::none(),
                },
                _ => Command::none(),
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let kb_event = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::CtrlS),
            keyboard::Key::Character("p") if modifiers.command() => Some(Message::CtrlP),
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
