use crate::common::exception::exception;
use crate::gui::styles::{self, style_conf};
use crate::storage::db::{self, TirraEntry};
use crate::storage::sync::{self, SyncDecision, SyncState};
use crate::storage::tirracrypto::TirraCrypto;
use crate::ui::ui::{EditorUi, TirraInterface};
use iced::theme::Theme;
use iced::widget::{
    column, container, horizontal_space, row, scrollable, text, text_editor, Button, MouseArea,
    Space, TextInput,
};
use iced::Background;
use iced::Task;
use iced::{Element, Length};

use chrono::Datelike;
use chrono::Timelike;
use chrono::{DateTime, Utc};

//use unicode_segmentation::UnicodeSegmentation;

pub struct EditorPage {
    pub editor_ui: EditorUi,
    pub content: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    Tick,
    EntryButtonClicked(u32),
    NewEntryButtonClicked,
    SyncStatusClicked,
    CommandLineSubmited,
    CommandLineInputChanged(String),
    SyncFetchDone(sync::SyncState),
    SyncPushDone(sync::SyncState),
}
impl EditorPage {
    pub fn new(db_location: &str, crypto_pwd: &[u8]) -> (Self, Task<Message>) {
        //instantiate Editor UI
        let editor_ui = EditorUi::new(db_location, crypto_pwd);
        // initial editor content
        let init_content = text_editor::Content::with_text(&editor_ui.entries[0].text);
        let db_id = editor_ui.db_id;
        let db_ver = editor_ui.db_ver;
        //return
        (
            Self {
                editor_ui: editor_ui,
                content: init_content,
            },
            if db_ver >= 1 {
                Task::perform(sync::sync_download(db_id), |value: sync::SyncState| {
                    Message::SyncFetchDone(value)
                })
            } else {
                Task::none()
            },
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                match &action {
                    text_editor::Action::Edit(_edit) => {
                        // block editing when in readonly mode
                        if !self.editor_ui.is_readonly() {
                            self.content.perform(action);
                            self.editor_ui.content_changed(&self.content.text());
                        }
                    }
                    _ => self.content.perform(action),
                }

                Task::none()
            }

            Message::Tick => {
                self.editor_ui.on_tick();
                if self.editor_ui.sync_fetch_needed() {
                    Task::perform(
                        sync::sync_download(self.editor_ui.db_id),
                        |value: sync::SyncState| Message::SyncFetchDone(value),
                    )
                } else {
                    Task::none()
                }
            }
            Message::EntryButtonClicked(entry_id) => {
                self.editor_ui.on_entry_selected(entry_id);
                self.update_editor_content();
                Task::none()
            }
            Message::NewEntryButtonClicked => {
                self.editor_ui.on_new_entry();
                self.update_editor_content();
                Task::none()
            }

            Message::CommandLineInputChanged(s) => {
                self.editor_ui.on_cli_input(s);
                Task::none()
            }

            Message::CommandLineSubmited => {
                if self.editor_ui.is_dirty {
                    self.save();
                    self.editor_ui.is_dirty = false;
                }
                // check if the request would work !
                let entries_opt = db::tirra_db_get_all_entries(
                    &self.editor_ui.crypto,
                    &self.editor_ui.cmd_line_text,
                )
                .ok();
                match entries_opt {
                    Some(entries) => {
                        self.editor_ui.entries = entries;
                        self.show_entry(self.entry_greatest_id());
                        self.editor_ui.load_request = self.editor_ui.cmd_line_text.clone();
                    }
                    _ => {
                        println!("New Loader request failed !!!");
                        self.editor_ui.cmd_line_text = self.editor_ui.load_request.clone();
                    }
                }

                Task::none()
            }

            Message::SyncFetchDone(state) => {
                self.editor_ui.sync_state = state;
                println!("------------------");
                // debug:  print local info
                if let Ok(local_info) = db::tirra_db_information(&self.editor_ui.crypto) {
                    println!("Ours:");
                    sync::info_sync_debug(&local_info);
                } else {
                    println!("local db: couldn't retrieve info block");
                }

                // debug: print origin database info.
                if state == SyncState::Fetched {
                    if let Some(origin_info) =
                        sync::sync_retrieve_information(&self.editor_ui.crypto)
                    {
                        println!("Theirs:");
                        sync::info_sync_debug(&origin_info);
                        println!("");
                    } else {
                        println!("origin db: couldn't retrieve info block");
                    }
                }
                // compute decision and apply it.
                let decision = sync::process_after_fetch(state, &self.editor_ui.crypto);
                self.update_sync_status(&decision);

                if decision == SyncDecision::Push {
                    let db_loc = self.editor_ui.crypto.get_db_location();
                    Task::perform(sync::sync_upload(1, db_loc), |value: sync::SyncState| {
                        Message::SyncPushDone(value)
                    })
                } else if decision == SyncDecision::UpdateCommits {
                    sync::update_origin_commit(&self.editor_ui.crypto);
                    Task::none()
                } else {
                    Task::none()
                }
            }

            Message::SyncPushDone(value) => {
                self.editor_ui.sync_state = value;
                println!("sync: pushing is done");
                if value == SyncState::Pushed {
                    self.editor_ui.sync_status.1 = format!("{}", "up to date");
                    // change commits
                    sync::update_origin_commit(&self.editor_ui.crypto);
                } else {
                    self.editor_ui.sync_status.1 = format!("{}", "error pushing");
                }
                Task::none()
            }

            Message::SyncStatusClicked => {
                if self.editor_ui.is_dirty {
                    println!("editor is dirty. dropping latest changes.");
                }
                self.save();
                match db::tirra_db_replace(&self.editor_ui.crypto, &sync::origin_db_temp_file()) {
                    Ok(()) => {
                        self.editor_ui.entries =
                            Self::reload_all(&self.editor_ui.crypto, &self.editor_ui.load_request);
                        self.editor_ui.is_dirty = false;
                        self.show_entry(self.entry_greatest_id());
                        // change commits
                        sync::update_origin_commit(&self.editor_ui.crypto);
                        self.editor_ui.sync_status = (false, format!("{}", "replaced"));
                    }
                    _ => {
                        println!("error: sync failed to replace local db");
                    }
                }

                Task::none()
            }
        }
    }

    fn update_sync_status(&mut self, decision: &SyncDecision) {
        let status_text: String;
        self.editor_ui.sync_status.0 = false;
        match decision {
            SyncDecision::ReplaceLocal => {
                self.editor_ui.sync_status.0 = true;
                status_text = format!("{}", "replace");
            }
            SyncDecision::Push => {
                status_text = format!("{}", "to upload");
            }
            SyncDecision::ResolveConflict => {
                status_text = format!("{}", "conflict [O] [T]");
            }
            SyncDecision::UpdateCommits => {
                status_text = format!("{}", "up to date*");
            }
            SyncDecision::StatusQuo => {
                status_text = format!("{}", "up to date");
            }
            SyncDecision::Failure => {
                status_text = format!("{}", "failure");
            }
        }
        if status_text != self.editor_ui.sync_status.1 {
            self.editor_ui.sync_status.1 = status_text;
            println!("sync dec: {}", self.editor_ui.sync_status.1);
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

        if self.editor_ui.curr_entry_id > 0 && !self.editor_ui.is_readonly() {
            div_add = div_add.on_press(Message::NewEntryButtonClicked);
        }

        //DIV : list of entries
        let div_entries = column(
            self.editor_ui.entries.iter().map(|ent| {
                let title = EditorPage::entry_title(ent, 30);
                let link_text = text(title)
                    .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
                    .shaping(text::Shaping::Advanced);
                let ent_button = Button::new(link_text)
                    .width(Length::Fill)
                    .style(if ent.id == self.editor_ui.curr_entry_id {
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
        let entry_id;
        if self.editor_ui.curr_entry_id > 0 {
            if let Some(entry) = self.entry_by_id(self.editor_ui.curr_entry_id) {
                entry_id = entry.id;
                let date_create_ts = entry.date_create;
                let date_modify_ts = entry.date_modify;

                let dt_create = EditorPage::datetime_from_unix(date_create_ts as i64);
                let dt_modify = EditorPage::datetime_from_unix(date_modify_ts as i64);

                div_date_create = dt_format(dt_create);
                div_date_modify = dt_format(dt_modify);
            } else {
                entry_id = 0;
                div_date_create = row![];
                div_date_modify = row![];
                exception("misalignment between gui and model (curr_entry_id)");
            }
        } else {
            entry_id = 0;
            div_date_create = row![];
            div_date_modify = row![];
        }

        // id
        let div_id = text(format!("{}", entry_id))
            .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
            .style(|_theme: &Theme| {
                //let palette = theme.extended_palette();
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });

        // sync
        let div_sync = if self.editor_ui.db_ver >= 1 {
            self.view_sync_status()
        } else {
            horizontal_space().into()
        };

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
            div_sync,
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
     * View for Sync Status box
     */
    fn view_sync_status(&self) -> Element<Message> {
        let sync_button;
        // sync state
        let sync_label = text("sync : ")
            .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });
        let sync_text = text(format!("{} ", self.editor_ui.sync_status.1))
            .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });

        if self.editor_ui.sync_status.0 {
            sync_button = text("[+]")
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS_HIGHLIGHT)
                .font(style_conf::FONT_STATUS_DATE_BOLD)
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });
        } else {
            sync_button = text("");
        }

        let sync_button_mouse: MouseArea<'_, Message> =
            MouseArea::new(sync_button).on_press(Message::SyncStatusClicked);

        row![sync_label, sync_text, sync_button_mouse].into()
    }

    /**
     * VIEW : editor
     */
    fn view_editor(&self) -> Element<Message> {
        // DIV : Command line experimentation
        let div_cmd_input = TextInput::new(
            "> SELECT * FROM entries WHERE ...",
            &self.editor_ui.cmd_line_text,
        )
        .width(Length::Fill)
        .size(style_conf::STYLE_TEXT_SIZE_COMMAND)
        .font(style_conf::FONT_COMMAND_LINE)
        .on_submit(Message::CommandLineSubmited)
        .on_input(Message::CommandLineInputChanged);

        let mut div_command_cont;
        if self.editor_ui.cmd_line_show {
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
        if self.editor_ui.curr_entry_id > 0 {
            // let the editor disabled if there is no current entry.
            div_editor_text = div_editor_text.on_action(Message::ActionPerformed);
        }

        // DIV : Editor Status Zone
        let div_editor_status = self.view_editor_status();

        //Editor
        column![div_command_cont, div_editor_text, div_editor_status].into()
    }

    fn save(&mut self) -> () {
        let updated = db::tirra_db_update_entry(
            &self.content.text(),
            self.editor_ui.curr_entry_id,
            &self.editor_ui.crypto,
        );
        match updated {
            Err(err) => {
                exception(&format!("update entry {:?}", err));
            }
            _ => {}
        }
    }

    fn reload_all(crypto: &TirraCrypto, request_string: &String) -> Vec<TirraEntry> {
        // Load all entries into memory:
        match db::tirra_db_get_all_entries(&crypto, &request_string) {
            Ok(entries) => entries,
            Err(_error) => {
                exception("loading entries");
                vec![]
            }
        }
    }

    /**
     * convert a timestamp into a datatime structure
     */
    fn datetime_from_unix(unix_ts: i64) -> DateTime<Utc> {
        match DateTime::from_timestamp(unix_ts, 0) {
            Some(date) => date,
            None => {
                exception("invalid timestamp");
                DateTime::<Utc>::MIN_UTC
            }
        }
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
        self.editor_ui.curr_entry_id = id;
        if id > 0 {
            if let Some(entry) = self.entry_by_id(id) {
                self.content = text_editor::Content::with_text(&entry.text);
            } else {
                exception("entry with id is not found");
            }
        } else {
            self.content = text_editor::Content::with_text(" Nothing was found");
        }
    }

    fn update_editor_content(&mut self) {
        match self.editor_ui.current_entry() {
            Some(entry) => {
                self.content = text_editor::Content::with_text(&entry.text);
            }
            _ => {
                self.content = text_editor::Content::with_text(" Nothing was found");
            }
        }
    }

    pub fn title(&self) -> String {
        format!("Tirra{} ", if self.editor_ui.is_dirty { "*" } else { "" })
    }

    fn entry_by_id(&self, id: u32) -> Option<&TirraEntry> {
        self.editor_ui.entries.iter().find(|ent| ent.id == id)
    }

    fn entry_greatest_id(&self) -> u32 {
        let mut id = 0u32;
        for entry in self.editor_ui.entries.iter() {
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
