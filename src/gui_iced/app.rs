
//use crate::widget::text_input;
use iced::time::{self, every};
use iced::widget::Id;
use iced::widget::operation;
use iced::window::settings::PlatformSpecific;
use iced::{event, Event, Size, Task};
use iced::{keyboard, window};
use iced::{Element, Settings, Subscription};

use crate::common::exception::exception;
use crate::gui_iced::pages::editor::{self, EditorPage};
use crate::gui_iced::pages::login::{self, LoginPage};
use crate::gui_iced::styles::style_conf;
use crate::ui::ui::{KbCtrl, TirraInterface};


static DEFAULT_TEMP_DB_PATH: &str = "temporary_db";

pub fn app_iced(db_path: String) -> iced::Result {
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
    iced::application(TirraIced::new_temp, TirraIced::update, TirraIced::view)
        .subscription(TirraIced::subscription)
        //.theme(TirraIced::theme)
        .title(TirraIced::title)
        .settings(Settings {
            id: Some(String::from("win-tirra")),
            default_font: style_conf::FONT_DEFAULT,
            default_text_size: style_conf::STYLE_TEXT_SIZE_EDITOR,
            ..Settings::default()
        })
        .window(window::Settings {
            icon: None,
            size: Size {
                width: 1000.,
                height: 800.,
            },
            min_size: Some(Size {
                width: 800.,
                height: 600.,
            }),
            exit_on_close_request: false,
            platform_specific: platform_specific,
            ..Default::default()
        })
        .run()
}

enum RunningPage {
    Login(LoginPage),
    Editor(EditorPage),
}
struct TirraIced {
    page: RunningPage,
    db_location: String,
}

#[derive(Debug, Clone)]
enum Message {
    PeriodicTick,
    CtrlPlusKey(KbCtrl),
    Editor(editor::Message),
    Login(login::Message),
    IgnoredEvent(Event),
}

impl TirraIced {
    fn new(db_to_use: String) -> (Self, Task<Message>) {
        let (login_page, _) = LoginPage::new(&db_to_use);

        // decide if dark_mode should be used by default.
        style_conf::dark_mode_in_paris();

        //return
        (
            Self {
                page: RunningPage::Login(login_page),
                db_location: db_to_use,
            },
            //command.map(Message::Editor),
            Task::none(),
        )
    }

    fn new_temp() -> (Self, Task<Message>) {
        let (login_page, _) = LoginPage::new(DEFAULT_TEMP_DB_PATH);

        // decide if dark_mode should be used by default.
        style_conf::dark_mode_in_paris();

        //return
        (
            Self {
                page: RunningPage::Login(login_page),
                db_location: DEFAULT_TEMP_DB_PATH.to_string(),
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
                match &mut self.page {
                    RunningPage::Editor(page_editor) => {
                        // inactivity
                        if page_editor.writer_ui.is_inactivity() {
                            println!("Inactivity: logging out");
                            let (login_page, _login_cmd) = LoginPage::new(&self.db_location);
                            self.page = RunningPage::Login(login_page);
                            // TODO: make sure the Editor and its content are destroyed !!
                            Task::none()
                        } else {
                            // trigger a file save
                            self.update_editor_page(editor::Message::Tick)
                        }
                    }
                    RunningPage::Login(login_page) => {
                        login_page.login_ui.on_tick();
                        Task::none()
                    }
                }
            }
            Message::CtrlPlusKey(key) => match &mut self.page {
                RunningPage::Editor(editor_page) => {
                    editor_page.writer_ui.on_ctrl(key);
                    Task::none()
                }

                RunningPage::Login(login_page) => {
                    login_page.login_ui.on_ctrl(key);
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
                    exception("running page should be login", None);
                    Task::none()
                }
            }
            Message::IgnoredEvent(event) => match event {
                Event::Window(win_ev) => match win_ev {
                    window::Event::Focused => {
                        if let RunningPage::Login(_login_page) = &self.page {
                            operation::focus(Id::new(
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
                                    editor_page.writer_ui.on_close();
                                }

                                RunningPage::Login(login_page) => {
                                    login_page.login_ui.on_close();
                                }
                            }
                        }
                        //window::close(window::Id::MAIN)
                        window::latest().and_then(window::close)
                    }
                    _ => Task::none(),
                },
                _ => match &mut self.page {
                    RunningPage::Editor(editor_page) => {
                        editor_page.writer_ui.on_activity();
                        Task::none()
                    }

                    RunningPage::Login(login_page) => {
                        login_page.login_ui.on_activity();
                        Task::none()
                    }
                },
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let kb_event = keyboard::listen().filter_map(|event| {
            if let keyboard::Event::KeyPressed { key,  modifiers, .. } = event {
            
                    match key.as_ref() {
                        keyboard::Key::Character("s") if modifiers.command() => {
                            Some(Message::CtrlPlusKey(KbCtrl::CtrlS))
                        }
                        keyboard::Key::Character("p") if modifiers.command() => {
                            Some(Message::CtrlPlusKey(KbCtrl::CtrlP))
                        }
                        keyboard::Key::Character("n") if modifiers.command() => {
                            Some(Message::CtrlPlusKey(KbCtrl::CtrlN))
                        }
                        keyboard::Key::Character("k") if modifiers.command() => {
                            Some(Message::CtrlPlusKey(KbCtrl::CtrlK))
                        }
                        _ => None,
                    }

            }else {
                None
            }
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


    fn update_editor_page(&mut self, message: editor::Message) -> Task<Message> {
        if let RunningPage::Editor(editor_page) = &mut self.page {
            editor_page.update(message).map(Message::Editor)
        } else {
            exception("running page should editor", None);
            Task::none()
        }
    }

    fn update_login_page(&mut self, message: login::Message) -> Task<Message> {
        if let RunningPage::Login(login_page) = &mut self.page {
            login_page.update(message).map(Message::Login)
        } else {
            exception("running page should login ", None);
            Task::none()
        }
    }
}
