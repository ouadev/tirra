use crate::gui::styles::{self, style_conf};
use crate::storage::db::{self, TirraEntry};
use crate::storage::tirracrypto::TirraCrypto;
use iced::theme::Theme;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, text, text_editor, Button, Space,
    TextInput,
};
use iced::Background;
use iced::Task;
use iced::{Element, Length};

use chrono::Datelike;
use chrono::Timelike;
use chrono::{DateTime, Utc};

//use unicode_segmentation::UnicodeSegmentation;

pub struct EditorPage {
    pub content: text_editor::Content,
    pub is_dirty: bool,
    pub entries: Vec<TirraEntry>,
    pub curr_entry_id: u32,
    cmd_line_show: bool,
    cmd_line_text: String,
    load_request: String,
    pub crypto: TirraCrypto,
}

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    SaveFile,
    ShowCommandLine,
    EntryButtonClicked(u32),
    NewEntryButtonClicked,
    CommandLineSubmited,
    CommandLineInputChanged(String),
}
impl EditorPage {
    pub fn new(db_location: &str, crypto_pwd: &[u8]) -> (Self, Task<Message>) {
        //Init Crypto
        let tirra_crypto = TirraCrypto::new(db_location, crypto_pwd);
        // intialize the backend
        if tirra_crypto.enc_db_found() == false {
            panic!("we are not supposed to be here without an encrypted database");
        }

        // Load all entries into memory and display the first one
        let def_req = db::tirra_db_default_read_req();
        let all_entries = db::tirra_db_get_all_entries(&tirra_crypto, &def_req)
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
                cmd_line_show: false,
                cmd_line_text: def_req.clone(),
                load_request: def_req,
                crypto: tirra_crypto,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Task::none()
            }

            Message::SaveFile => {
                if self.is_dirty {
                    self.save();
                    self.entries = self.reload_all().unwrap();
                    self.is_dirty = false;
                }
                Task::none()
            }
            Message::EntryButtonClicked(entry_id) => {
                // Save first
                if self.is_dirty {
                    self.save();
                    self.entries = self.reload_all().unwrap();
                    self.is_dirty = false;
                }
                //
                self.show_entry(entry_id);

                Task::none()
            }
            Message::NewEntryButtonClicked => {
                // Save before creating a new entry
                if self.is_dirty {
                    self.save();
                    self.is_dirty = false;
                }
                db::tirra_db_add_entry(db::TIRRA_ENTRY_TYPE_GENERAL, "", &self.crypto).unwrap();
                self.entries = self.reload_all().unwrap();
                self.show_entry(self.entry_greatest_id());
                Task::none()
            }

            Message::CommandLineInputChanged(s) => {
                self.cmd_line_text = s;
                Task::none()
            }

            Message::CommandLineSubmited => {
                if self.is_dirty {
                    self.save();
                    self.is_dirty = false;
                }
                // check if the request would work !
                let entries_opt =
                    db::tirra_db_get_all_entries(&self.crypto, &self.cmd_line_text).ok();
                match entries_opt {
                    Some(entries) => {
                        self.entries = entries;
                        self.show_entry(self.entry_greatest_id());
                        self.load_request = self.cmd_line_text.clone();
                    }
                    _ => {
                        println!("New Loader request failed !!!");
                        self.cmd_line_text = self.load_request.clone();
                    }
                }

                Task::none()
            }

            Message::ShowCommandLine => {
                self.cmd_line_show = !self.cmd_line_show;
                Task::none()
            }
        }
    }
    pub fn view(&self) -> Element<Message> {
        // DIV : Editor Text Zone + Command bar
        let div_editor = self.view_editor();
        // DIV : Separator
        let div_sep = container("")
            .width(20)
            .height(Length::Fill)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default().background(Background::Color(palette.background_main))
            });
        // DIV : Left Pan
        let div_leftpan = self.view_left_pan();
        // All
        row![div_leftpan, div_sep, div_editor].into()
    }

    /**
     * VIEW : Left Pan
     */
    fn view_left_pan(&self) -> Element<Message> {
        // DIV : ADD Button
        let mut div_add =
            Button::new(text(format!(" + New paper ")).size(style_conf::STYLE_TEXT_SIZE_NORMAL))
                .width(Length::Fill)
                .style(styles::button::button_main);

        if self.curr_entry_id > 0 {
            div_add = div_add.on_press(Message::NewEntryButtonClicked);
        }

        //DIV : list of entries
        let div_entries = column(
            self.entries.iter().map(|ent| {
                let title = EditorPage::entry_title(ent, 30);
                let link_text = text(title)
                    .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
                    .shaping(text::Shaping::Advanced);
                let ent_button = Button::new(link_text)
                    .width(Length::Fill)
                    .style(if ent.id == self.curr_entry_id {
                        styles::button::button_entry_selected
                    } else {
                        styles::button::button_entry
                    })
                    .clip(true)
                    .on_press(Message::EntryButtonClicked(ent.id));

                //let ent_separator: Rule = horizontal_rule(1);
                column![ent_button].into()
            }), //map
        ); //Column

        let div_entries_scroll = scrollable(div_entries);

        // DIV : Left Pan
        container(column![div_add, div_entries_scroll])
            .width(250)
            .height(Length::Fill)
            .style(|_theme: &Theme| {
                //let palette = theme.extended_palette();
                let palette = style_conf::palette();

                container::Style::default().background(palette.background_secondary)
            })
            .into()
    }

    /**
     * VIEW : editor status zone
     */
    fn view_editor_status(&self) -> Element<Message> {
        // Closure : generate datetime formatting
        let dt_format = |dt: DateTime<Utc>| {
            let year_month_day = format!(
                "{} {} {}",
                dt.date_naive().year_ce().1,
                EditorPage::month_abr(dt.month()),
                dt.date_naive().day(),
            );

            let weekday_time = format!(
                "{}.{:02}:{:02}",
                dt.date_naive().weekday(),
                dt.time().hour(),
                dt.time().minute(),
            );

            let text_ymd = text(year_month_day)
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS_HIGHLIGHT)
                .font(style_conf::FONT_STATUS_DATE_BOLD)
                .style(|_theme: &Theme| {
                    //let palette = theme.extended_palette();
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });

            let text_space = text("  ")
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                .font(style_conf::FONT_STATUS_DATE)
                .style(|_theme: &Theme| {
                    //let palette = theme.extended_palette();
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });

            let text_wdm = text(weekday_time)
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                .font(style_conf::FONT_STATUS_DATE)
                .style(|_theme: &Theme| {
                    //let palette = theme.extended_palette();
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });

            row![text_ymd, text_space, text_wdm].height(Length::Shrink)
        };

        // dates
        let div_date_create;
        let div_date_modify;
        if self.curr_entry_id > 0 {
            let date_create_ts = self.entry_by_id(self.curr_entry_id).unwrap().date_create;
            let date_modify_ts = self.entry_by_id(self.curr_entry_id).unwrap().date_modify;
            let dt_create = EditorPage::datetime_from_unix(date_create_ts.try_into().unwrap());
            let dt_modify = EditorPage::datetime_from_unix(date_modify_ts.try_into().unwrap());

            div_date_create = dt_format(dt_create);
            div_date_modify = dt_format(dt_modify);
        } else {
            div_date_create = row![];
            div_date_modify = row![];
        }

        // id
        let div_id = text(format!(
            "{}",
            &self.entry_by_id(self.curr_entry_id).unwrap().id
        ))
        .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
        .style(|_theme: &Theme| {
            //let palette = theme.extended_palette();
            let palette = style_conf::palette();
            text::Style {
                color: Some(palette.text),
            }
        });

        // status bar
        container(row![
            Space::with_width(20),
            div_date_create,
            text("   -   ")
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                .style(|_theme: &Theme| {
                    //let palette = theme.extended_palette();
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                }),
            div_date_modify,
            horizontal_space(),
            div_id
        ])
        .style(|_theme: &Theme| {
            let palette = style_conf::palette();
            container::Style::default().background(Background::Color(palette.background_main))
        })
        .into()
    }

    /**
     * VIEW : editor
     */
    fn view_editor(&self) -> Element<Message> {
        // DIV : Command line experimentation
        let div_cmd_input =
            TextInput::new("> SELECT * FROM entries WHERE ...", &self.cmd_line_text)
                .width(Length::Fill)
                .size(style_conf::STYLE_TEXT_SIZE_COMMAND)
                .font(style_conf::FONT_COMMAND_LINE)
                .on_submit(Message::CommandLineSubmited)
                .on_input(Message::CommandLineInputChanged);

        let mut div_command_cont;
        if self.cmd_line_show {
            div_command_cont = container(div_cmd_input).height(40);
        } else {
            div_command_cont = container("").height(10);
        }

        div_command_cont = div_command_cont
            .width(Length::Fill)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default().background(Background::Color(palette.background_main))
            });

        // DIV : Editor Text Zone
        let mut div_editor_text = text_editor(&self.content)
            .height(Length::Fill)
            .padding(20)
            .font(style_conf::FONT_EDITOR)
            .style(styles::text_editor::main_style);
        if self.curr_entry_id > 0 {
            // let the editor disabled if there is no current entry.
            div_editor_text = div_editor_text.on_action(Message::ActionPerformed);
        }

        // DIV : Editor Status Zone
        let div_editor_status = self.view_editor_status();

        //Editor
        column![div_command_cont, div_editor_text, div_editor_status].into()
    }

    fn save(&mut self) -> () {
        db::tirra_db_update_entry(&self.content.text(), self.curr_entry_id, &self.crypto)
            .expect("Tirra+Error: failed to save current file");
    }

    fn reload_all(&mut self) -> Option<Vec<TirraEntry>> {
        // Load all entries into memory:
        // note: Error is discarded here
        db::tirra_db_get_all_entries(&self.crypto, &self.load_request).ok()
    }

    /**
     * convert a timestamp into a datatime structure
     */
    fn datetime_from_unix(unix_ts: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(unix_ts, 0).expect("invalid timestamp")
    }
    /**
     * generate an abreviated string for the name of a month
     */
    fn month_abr(month: u32) -> &'static str {
        match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "-",
        }
    }

    /**
     * Show an entry defined by ID in the editor
     */
    fn show_entry(&mut self, id: u32) {
        self.curr_entry_id = id;
        if id > 0 {
            self.content = text_editor::Content::with_text(&self.entry_by_id(id).unwrap().text);
        } else {
            self.content = text_editor::Content::with_text(" Nothing was found");
        }
    }

    pub fn title(&self) -> String {
        format!("Tirra{} ", if self.is_dirty { "*" } else { "" })
    }

    fn entry_by_id(&self, id: u32) -> Option<&TirraEntry> {
        self.entries.iter().find(|ent| ent.id == id)
    }

    fn entry_greatest_id(&self) -> u32 {
        let mut id = 0u32;
        for entry in self.entries.iter() {
            if entry.id > id {
                id = entry.id;
            }
        }
        id
    }

    /*
        fn calc_title(entry: &TirraEntry, max_chars: usize) -> &str {
            let mut last_index: usize = 0;
            let mut first_index: usize = 0;
            let mut first_found = false;
            let mut collected: usize = 0;
            let text_str = entry.text.as_str();
            let graphems = UnicodeSegmentation::grapheme_indices(text_str, true);
            //

            for (i, gr_ind) in graphems.enumerate() {
                if !first_found && gr_ind.1 != " " && gr_ind.1 != "\n" {
                    first_found = true;
                    first_index = i;
                }

                if first_found && (collected == max_chars || gr_ind.1 == "\n") {
                    break;
                }

                if first_found {
                    collected += 1;
                }

                last_index = gr_ind.0;
            }

            if collected != 0 {
                &entry.text[first_index..last_index]
            } else {
                "..."
            }
            //
        }
    */
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

        if collected != 0 {
            while entry.text.is_char_boundary(last_index + 1) == false {
                last_index += 1;
            }
            &entry.text[first_index..last_index + 1]
        } else {
            "..."
        }
    }
}
