use std::borrow::Cow;

use iced::executor;
use iced::highlighter::{self};
use iced::theme::Theme;
use iced::time::{self, every};
use iced::widget::text_editor;
use iced::{keyboard, window};
use iced::{Application, Command, Element, Settings, Subscription};

use crate::gui::pages::editor::{EditorPage, Message};
use crate::gui::styles::style_constants::FONT_DEJAVU_SANS_MONO;
use crate::gui::styles::style_constants::FONT_DEJAVU_SANS_MONO_BYTES;
use crate::tirracrypto::TirraCrypto;

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

impl Application for TirraIced {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        //Init Crypto
        let tirra_crypto = TirraCrypto::new(TIRRA_DB_PATH);
        // intialize the backend
        if tirra_crypto.enc_db_found() == false {
            println!("enc db not found");
            // create new db
            db::tirra_db_init(TIRRA_DB_PATH, &tirra_crypto).expect("database init error");
            // insert first empty entry
            db::tirra_db_add_entry(TIRRA_DB_PATH, "Welcome ...", &tirra_crypto)
                .expect("first entry add failed");
        }

        // Load all entries into memory and display the first one
        let all_entries = db::tirra_db_get_all_entries(TIRRA_DB_PATH, &tirra_crypto)
            .expect("Error loading entries from database");
        let init_content = text_editor::Content::with_text(&all_entries[0].text);
        let id = all_entries[0].id;

        //return
        (
            Self {
                theme: highlighter::Theme::InspiredGitHub,

                editor_page: EditorPage {
                    curr_entry_id: id,
                    content: init_content,
                    is_dirty: false,
                    entries: all_entries,
                    db_location: String::from(TIRRA_DB_PATH),
                    crypto: tirra_crypto,
                },
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Tirra{} ", if self.editor_page.is_dirty { "*" } else { "" })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        self.editor_page.update(message)
    }

    fn subscription(&self) -> Subscription<Message> {
        let kb_event = keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => Some(Message::SaveFile),
            _ => None,
        });

        // Configure periodical save tick
        let tick_event = every(time::Duration::new(5, 0)).map(|_| Message::PeriodicTick);

        Subscription::batch(vec![tick_event, kb_event])
    }

    fn view(&self) -> Element<Message> {
        self.editor_page.view()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}