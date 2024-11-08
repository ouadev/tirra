use std::borrow::Cow;
use std::env;

use crate::widget::text_input;
use iced::highlighter::{self};
use iced::theme::Theme;
use iced::time::{self, every};
use iced::window::settings::PlatformSpecific;
use iced::{event, widget, Event, Task};
use iced::{keyboard, window};
use iced::{Element, Settings, Subscription};

extern crate tirra;
use tirra::gui::pages::editor::{self, EditorPage};
use tirra::gui::pages::login::{self, LoginPage};
use tirra::gui::styles::style_conf;

// Constants
const TIRRA_DB_PATH_TESTING: &str = "stuff/dbs/test.db.enc";

// String : database (plaintext) locationPixels

pub fn main() -> iced::Result {
    #[cfg(target_os = "linux")]
    let platform_specific = PlatformSpecific {
        application_id: String::from("win-tirra-lnx"),
        override_redirect: false,
    };
    #[cfg(target_os = "macos")]
    let platform_specific = PlatformSpecific {
        title_hidden: false,
        titlebar_transparent: false,
        fullsize_content_view: false,
    };
    #[cfg(target_os = "windows")]
    let platform_specific = PlatformSpecific {
        drag_and_drop: true,
        skip_taskbar: false,
        undecorated_shadow: false,
    };

    // Run ICED
    iced::application(TirraIced::title, TirraIced::update, TirraIced::view)
        .subscription(TirraIced::subscription)
        .theme(TirraIced::theme)
        .settings(Settings {
            id: Some(String::from("win-tirra")),
            fonts: vec![Cow::Borrowed(style_conf::FONT_EXTERNAL_BYTES)],
            default_font: style_conf::FONT_DEFAULT,
            default_text_size: style_conf::STYLE_TEXT_SIZE_EDITOR,
            ..Settings::default()
        })
        .window(window::Settings {
            icon: None,
            exit_on_close_request: false,
            platform_specific: platform_specific,
            ..Default::default()
        })
        .run_with(TirraIced::new)
    /*
        TirraIced::run(Settings::<String> {
            id: Some(String::from("win-tirra")),
            flags: db_to_use,
            fonts: vec![Cow::Borrowed(style_conf::FONT_EXTERNAL_BYTES)],
            default_font: style_conf::FONT_DEFAULT,
            default_text_size: style_conf::STYLE_TEXT_SIZE_EDITOR,
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
    */
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

impl TirraIced {
    fn new() -> (Self, Task<Message>) {
        let mut db_to_use = String::from(TIRRA_DB_PATH_TESTING);
        // check arguments
        let args: Vec<String> = env::args().collect();
        if args.len() == 2 {
            db_to_use = String::from(&args[1]);
        }

        let (login_page, _login_cmd) = LoginPage::new(&db_to_use);

        //return
        (
            Self {
                theme: highlighter::Theme::InspiredGitHub,
                editor_page: None,
                login_page: login_page,
                logged_in: false,
                db_location: db_to_use,
            },
            //command.map(Message::Editor),
            Task::none(),
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

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PeriodicTick | Message::CtrlS => {
                if self.logged_in {
                    self.editor_page
                        .as_mut()
                        .unwrap()
                        .update(editor::Message::SaveFile)
                        .map(Message::Editor)
                } else {
                    Task::none()
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
                    Task::none()
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
                    Task::none()
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
                            _ => Task::none(),
                        }
                    }
                    _ => Task::none(),
                }
            }
            Message::IgnoredEvent(event) => match event {
                Event::Window(win_ev) => match win_ev {
                    window::Event::Focused => {
                        if self.logged_in == false {
                            text_input::focus(text_input::Id::new(
                                login::LoginPage::text_input_id_to_focus(),
                            ))
                        } else {
                            Task::none()
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
                        //window::close(window::Id::MAIN)
                        window::get_latest().and_then(window::close)
                    }
                    _ => Task::none(),
                },
                _ => Task::none(),
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
