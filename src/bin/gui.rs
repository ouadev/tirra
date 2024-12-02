#![windows_subsystem = "windows"]
use std::collections::HashMap;
use std::env;
use std::path::Path;

use crate::widget::text_input;
use chrono::Utc;
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
const TIRRA_INACTIVITY_SECONDS: i64 = 180; // close the editor if inactivity is detected
#[cfg(target_os = "linux")]
const TIRRA_HOME_DIR_PATH: &str = "HOME";
#[cfg(target_os = "macos")]
const TIRRA_HOME_DIR_PATH: &str = "HOME";
#[cfg(target_os = "windows")]
const TIRRA_HOME_DIR_PATH: &str = "USERPROFILE";
const TIRRA_DEFAULT_DB_NAME: &str = "awal.tirra";

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
            //fonts: vec![Cow::Borrowed(style_conf::FONT_EXTERNAL_BYTES)],
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
}

struct TirraIced {
    theme: highlighter::Theme,
    login_page: LoginPage,
    editor_page: Option<EditorPage>,
    logged_in: bool,
    db_location: String,
    last_act: i64,
}

#[derive(Debug, Clone)]
enum Message {
    PeriodicTick,
    CtrlS,
    CtrlP,
    CtrlN,
    CtrlK,
    Editor(editor::Message),
    Login(login::Message),
    IgnoredEvent(Event),
    HttpsGetDone(String),
}

impl TirraIced {
    fn new() -> (Self, Task<Message>) {
        let mut db_to_use = default_user_db_path();
        // check arguments
        let args: Vec<String> = env::args().collect();
        if args.len() == 2 {
            db_to_use = String::from(&args[1]);
        }

        let (login_page, _login_cmd) = LoginPage::new(&db_to_use);

        // decide if dark_mode should be used by default.
        style_conf::dark_mode_in_paris();

        //return
        (
            Self {
                theme: highlighter::Theme::InspiredGitHub,
                editor_page: None,
                login_page: login_page,
                logged_in: false,
                db_location: db_to_use,
                last_act: current_timestamp(),
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
        //process message
        match message {
            Message::PeriodicTick | Message::CtrlS => {
                if self.logged_in {
                    if current_timestamp() - self.last_act > TIRRA_INACTIVITY_SECONDS {
                        println!("Inactivity: logging out");
                        self.logged_in = false;
                        let (login_page, _login_cmd) = LoginPage::new(&self.db_location);
                        self.login_page = login_page;
                    }
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
            Message::CtrlN => {
                style_conf::toggle_theme();
                Task::none()
            }

            Message::CtrlK => {
                return Task::perform(
                    async move {
                        // Build the client using the builder pattern
                        let client = reqwest::Client::builder().build().unwrap();
                        // Perform the actual execution of the network request
                        let res = client.get("https://httpbin.org/ip").send().await.unwrap();
                        // Parse the response body as Json in this case
                        let ip = res.json::<HashMap<String, String>>().await.unwrap();

                        match ip.get("origin") {
                            Some(val) => val.clone(),
                            None => String::from("voidip"),
                        }
                    },
                    |value: String| Message::HttpsGetDone(value),
                );
            }

            Message::HttpsGetDone(str_ip) => {
                println!("Retrieved IP = {}", str_ip);
                Task::none()
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
                _ => {
                    self.last_act = current_timestamp();
                    Task::none()
                }
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let kb_event = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::CtrlS),
            keyboard::Key::Character("p") if modifiers.command() => Some(Message::CtrlP),
            keyboard::Key::Character("n") if modifiers.command() => Some(Message::CtrlN),
            keyboard::Key::Character("k") if modifiers.command() => Some(Message::CtrlK),
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

fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

fn default_user_db_path() -> String {
    // pick up HOME environment variable.
    let home_path = env::var(TIRRA_HOME_DIR_PATH).unwrap();
    let db_path = Path::new(home_path.as_str()).join(TIRRA_DEFAULT_DB_NAME);
    format!("{}/awal.tirra", home_path);
    match db_path.to_str() {
        None => String::from(TIRRA_DEFAULT_DB_NAME), // create default db in the same directory as the binary file.
        Some(path) => String::from(path),
    }
}
