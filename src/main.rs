use std::borrow::Cow;

use iced::executor;
use iced::highlighter::{self};
use iced::theme::Theme;
use iced::time::{self, every};
use iced::{keyboard, window};
use iced::{Application, Command, Element, Settings, Subscription};

use crate::gui::pages::editor::{self, EditorPage};
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
    editor_page: EditorPage,
}

#[derive(Debug, Clone)]
enum Message {
    PeriodicTick,
    CtrlS,
    Editor(editor::Message),
}

impl Application for TirraIced {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let (editor_page, command) = EditorPage::new(TIRRA_DB_PATH);

        //return
        (
            Self {
                theme: highlighter::Theme::InspiredGitHub,
                editor_page: editor_page,
            },
            command.map(Message::Editor),
        )
    }

    fn title(&self) -> String {
        format!("Tirra{} ", if self.editor_page.is_dirty { "*" } else { "" })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let editor_msg: editor::Message;

        match message {
            Message::PeriodicTick | Message::CtrlS => {
                editor_msg = editor::Message::SaveFile;
            }
            Message::Editor(msg) => {
                editor_msg = msg;
            }
        }
        self.editor_page.update(editor_msg).map(Message::Editor)
    }

    fn subscription(&self) -> Subscription<Message> {
        let kb_event = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::CtrlS),
            _ => None,
        });

        // Configure periodical save tick
        let tick_event = every(time::Duration::new(5, 0)).map(|_| Message::PeriodicTick);

        Subscription::batch(vec![tick_event, kb_event])
    }

    fn view(&self) -> Element<Message> {
        self.editor_page.view().map(Message::Editor)
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}
