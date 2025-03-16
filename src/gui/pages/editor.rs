use crate::common::exception::exception;
use crate::common::utils;
use crate::gui::styles::{self, style_conf};
use crate::storage::sync;
use crate::ui::ui::{BgRun, TirraInterface, WriterUi};
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

pub struct EditorPage {
    pub writer_ui: WriterUi,
    pub content: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    EditorAction(text_editor::Action),
    EntryClicked(u32),
    NewEntryClicked,
    SyncStatusClicked,
    CliSubmited,
    CliChanged(String),
    TaskSyncFetchDone(sync::SyncState),
    TaskSyncPushDone(sync::SyncState),
}
impl EditorPage {
    pub fn new(db_location: &str, crypto_pwd: &[u8]) -> (Self, Task<Message>) {
        //instantiate Editor UI
        let mut writer_ui = WriterUi::new();
        writer_ui.connect(db_location, crypto_pwd);
        // initial editor content
        let init_content = match writer_ui.current_entry() {
            Some(entry) => text_editor::Content::with_text(&entry.text),
            _ => text_editor::Content::with_text("no entry is found !!"),
        };

        //return
        (
            Self {
                writer_ui: writer_ui,
                content: init_content,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EditorAction(action) => {
                match &action {
                    text_editor::Action::Edit(_edit) => {
                        // block editing when in readonly mode
                        if !self.writer_ui.is_readonly() {
                            self.content.perform(action);
                            self.writer_ui.content_changed(&self.content.text());
                        }
                    }
                    _ => self.content.perform(action),
                }

                Task::none()
            }

            Message::Tick => {
                self.writer_ui.on_tick();
                self.task_from_ui()
            }
            Message::EntryClicked(entry_id) => {
                self.writer_ui.on_entry_selected(entry_id);
                self.refresh_editor();
                Task::none()
            }
            Message::NewEntryClicked => {
                self.writer_ui.on_new_entry();
                self.refresh_editor();
                Task::none()
            }

            Message::CliChanged(s) => {
                self.writer_ui.on_cli_input(s);
                Task::none()
            }

            Message::CliSubmited => {
                self.writer_ui.on_cli_submit();
                self.refresh_editor();
                Task::none()
            }

            Message::TaskSyncFetchDone(state) => {
                self.writer_ui.on_sync_fetched(state);
                self.task_from_ui()
            }

            Message::TaskSyncPushDone(value) => {
                self.writer_ui.on_sync_pushed(value);
                Task::none()
            }

            Message::SyncStatusClicked => {
                self.writer_ui.on_sync_clicked();
                self.refresh_editor();
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

        if self.writer_ui.curr_entry_id > 0 && !self.writer_ui.is_readonly() {
            div_add = div_add.on_press(Message::NewEntryClicked);
        }

        //DIV : list of entries
        let div_entries = column(
            self.writer_ui.entries.iter().map(|ent| {
                let title = WriterUi::entry_title(ent, 30);
                let link_text = text(title)
                    .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
                    .shaping(text::Shaping::Advanced);
                let ent_button = Button::new(link_text)
                    .width(Length::Fill)
                    .style(if ent.id == self.writer_ui.curr_entry_id {
                        styles::button::button_entry_selected
                    } else {
                        styles::button::button_entry
                    })
                    .clip(true)
                    .on_press(Message::EntryClicked(ent.id));

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
                utils::month_abr(dt.month()),
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
        if self.writer_ui.curr_entry_id > 0 {
            if let Some(entry) = self.writer_ui.current_entry() {
                entry_id = entry.id;
                let date_create_ts = entry.date_create;
                let date_modify_ts = entry.date_modify;

                let dt_create = utils::datetime_from_unix(date_create_ts as i64);
                let dt_modify = utils::datetime_from_unix(date_modify_ts as i64);

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
        let div_sync = if self.writer_ui.db_ver >= 1 {
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
        let sync_text = text(format!("{} ", self.writer_ui.sync_status.1))
            .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });

        if self.writer_ui.sync_status.0 {
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
            &self.writer_ui.cmd_line_text,
        )
        .width(Length::Fill)
        .size(style_conf::STYLE_TEXT_SIZE_COMMAND)
        .font(style_conf::FONT_COMMAND_LINE)
        .on_submit(Message::CliSubmited)
        .on_input(Message::CliChanged);

        let mut div_command_cont;
        if self.writer_ui.cmd_line_show {
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
        if self.writer_ui.curr_entry_id > 0 {
            // let the editor disabled if there is no current entry.
            div_editor_text = div_editor_text.on_action(Message::EditorAction);
        }

        // DIV : Editor Status Zone
        let div_editor_status = self.view_editor_status();

        //Editor
        column![div_command_cont, div_editor_text, div_editor_status].into()
    }

    fn refresh_editor(&mut self) {
        match self.writer_ui.current_entry() {
            Some(entry) => {
                self.content = text_editor::Content::with_text(&entry.text);
            }
            _ => {
                self.content = text_editor::Content::with_text(" Nothing was found");
            }
        }
    }

    pub fn title(&self) -> String {
        self.writer_ui.title()
    }

    /**
     * convert Tirra UI BgWork object to an Iced executable Task
     */
    fn task_from_ui(&self) -> Task<Message> {
        let bg_work = self.writer_ui.background_work();
        match bg_work {
            BgRun::Nothing => Task::none(),
            BgRun::SyncUpload => {
                //TODO: editor page shouldn't bother accessing internal crypto object.
                let db_loc = self.writer_ui.crypto.get_db_location();
                Task::perform(sync::sync_upload(1, db_loc), |value: sync::SyncState| {
                    Message::TaskSyncPushDone(value)
                })
            }
            BgRun::SyncDownload => Task::perform(
                sync::sync_download(self.writer_ui.db_id),
                |value: sync::SyncState| Message::TaskSyncFetchDone(value),
            ),
        }
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
}
