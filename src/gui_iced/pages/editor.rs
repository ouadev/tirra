use crate::gui_iced::components;
use crate::gui_iced::components::button::button_list_entry;
use crate::gui_iced::styles::{self, style_conf};
use crate::ui::ui::{BgRun, TirraInterface, WriterUi};

use iced::theme::Theme;

use iced::widget::scrollable::Scrollbar;
use iced::widget::{
    self, column, container, mouse_area, row, scrollable, text, text_editor, Button, Id, Space,
    TextInput,
};
use iced::widget::{operation, Text};
use iced::Task;
use iced::{border, Background};
use iced::{Element, Length};

const SEARCH_INPUT_ICED_ID: &str = "searchinput-id";
const CLI_INPUT_ICED_ID: &str = "cliinput-id";
const CREATE_DATE_EDIT_ID: &str = "create_date_input_id";
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
    CliSubmited,
    CliChanged(String),
    SearchSubmited,
    SearchChanged(String),
    CreateDateSubmited,
    CreateDateChanged(String),
    SortToggled,
    CreateDateDoubleClicked,
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

            Message::SearchChanged(s) => {
                self.writer_ui.on_search_input(s);
                Task::none()
            }

            Message::SearchSubmited => {
                self.writer_ui.on_search_submit();
                self.refresh_editor();
                Task::none()
            }

            Message::CreateDateChanged(s) => {
                self.writer_ui.on_create_date_input(s);
                Task::none()
            }

            Message::CreateDateSubmited => {
                self.writer_ui.on_create_date_submit();
                self.refresh_editor();
                Task::none()
            }
            Message::SortToggled => {
                self.writer_ui.on_sort_toggled();
                self.refresh_editor();
                Task::none()
            }
            Message::CreateDateDoubleClicked => {
                self.writer_ui.on_create_date_doubleclicked();
                operation::focus(Id::new(CREATE_DATE_EDIT_ID))
            }
        }
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
            .height(Length::Fixed(30.));

        if self.writer_ui.view_button_newentry_enabled() {
            div_add = div_add.on_press(Message::NewEntryClicked);
        }

        // Another control button
        let sort_icon: Text<'_>;
        if self.writer_ui.order_by_date_create {
            sort_icon = style_conf::icon_sort_create();
        } else {
            sort_icon = style_conf::icon_sort_modify();
        };

        let div_sort = components::button::button_action(sort_icon)
            .width(Length::Fixed(40.))
            .height(Length::Fixed(30.))
            .on_press(Message::SortToggled);

        // Search
        let div_search = TextInput::new("search", &self.writer_ui.search_text())
            .width(Length::Fill)
            .size(15.0)
            //.font(style_conf::FONT_COMMAND_LINE)
            .style(styles::text_input::transparent_style)
            .on_submit(Message::SearchSubmited)
            .on_input(Message::SearchChanged)
            .id(Id::new(SEARCH_INPUT_ICED_ID));

        // control bar
        let control_bar = container(row![div_add, div_search, div_sort]).style(|_theme: &Theme| {
            let palette = style_conf::palette();
            container::Style::default().background(palette.background_neutral)
        });

        //DIV : list of entries
        let div_entries = column(self.writer_ui.entry_view_iter().map(|entry_view| {
            // entry link text
            let link_text = text(entry_view.title)
                .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
                .shaping(text::Shaping::Advanced);

            //entry button
            let entry_button = button_list_entry(link_text, entry_view.selected)
                .width(Length::Fill)
                .on_press(Message::EntryClicked(entry_view.id));

            column![entry_button].into()
        })); //Column

        // entries Container
        let container_entries =
            container(div_entries)
                .height(Length::Fill)
                .style(|_theme: &Theme| {
                    let palette = style_conf::palette();
                    let border = border::width(1).color(palette.background_main);
                    container::Style::default()
                        .background(palette.background_secondary)
                        .border(border)
                });

        // entries scrollable
        let scrollbar: Scrollbar = Scrollbar::default().spacing(0).width(6).scroller_width(6);
        let direction = scrollable::Direction::Vertical(scrollbar);
        let entries_scroll = scrollable(container_entries)
            .direction(direction)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::text_editor::scroller_style);

        // DIV : Left Pan
        container(column![control_bar, entries_scroll])
            .width(250)
            .into()
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
                .width(Length::Fixed(150.))
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

        // status bar
        container(row![
            Space::new().width(20),
            date_create_area,
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
            widget::space::horizontal(),
            div_id
        ])
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
        let div_cmd_input = TextInput::new(
            WriterUi::UI_WRITER_CLI_PLACEHOLDER,
            &self.writer_ui.cli_text(),
        )
        .width(Length::Fill)
        .size(style_conf::STYLE_TEXT_SIZE_COMMAND)
        .font(style_conf::FONT_COMMAND_LINE)
        .style(styles::text_input::transparent_style)
        .on_submit(Message::CliSubmited)
        .on_input(Message::CliChanged)
        .id(Id::new(CLI_INPUT_ICED_ID));

        let mut div_command_cont;
        if self.writer_ui.is_cli_visible() {
            div_command_cont = container(div_cmd_input).height(30);
        } else {
            div_command_cont = container("").height(10);
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
            .height(Length::Shrink)
            .padding(20)
            .font(style_conf::FONT_EDITOR)
            .style(styles::text_editor::main_style);
        if self.writer_ui.is_entry_selected() {
            // let the editor disabled if there is no current entry.
            div_editor = div_editor.on_action(Message::EditorAction);
        }

        let editor_cont = container(div_editor).height(Length::Fill);
        let scrollbar: Scrollbar = Scrollbar::default().spacing(0).width(6).scroller_width(6);
        let direction = scrollable::Direction::Vertical(scrollbar);
        let div_editor_text = scrollable(editor_cont)
            .direction(direction)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::text_editor::scroller_style);

        // DIV : Editor Status Zone
        let div_editor_status = self.view_editor_status();

        //Editor
        column![div_command_cont, div_editor_text, div_editor_status].into()
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
}
