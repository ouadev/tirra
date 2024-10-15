use iced::theme::Theme;
use iced::widget::{column, container, horizontal_space, row, text, text_editor, Button};
use iced::Background;
use iced::{theme, Command};
use iced::{Element, Length};

use crate::db::{self, TirraEntry};
use crate::gui::styles::button::TirraButtonStyle;
use crate::gui::styles::button::TirraButtonType;
use crate::gui::styles::style_constants;
use crate::gui::styles::text_editor::EditorStyle;
use crate::tirracrypto::TirraCrypto;

pub struct EditorPage {
    pub content: text_editor::Content,
    pub is_dirty: bool,
    pub entries: Vec<TirraEntry>,
    pub curr_entry_id: u32,
    pub db_location: String,
    pub crypto: TirraCrypto,
}

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    SaveFile,
    EntryButtonClicked(u32),
    NewEntryButtonClicked,
}
impl EditorPage {
    pub fn new(db_location: &str, crypto_pwd: &[u8]) -> (Self, Command<Message>) {
        //Init Crypto
        let tirra_crypto = TirraCrypto::new(db_location, crypto_pwd);
        // intialize the backend
        if tirra_crypto.enc_db_found() == false {
            println!("enc db not found");
            // create new db
            db::tirra_db_init(db_location, &tirra_crypto).expect("database init error");
            // insert first empty entry
            db::tirra_db_add_entry(db_location, "Welcome ...", &tirra_crypto)
                .expect("first entry add failed");
        }

        // Load all entries into memory and display the first one
        let all_entries = db::tirra_db_get_all_entries(db_location, &tirra_crypto)
            .expect("Error loading entries from database");
        let init_content = text_editor::Content::with_text(&all_entries[0].text);
        let id = all_entries[0].id;

        //return
        (
            Self {
                curr_entry_id: id,
                content: init_content,
                is_dirty: false,
                entries: all_entries,
                db_location: String::from(db_location),
                crypto: tirra_crypto,
            },
            Command::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Command::none()
            }

            Message::SaveFile => {
                if self.is_dirty {
                    db::tirra_db_update_entry(
                        &self.db_location,
                        &self.content.text(),
                        self.curr_entry_id,
                        &self.crypto,
                    )
                    .unwrap();
                    self.is_dirty = false;

                    // Load all entries into memory and display the first one
                    self.entries = db::tirra_db_get_all_entries(&self.db_location, &self.crypto)
                        .expect("Error loading entries from database");
                }
                Command::none()
            }
            Message::EntryButtonClicked(entry_id) => {
                // Save first
                if self.is_dirty {
                    db::tirra_db_update_entry(
                        &self.db_location,
                        &self.content.text(),
                        self.curr_entry_id,
                        &self.crypto,
                    )
                    .unwrap();
                    self.is_dirty = false;
                    // Load all entries into memory and display the first one
                    self.entries = db::tirra_db_get_all_entries(&self.db_location, &self.crypto)
                        .expect("Error loading entries from database");
                }
                //
                self.curr_entry_id = entry_id;
                self.content = text_editor::Content::with_text(
                    &self.entry_by_id(self.curr_entry_id).unwrap().text,
                );

                Command::none()
            }
            Message::NewEntryButtonClicked => {
                println!("New paper will be created");
                // Save before creating a new entry
                if self.is_dirty {
                    db::tirra_db_update_entry(
                        &self.db_location,
                        &self.content.text(),
                        self.curr_entry_id,
                        &self.crypto,
                    )
                    .unwrap();
                    self.is_dirty = false;
                }
                db::tirra_db_add_entry(&self.db_location, "Pour your soul here >", &self.crypto)
                    .unwrap();
                // Load all entries into memory and display the first one
                self.entries = db::tirra_db_get_all_entries(&self.db_location, &self.crypto)
                    .expect("Error loading entries from database");
                Command::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message> {
        // DIV : Editor Text Zone
        let div_editor_text = text_editor(&self.content)
            .height(Length::Fill)
            .style(theme::TextEditor::Custom(Box::new(EditorStyle {})))
            .on_action(Message::ActionPerformed);

        // DIV : Editor Status Zone
        let div_editor_status = container(row![
            text(format!(
                "{}",
                self.entry_by_id(self.curr_entry_id).unwrap().date
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
                let title = EditorPage::entry_title(ent, 20);
                let ent_button = Button::new(text(format!("{}", title)))
                    .width(Length::Fill)
                    .style(theme::Button::custom(TirraButtonStyle {
                        button_type: TirraButtonType::EntryOpen,
                        selected: (ent.id == self.curr_entry_id),
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

    pub fn title(&self) -> String {
        format!("Tirra{} ", if self.is_dirty { "*" } else { "" })
    }

    fn entry_by_id(&self, id: u32) -> Option<&TirraEntry> {
        self.entries.iter().find(|ent| ent.id == id)
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
}
