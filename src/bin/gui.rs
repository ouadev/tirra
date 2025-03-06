#![windows_subsystem = "windows"]
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
use tirra::common::exception::exception;
use tirra::gui::pages::editor::{self, EditorPage};
use tirra::gui::pages::login::{self, LoginPage};
use tirra::gui::styles::style_conf;
use tirra::ui::ui::{KbCtrl, TirraInterface};

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

enum RunningPage {
    Login(LoginPage),
    Editor(EditorPage),
}
struct TirraIced {
    theme: highlighter::Theme,
    page: RunningPage,
    db_location: String,
    last_act: i64,
    ticks: u64,
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
                page: RunningPage::Login(login_page),
                db_location: db_to_use,
                last_act: current_timestamp(),
                ticks: 0,
            },
            //command.map(Message::Editor),
            Task::none(),
        )
    }

    fn title(&self) -> String {
        match &self.page {
            RunningPage::Editor(page) => page.title(),
            RunningPage::Login(page) => page.title(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        //process message
        match message {
            Message::PeriodicTick => {
                self.ticks += 1;
                match &self.page {
                    RunningPage::Editor(_page) => {
                        // inactivity
                        if current_timestamp() - self.last_act > TIRRA_INACTIVITY_SECONDS {
                            println!("Inactivity: logging out");
                            let (login_page, _login_cmd) = LoginPage::new(&self.db_location);
                            self.page = RunningPage::Login(login_page);
                            // TODO: make sure the Editor and its content are destroyed !!
                        }
                        // trigger a file save
                        self.update_editor_page(editor::Message::Tick)
                    }
                    RunningPage::Login(login_page) => {
                        login_page.login_ui.on_tick(self.ticks);
                        Task::none()
                    }
                }
            }
            Message::CtrlS => match &mut self.page {
                RunningPage::Editor(editor_page) => {
                    editor_page.editor_ui.on_ctrl(KbCtrl::CtrlS);
                    Task::none()
                }

                RunningPage::Login(login_page) => {
                    login_page.login_ui.on_ctrl(KbCtrl::CtrlS);
                    Task::none()
                }
            },
            Message::CtrlP => match &mut self.page {
                RunningPage::Editor(_editor) => {
                    self.update_editor_page(editor::Message::ShowCommandLine)
                }

                RunningPage::Login(login_page) => {
                    login_page.login_ui.on_ctrl(KbCtrl::CtrlP);
                    Task::none()
                }
            },

            Message::CtrlN => match &mut self.page {
                RunningPage::Editor(editor_page) => {
                    editor_page.editor_ui.on_ctrl(KbCtrl::CtrlN);
                    Task::none()
                }

                RunningPage::Login(login_page) => {
                    login_page.login_ui.on_ctrl(KbCtrl::CtrlN);
                    Task::none()
                }
            },

            Message::CtrlK => match &mut self.page {
                RunningPage::Editor(editor_page) => {
                    editor_page.editor_ui.on_ctrl(KbCtrl::CtrlK);
                    Task::none()
                }

                RunningPage::Login(login_page) => {
                    login_page.login_ui.on_ctrl(KbCtrl::CtrlK);
                    Task::none()
                }
            },

            Message::Editor(msg) => {
                if let RunningPage::Editor(_editor) = &self.page {
                    self.update_editor_page(msg)
                } else {
                    Task::none()
                }
            }
            Message::Login(loginmsg) => {
                let _ = self.update_login_page(loginmsg);

                if let RunningPage::Login(login_page) = &self.page {
                    if login_page.login_ui.is_logged_in() {
                        // Login is successful
                        let (editor_page, command) = EditorPage::new(
                            &self.db_location,
                            login_page.login_ui.password.as_bytes(),
                        );
                        self.page = RunningPage::Editor(editor_page);
                        command.map(Message::Editor)
                    } else {
                        Task::none()
                    }
                } else {
                    exception("running page should be login");
                    Task::none()
                }
            }
            Message::IgnoredEvent(event) => match event {
                Event::Window(win_ev) => match win_ev {
                    window::Event::Focused => {
                        if let RunningPage::Login(_login_page) = &self.page {
                            text_input::focus(text_input::Id::new(
                                login::LoginPage::text_input_id_to_focus(),
                            ))
                        } else {
                            Task::none()
                        }
                    }
                    window::Event::CloseRequested => {
                        if let RunningPage::Editor(_editor) = &self.page {
                            match &mut self.page {
                                RunningPage::Editor(editor_page) => {
                                    editor_page.editor_ui.on_close();
                                }

                                RunningPage::Login(login_page) => {
                                    login_page.login_ui.on_close();
                                }
                            }
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
        match &self.page {
            RunningPage::Login(login_page) => login_page.view().map(Message::Login),

            RunningPage::Editor(editor_page) => editor_page.view().map(Message::Editor),
        }
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn update_editor_page(&mut self, message: editor::Message) -> Task<Message> {
        if let RunningPage::Editor(editor_page) = &mut self.page {
            editor_page.update(message).map(Message::Editor)
        } else {
            exception("running page should ");
            Task::none()
        }
    }

    fn update_login_page(&mut self, message: login::Message) -> Task<Message> {
        if let RunningPage::Login(login_page) = &mut self.page {
            login_page.update(message).map(Message::Login)
        } else {
            exception("running page should ");
            Task::none()
        }
    }
}

fn current_timestamp() -> i64 {
    Utc::now().timestamp()
}

fn default_user_db_path() -> String {
    // pick up HOME environment variable.
    let home_path = match env::var(TIRRA_HOME_DIR_PATH) {
        Ok(var) => var,
        _ => {
            exception("cannot read environment variable");
            String::from("")
        }
    };
    let db_path = Path::new(home_path.as_str()).join(TIRRA_DEFAULT_DB_NAME);
    match db_path.to_str() {
        None => String::from(TIRRA_DEFAULT_DB_NAME), // create default db in the same directory as the binary file.
        Some(path) => String::from(path),
    }
}
