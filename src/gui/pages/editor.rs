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
        let mut div_add = Button::new(
            text(WriterUi::UI_WRITER_NEWENTRY_TEXT).size(style_conf::STYLE_TEXT_SIZE_NORMAL),
        )
        .width(Length::Fill)
        .style(styles::button::button_main);

        if self.writer_ui.view_button_newentry_enabled() {
            div_add = div_add.on_press(Message::NewEntryClicked);
        }

        //DIV : list of entries
        let div_entries = column(self.writer_ui.entry_view_iter().map(|entry_view| {
            // entry link text
            let link_text = text(entry_view.title)
                .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
                .shaping(text::Shaping::Advanced);
            //entry button
            let ent_button = Button::new(link_text)
                .width(Length::Fill)
                .style(if entry_view.selected {
                    styles::button::button_entry_selected
                } else {
                    styles::button::button_entry
                })
                .clip(true)
                .on_press(Message::EntryClicked(entry_view.id));

            //let ent_separator: Rule = horizontal_rule(1);
            column![ent_button].into()
        })); //Column

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
        let dt_view = |date_str: String, time_str: String| {
            let text_ymd = text(date_str)
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS_HIGHLIGHT)
                .font(style_conf::FONT_STATUS_DATE_BOLD)
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });

            let text_space = text("  ")
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                .font(style_conf::FONT_STATUS_DATE)
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });

            let text_wdm = text(time_str)
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                .font(style_conf::FONT_STATUS_DATE)
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                });

            row![text_ymd, text_space, text_wdm].height(Length::Shrink)
        };

        //calculate the date_time to display
        let div_date_create;
        let div_date_modify;
        let entry_id;
        match self.writer_ui.view_status_current_entry_date() {
            Some((id, dt_create, tm_create, dt_modify, tm_modify)) => {
                entry_id = id;
                div_date_create = dt_view(dt_create, tm_create);
                div_date_modify = dt_view(tm_modify, dt_modify);
            }
            None => {
                entry_id = 0;
                div_date_create = row![];
                div_date_modify = row![];
            }
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
        let div_sync = if self.writer_ui.view_sync_status_visible() {
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
            WriterUi::UI_WRITER_CLI_PLACEHOLDER,
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
        if self.writer_ui.is_entry_selected() {
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
                let db_loc = self.writer_ui.get_db_location();
                Task::perform(sync::sync_upload(1, db_loc), |value: sync::SyncState| {
                    Message::TaskSyncPushDone(value)
                })
            }
            BgRun::SyncDownload(db_id) => {
                Task::perform(sync::sync_download(db_id), |value: sync::SyncState| {
                    Message::TaskSyncFetchDone(value)
                })
            }
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
