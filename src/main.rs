use std::borrow::Cow;

use gui::styles::style_constants;
use iced::highlighter::{self};
use iced::theme;
use iced::theme::Theme;
use iced::time::{self, every};
use iced::widget::{column, container, horizontal_space, row, text, text_editor, Button};
use iced::{executor, Background};
use iced::{keyboard, window};
use iced::{Application, Command, Element, Length, Settings, Subscription};

use crate::db::TirraEntry;
use crate::gui::styles::button::TirraButtonStyle;
use crate::gui::styles::button::TirraButtonType;
use crate::gui::styles::style_constants::FONT_DEJAVU_SANS_MONO;
use crate::gui::styles::style_constants::FONT_DEJAVU_SANS_MONO_BYTES;
use crate::gui::styles::text_editor::EditorStyle;
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
    content: text_editor::Content,
    theme: highlighter::Theme,
    is_dirty: bool,
    entries: Vec<TirraEntry>,
    _curr_entry_id: u32,
    crypto: TirraCrypto,
}

#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    SaveFile,
    PeriodicTick,
    EntryButtonClicked(u32),
    NewEntryButtonClicked,
}
/*
#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}
    */

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
                content: init_content,
                theme: highlighter::Theme::InspiredGitHub,
                is_dirty: false,
                entries: all_entries,
                _curr_entry_id: id,
                crypto: tirra_crypto,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        format!("Tirra{} - ouadv", if self.is_dirty { "*" } else { "" })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Command::none()
            }

            Message::SaveFile | Message::PeriodicTick => {
                //save_content_to_disk(&self.content.text());
                if self.is_dirty {
                    db::tirra_db_update_entry(
                        TIRRA_DB_PATH,
                        &self.content.text(),
                        self._curr_entry_id,
                        &self.crypto,
                    )
                    .unwrap();
                    self.is_dirty = false;

                    // Load all entries into memory and display the first one
                    self.entries = db::tirra_db_get_all_entries(TIRRA_DB_PATH, &self.crypto)
                        .expect("Error loading entries from database");
                }
                Command::none()
            }
            Message::EntryButtonClicked(entry_id) => {
                println!("entry selected : {}", entry_id);
                // Save first
                if self.is_dirty {
                    db::tirra_db_update_entry(
                        TIRRA_DB_PATH,
                        &self.content.text(),
                        self._curr_entry_id,
                        &self.crypto,
                    )
                    .unwrap();
                    self.is_dirty = false;
                    // Load all entries into memory and display the first one
                    self.entries = db::tirra_db_get_all_entries(TIRRA_DB_PATH, &self.crypto)
                        .expect("Error loading entries from database");
                }
                //
                self._curr_entry_id = entry_id;
                self.content = text_editor::Content::with_text(
                    &entry_ref_by_id(&self.entries, entry_id).unwrap().text,
                );

                Command::none()
            }
            Message::NewEntryButtonClicked => {
                println!("New paper will be created");
                // Save before creating a new entry
                if self.is_dirty {
                    db::tirra_db_update_entry(
                        TIRRA_DB_PATH,
                        &self.content.text(),
                        self._curr_entry_id,
                        &self.crypto,
                    )
                    .unwrap();
                    self.is_dirty = false;
                }
                db::tirra_db_add_entry(TIRRA_DB_PATH, "Pour your soul here >", &self.crypto)
                    .unwrap();
                // Load all entries into memory and display the first one
                self.entries = db::tirra_db_get_all_entries(TIRRA_DB_PATH, &self.crypto)
                    .expect("Error loading entries from database");
                Command::none()
            }
        }
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
        // DIV : Editor Text Zone
        let div_editor_text = text_editor(&self.content)
            .height(Length::Fill)
            .style(theme::TextEditor::Custom(Box::new(EditorStyle {})))
            .on_action(Message::ActionPerformed);

        // DIV : Editor Status Zone
        let div_editor_status = container(row![
            text(format!(
                "{}",
                entry_ref_by_id(&self.entries, self._curr_entry_id)
                    .unwrap()
                    .date
            )),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ])
        .style(|_theme: &Theme| {
            container::Appearance::default()
                .with_background(Background::Color(style_constants::STYLE_EDITOR_BG_COLOR))
        });

        //Editor
        let div_editor = column![div_editor_text, div_editor_status];

        // DIV : ADD Button
        let div_add = Button::new(" + New paper ")
            .width(Length::Fill)
            .style(theme::Button::custom(TirraButtonStyle {
                button_type: TirraButtonType::EntryAdd,
                selected: false,
            }))
            .on_press(Message::NewEntryButtonClicked);

        //DIV : list of entries
        let div_entries = column(
            self.entries.iter().map(|ent| {
                let title = entry_title(ent, 20);
                let ent_button = Button::new(text(format!("{}", title)))
                    .width(Length::Fill)
                    .style(theme::Button::custom(TirraButtonStyle {
                        button_type: TirraButtonType::EntryOpen,
                        selected: (ent.id == self._curr_entry_id),
                    }))
                    .clip(true)
                    .on_press(Message::EntryButtonClicked(ent.id));

                ent_button.into()
            }), //map
        ); //Column

        //let div_sep: Rule = Rule::vertical(50);
        let div_sep = container("")
            .width(20)
            .height(Length::Fill)
            .style(|_theme: &Theme| {
                container::Appearance::default()
                    .with_background(Background::Color(style_constants::STYLE_EDITOR_BG_COLOR))
            });
        // DIV : Left Pan
        let div_leftpan = container(column![div_add, div_entries])
            .width(250)
            .height(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Appearance::default().with_background(palette.background.strong.color)
            });

        // BODY
        let body = row![div_leftpan, div_sep, div_editor];
        body.into()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

fn entry_ref_by_id(entries: &Vec<TirraEntry>, id: u32) -> Option<&TirraEntry> {
    entries.iter().find(|ent| ent.id == id)
}

fn entry_title(entry: &TirraEntry, max_chars: u8) -> &str {
    let mut last_index: usize = 0;
    let mut first_index: usize = 0;
    let mut first_found = false;
    let mut collected = 0u8;
    for (i, c) in entry.text.chars().enumerate() {
        if !first_found && c != ' ' && c != '\n' {
            first_found = true;
            first_index = i;
        }

        if first_found && (collected == max_chars || c == '\n') {
            break;
        }

        if first_found {
            collected += 1;
        }

        last_index = i;
    }

    //println!("last_index = {}", last_index);
    if collected != 0 {
        &entry.text[first_index..last_index + 1]
    } else {
        &entry.text[0..0]
    }
}

/*
fn action<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'a str,
    on_press: Option<Message>,
) -> Element<'a, Message> {
    let action = button(container(content).width(30).center_x());

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(theme::Container::Box)
        .into()
    } else {
        action.style(theme::Button::Secondary).into()
    }
}
*/

/*
fn save_content_to_disk (cont: &str) -> () {
    //let mut secret_file = File::create("/tmp/notes.txt").expect("creation failed");
    println!("saving....");

    fs::write("/tmp/editor.txt",cont).expect("");
}
*/
