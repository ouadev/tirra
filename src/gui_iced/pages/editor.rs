use crate::gui_iced::components;
use crate::gui_iced::components::button::{button_action, button_list_entry};
use crate::gui_iced::styles::style_conf::palette;
use crate::gui_iced::styles::{self, style_conf};
use crate::ui::ui::{BgRun, TirraInterface, WriterUi};

use iced::theme::Theme;

use iced::widget::scrollable::Scrollbar;
use iced::widget::text::Wrapping;
use iced::widget::{
    self, column, container, mouse_area, row, scrollable, space, text, text_editor, Button, Id,
    Space, TextInput,
};
use iced::widget::{operation, Text};
use iced::{border, Background};
use iced::{Element, Length};
use iced::{Padding, Task};

const SEARCH_INPUT_ICED_ID: &str = "searchinput-id";
const CLI_INPUT_ICED_ID: &str = "cliinput-id";
const CREATE_DATE_EDIT_ID: &str = "create_date_input_id";
const TEXT_EDITOR_ID: &str = "text_editor_id";
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
    LiveEntryClicked,
    CliSubmited,
    CliChanged(String),
    SearchSubmited,
    SearchChanged(String),
    CreateDateSubmited,
    CreateDateChanged(String),
    SortToggled,
    CreateDateDoubleClicked,
    DeleteEntryClicked,
    ResetCliClicked,
    LoaderPagination(bool),
}
impl EditorPage {
    pub fn new(db_location: &str, crypto_pwd: &[u8]) -> (Self, Task<Message>) {
        //instantiate Editor UI
        let mut writer_ui = WriterUi::new();
        writer_ui.connect(db_location, crypto_pwd);
        // initial editor content
        let init_content = match &writer_ui.entry_live {
            Some(entry) => text_editor::Content::with_text(&entry.text),
            _ => text_editor::Content::with_text(""),
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
        let task;
        match message {
            Message::EditorAction(action) => {
                match &action {
                    text_editor::Action::Edit(_edit) => {
                        // block editing when in readonly mode
                        if !self.writer_ui.is_readonly() && self.writer_ui.is_entry_selected() {
                            self.content.perform(action);
                            self.writer_ui.content_changed(&self.content.text());
                        }
                    }
                    _ => self.content.perform(action),
                }

                task = Task::none();
            }

            Message::Tick => {
                self.writer_ui.on_tick();
                task = self.task_from_ui();
            }
            Message::EntryClicked(entry_id) => {
                self.writer_ui.on_entry_selected(entry_id);
                task = Task::none();
            }
            Message::NewEntryClicked => {
                self.writer_ui.on_new_entry();
                task = operation::focus(Id::new(TEXT_EDITOR_ID));
            }

            Message::LiveEntryClicked => {
                task = Task::none();
            }

            Message::CliChanged(s) => {
                self.writer_ui.on_cli_input(s);
                task = Task::none();
            }

            Message::CliSubmited => {
                self.writer_ui.on_cli_submit();
                task = Task::none();
            }

            Message::SearchChanged(s) => {
                self.writer_ui.on_search_input(s);
                task = Task::none();
            }

            Message::SearchSubmited => {
                self.writer_ui.on_search_submit();
                task = Task::none();
            }

            Message::CreateDateChanged(s) => {
                self.writer_ui.on_create_date_input(s);
                task = Task::none();
            }

            Message::CreateDateSubmited => {
                self.writer_ui.on_create_date_submit();
                task = Task::none();
            }
            Message::SortToggled => {
                self.writer_ui.on_sort_toggled();
                task = Task::none();
            }
            Message::CreateDateDoubleClicked => {
                self.writer_ui.on_create_date_doubleclicked();
                task = operation::focus(Id::new(CREATE_DATE_EDIT_ID));
            }
            Message::DeleteEntryClicked => {
                self.writer_ui.on_delete_entry_clicked();
                task = Task::none();
            }
            Message::ResetCliClicked => {
                self.writer_ui.on_reset_cli_clicked();
                task = Task::none();
            }
            Message::LoaderPagination(next) => {
                self.writer_ui.on_pagination_clicked(next);
                task = Task::none();
            }
        }

        if self.writer_ui.editor_needs_refresh {
            self.refresh_editor();
            self.writer_ui.editor_needs_refresh = false;
        }

        task
    }

    pub fn view(&self) -> Element<'_, Message> {
        // DIV : Editor Text Zone + Command bar
        let div_editor = self.view_editor();
        // DIV : Separator
        let div_sep = container("")
            .width(20)
            .height(Length::Fill)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default()
                    .background(Background::Color(palette.background_neutral))
            });
        // DIV : Left Pan
        let div_leftpan = self.view_left_pan();

        // All
        if let Some(message) = &self.writer_ui.error_screen {
            row![self.view_error_screen(message.clone())].into()
        } else {
            row![div_leftpan, div_sep, div_editor].into()
        }
    }

    /**
     * VIEW : Left Pan
     */
    fn view_left_pan(&self) -> Element<'_, Message> {
        // DIV : ADD Button
        let mut div_add = components::button::button_action(style_conf::icon_add())
            .width(Length::Fixed(40.))
            .height(Length::Fill);

        //if self.writer_ui.view_button_newentry_enabled() {
        div_add = div_add.on_press(Message::NewEntryClicked);
        //}

        // Another control button
        let sort_icon: Text<'_>;
        if self.writer_ui.order_by_date_create {
            sort_icon = style_conf::icon_sort_create();
        } else {
            sort_icon = style_conf::icon_sort_modify();
        };

        let mut div_sort = components::button::button_action(sort_icon)
            .width(Length::Fixed(40.))
            .height(Length::Fill);

        if !self.writer_ui.cli_mode {
            div_sort = div_sort.on_press(Message::SortToggled);
        }

        // Search
        let div_search = TextInput::new("search", &self.writer_ui.search_text())
            .width(Length::Fill)
            .size(15.0)
            //.font(style_conf::FONT_COMMAND_LINE)
            .style(styles::text_input::transparent_style)
            .on_submit(Message::SearchSubmited)
            .on_input(Message::SearchChanged)
            .id(Id::new(SEARCH_INPUT_ICED_ID));

        let container_search = container(div_search).padding(1.);

        // control bar
        let control_bar = container(row![div_add, container_search, div_sort])
            .height(Length::Fixed(30.))
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default().background(palette.background_neutral)
            });

        // Live Entry
        let div_live = match &self.writer_ui.entry_live {
            Some(entry) => {
                match self.writer_ui.entry_list.find_by_id(entry.id) {
                    Some(_expl_entry) => {
                        //don't display
                        let button = button_list_entry(text(""), false)
                            .width(Length::Fill)
                            .height(0.0);
                        column![button]
                    }
                    _ => {
                        let live_text = text(entry.title(25))
                            .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                            .wrapping(Wrapping::WordOrGlyph)
                            .shaping(text::Shaping::Advanced);
                        let button = button_list_entry(live_text, true)
                            .width(Length::Fill)
                            .height(30.)
                            .on_press(Message::LiveEntryClicked);
                        let separator = container(space()).height(4.0).width(Length::Fill).style(
                            |_theme: &Theme| {
                                let palette = style_conf::palette();
                                container::Style::default().background(palette.background_neutral)
                            },
                        );
                        column![button, separator]
                    }
                }
            }
            _ => {
                let button = button_list_entry(text(""), false)
                    .width(Length::Fill)
                    .height(0.0);
                column![button]
            }
        };

        //DIV : list of entries
        let div_entries = column(self.writer_ui.entry_view_iter().map(|entry_view| {
            // entry link text
            let link_text = text(if entry_view.title.is_empty() {
                "...".to_string()
            } else {
                entry_view.title
            })
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .wrapping(Wrapping::WordOrGlyph)
            .shaping(text::Shaping::Advanced);

            //entry button
            let entry_button = button_list_entry(link_text, entry_view.selected)
                .width(Length::Fill)
                .height(35.)
                .on_press(Message::EntryClicked(entry_view.id));

            let separator =
                container(space())
                    .height(0.0)
                    .width(Length::Fill)
                    .style(|_theme: &Theme| {
                        let palette = style_conf::palette();
                        container::Style::default().background(palette.background_neutral)
                    });

            column![entry_button, separator].into()
        })); //Column

        // entries Container
        let container_entries = container(div_entries)
            .height(Length::Fill)
            .padding(Padding {
                bottom: 20.,
                ..Default::default()
            })
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default().background(palette.background_secondary)
            });

        // entries scrollable
        let scrollbar: Scrollbar = Scrollbar::default().spacing(0).width(6).scroller_width(6);
        let direction = scrollable::Direction::Vertical(scrollbar);
        let entries_scroll = scrollable(container_entries)
            .direction(direction)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::text_editor::scroller_style);

        let separator = Space::new().width(Length::Fill).height(Length::Fixed(1.0));

        // Pagination details
        let div_pagination;
        let pagination_brief = self.writer_ui.pagination_brief();
        if !self.writer_ui.cli_mode
            && pagination_brief.2 > (pagination_brief.1 - pagination_brief.0)
        {
            let div_page_prev = button_action(style_conf::icon_left())
                .width(50.)
                .height(Length::Fill)
                .on_press(Message::LoaderPagination(false));
            let div_page_next = button_action(style_conf::icon_right())
                .width(50.)
                .height(Length::Fill)
                .on_press(Message::LoaderPagination(true));

            let page_brief = format!(
                "{} - {} / {}",
                pagination_brief.0, pagination_brief.1, pagination_brief.2
            );
            let div_page_text = text(page_brief)
                .align_x(text::Alignment::Center)
                .size(13.)
                .wrapping(Wrapping::WordOrGlyph)
                .shaping(text::Shaping::Advanced)
                .color(palette().text)
                .width(Length::Fill);

            let mut pagination_row = row![];
            if pagination_brief.0 > 0 {
                pagination_row = pagination_row.push(div_page_prev);
            }else{
                pagination_row = pagination_row.push(container(space()).width(50.));
            }
            pagination_row = pagination_row.push(div_page_text);
            if pagination_brief.2 > pagination_brief.1 {
                pagination_row = pagination_row.push(div_page_next);
            }else{
                pagination_row = pagination_row.push(container(space()).width(50.));
            }

            div_pagination = container(pagination_row)
                .height(Length::Fixed(30.))
                .padding(Padding {
                    top: 5.,
                    ..Default::default()
                })
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    container::Style::default().background(palette.background_secondary)
                });
        } else {
            let page_brief = format!("{}", pagination_brief.2);
            let div_page_text = text(page_brief)
                .align_x(text::Alignment::Center)
                .size(13.)
                .wrapping(Wrapping::WordOrGlyph)
                .shaping(text::Shaping::Advanced)
                .color(palette().text)
                .width(Length::Fill);

            div_pagination = container(div_page_text)
                .height(Length::Fixed(30.))
                .padding(Padding {
                    top: 5.,
                    ..Default::default()
                })
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    container::Style::default().background(palette.background_secondary)
                });
        }

        // DIV : Left Pan
        let left_pan = container(column![
            control_bar,
            separator,
            div_live,
            entries_scroll,
            div_pagination
        ])
        .width(250)
        .height(Length::Fill)
        .padding(Padding {
            top: 0.,
            right: 1.,
            bottom: 1.,
            left: 0.,
        })
        .style(|_theme: &Theme| {
            let palette = style_conf::palette();
            let border = border::width(1).color(palette.background_main);
            container::Style::default()
                .background(palette.background_main)
                .border(border)
        });

        left_pan.into()
    }

    /**
     * VIEW : editor status zone
     */
    fn view_editor_status(&self) -> Element<'_, Message> {
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

            row![text_ymd, text_space, text_wdm]
                .height(Length::Shrink)
                .width(Length::Fixed(180.))
        };

        //calculate the date_time to display
        let div_date_create;
        let div_date_modify;
        let entry_id;
        match self.writer_ui.view_status_current_entry_date() {
            Some((id, dt_create, tm_create, dt_modify, tm_modify)) => {
                entry_id = id;
                div_date_create = dt_view(dt_create, tm_create);
                div_date_modify = dt_view(dt_modify, tm_modify);
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

        //mouse areas
        let date_modify = TextInput::new("epoch", &self.writer_ui.create_date_change_text)
            .width(Length::Fixed(150.))
            .size(10.0)
            .style(styles::text_input::main_style_borders)
            .on_submit(Message::CreateDateSubmited)
            .on_input(Message::CreateDateChanged)
            .id(Id::new(CREATE_DATE_EDIT_ID));

        let date_create_area;
        if self.writer_ui.modifying_create_date > 0 {
            date_create_area = container(date_modify);
        } else {
            date_create_area = container(
                mouse_area(div_date_create).on_double_click(Message::CreateDateDoubleClicked),
            );
        }

        let button_delete: Element<'_, _, _, _> = if self.writer_ui.is_cli_visible() {
            button_action(style_conf::icon_delete_entry().height(7.))
                .on_press(Message::DeleteEntryClicked)
                .into()
        } else {
            Space::new().width(32).into()
        };

        // status bar
        container(row![
            Space::new().width(20),
            date_create_area,
            text("-    ")
                .size(style_conf::STYLE_TEXT_SIZE_EDITOR_STATUS)
                .style(|_theme: &Theme| {
                    //let palette = theme.extended_palette();
                    let palette = style_conf::palette();
                    text::Style {
                        color: Some(palette.text),
                    }
                }),
            div_date_modify,
            widget::space::horizontal(),
            button_delete,
            widget::space::horizontal(),
            div_id,
            Space::new().width(5)
        ])
        .padding(2)
        .style(|_theme: &Theme| {
            let palette = style_conf::palette();
            container::Style::default().background(Background::Color(palette.background_neutral))
        })
        .into()
    }

    /**
     * VIEW : editor
     */
    fn view_editor(&self) -> Element<'_, Message> {
        // DIV : Command line
        let input_cmd = TextInput::new(
            WriterUi::UI_WRITER_CLI_PLACEHOLDER,
            &self.writer_ui.cli_text(),
        )
        .width(Length::Fill)
        .size(style_conf::STYLE_TEXT_SIZE_COMMAND)
        .font(style_conf::FONT_COMMAND_LINE)
        .style(styles::text_input::main_style)
        .on_submit(Message::CliSubmited)
        .on_input(Message::CliChanged)
        .id(Id::new(CLI_INPUT_ICED_ID));

        let button_reset = if self.writer_ui.cli_mode {
            button_action(style_conf::icon_undo())
                .width(30.)
                .on_press(Message::ResetCliClicked)
        } else {
            button_action(text("")).width(30.)
        };

        let div_cli = row![
            input_cmd,
            Space::new().width(10),
            button_reset,
            Space::new().width(5),
        ];

        let mut div_command_cont;
        if self.writer_ui.is_cli_visible() {
            div_command_cont = container(div_cli).height(Length::Fixed(30.));
        } else {
            div_command_cont = container(space()).height(Length::Fixed(30.));
        }

        div_command_cont = div_command_cont
            .width(Length::Fill)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default()
                    .background(Background::Color(palette.background_neutral))
            });

        // DIV : Editor Text Zone
        let mut div_editor = text_editor(&self.content)
            .height(Length::Fill)
            .size(style_conf::TEXT_EDITOR_FONT_SIZE)
            .padding(Padding {
                top: 0.,
                right: 10.,
                bottom: 2.,
                left: 20.,
            })
            .font(style_conf::FONT_EDITOR)
            .style(styles::text_editor::main_style)
            .id(Id::new(TEXT_EDITOR_ID));
        // if self.writer_ui.is_entry_selected() {
        // let the editor disabled if there is no current entry.
        div_editor = div_editor.on_action(Message::EditorAction);
        //}

        let editor_cont = if self.writer_ui.is_entry_selected() {
            container(div_editor).height(Length::Fill)
        } else {
            container(space().height(Length::Fill))
                .height(Length::Fill)
                .width(Length::Fill)
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    container::Style::default()
                        .background(Background::Color(palette.background_neutral))
                })
        };

        // DIV : Editor Status Zone
        let div_editor_status = if self.writer_ui.is_entry_selected() {
            self.view_editor_status()
        } else {
            container(space()).into()
        };

        //Editor
        column![div_command_cont, editor_cont, div_editor_status].into()
    }

    fn view_error_screen(&self, message: String) -> Element<'_, Message> {
        //Error optional div
        let msg_text = text(message)
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .shaping(text::Shaping::Advanced);
        let msg_button = Button::new(msg_text)
            .width(400)
            .style(styles::button::button_entry_selected)
            .clip(true);
        //.on_press(Message::EntryClicked(0));

        container(msg_button)
            .center_y(Length::Fill)
            .center_x(Length::Fill)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default().background(Background::Color(palette.background_main))
            })
            .into()
    }

    pub fn refresh_editor(&mut self) {
        match &self.writer_ui.entry_live {
            Some(entry) => {
                self.content = text_editor::Content::with_text(&entry.text);
            }
            _ => {
                self.content = text_editor::Content::with_text("");
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
        }
    }

    /**
     * returns the ID of the text input to gain focus when the program starts.
     */
    pub fn search_input_id_to_focus() -> &'static str {
        &SEARCH_INPUT_ICED_ID
    }

    /**
     * returns the ID of the text input to gain focus when the program starts.
     */
    pub fn cli_input_id_to_focus() -> &'static str {
        &CLI_INPUT_ICED_ID
    }

    /**
     * returns the ID of the text editor.
     */
    pub fn text_editor_id_to_input() -> &'static str {
        &TEXT_EDITOR_ID
    }
}
