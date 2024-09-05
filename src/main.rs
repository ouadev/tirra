use iced::executor;
use iced::highlighter::{self, Highlighter};
use iced::keyboard;
use iced::theme::{self, Theme};
use iced::widget::{
    button, column, container, horizontal_space, pick_list, row, text,
    text_editor, tooltip,
};
use iced::{
    Alignment, Application, Command, Element, Font, Length, Settings,
    Subscription,
};

use std::ffi;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::io;
use std::fs::File;
use std::fs;



pub fn main() -> iced::Result {
    Counter::run(Settings::default())
}

struct Counter {
    value: i32,
    content: text_editor::Content,
    file: Option<PathBuf>,
    theme: highlighter::Theme,
    is_loading: bool,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ActionPerformed(text_editor::Action),
    ThemeSelected(highlighter::Theme),
    NewFile,
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    SaveFile,
    FileSaved(Result<PathBuf, Error>),
}


#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    IoError(io::ErrorKind),
}

impl Application for Counter {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                value: 0,
                file: None,
                content: text_editor::Content::new(),
                theme: highlighter::Theme::SolarizedDark,
                is_loading: false,
                is_dirty: false,
            },
            //Command::perform( (), Message::FileOpened),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("TiRRa")
    }


    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.perform(action);

                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }
            Message::NewFile => {
                if !self.is_loading {
                    self.file = None;
                    self.content = text_editor::Content::new();
                }

                Command::none()
            }
            Message::OpenFile => {
                if self.is_loading {
                    Command::none()
                } else {
                    self.is_loading = true;

                    //Command::perform(String("ded"), Message::FileOpened)
                    Command::none()
                }
            }
            Message::FileOpened(result) => {
                self.is_loading = false;
                self.is_dirty = false;

                if let Ok((path, contents)) = result {
                    self.file = Some(path);
                    self.content = text_editor::Content::with_text(&contents);
                }

                Command::none()
            }
            Message::SaveFile => {
                if self.is_loading {
                    Command::none()
                } else {
                    //self.is_loading = true;

                    //Command::perform(
                    //    save_file(self.file.clone(), self.content.text()),
                    //    Message::FileSaved,
                    //)
                    save_content_to_disk(&self.content.text());
                    Command::none()
                }
            }
            Message::FileSaved(result) => {
                println!("saved");
                self.is_loading = false;

                if let Ok(path) = result {
                    self.file = Some(path);
                    self.is_dirty = false;
                }

                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            keyboard::Key::Character("s") if modifiers.command() => {
                Some(Message::SaveFile)
            }
            _ => None,
        })
    }

    fn view(&self) -> Element<Message> {
        let controls = row![
            action(new_icon(), "New file", Some(Message::NewFile)),
            action(
                open_icon(),
                "Open file",
                (!self.is_loading).then_some(Message::OpenFile)
            ),
            action(
                save_icon(),
                "Save file",
                self.is_dirty.then_some(Message::SaveFile)
            ),
            horizontal_space(),
            pick_list(
                highlighter::Theme::ALL,
                Some(self.theme),
                Message::ThemeSelected
            )
            .text_size(14)
            .padding([5, 10])
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let status = row![
            text(if let Some(path) = &self.file {
                let path = path.display().to_string();

                if path.len() > 60 {
                    format!("...{}", &path[path.len() - 40..])
                } else {
                    path
                }
            } else {
                String::from("New file")
            }),
            horizontal_space(),
            text({
                let (line, column) = self.content.cursor_position();

                format!("{}:{}", line + 1, column + 1)
            })
        ]
        .spacing(10);

        column![
            controls,
            text_editor(&self.content)
                .height(Length::Fill)
                .on_action(Message::ActionPerformed)
                .highlight::<Highlighter>(
                    highlighter::Settings {
                        theme: self.theme,
                        extension: self
                            .file
                            .as_deref()
                            .and_then(Path::extension)
                            .and_then(ffi::OsStr::to_str)
                            .map(str::to_string)
                            .unwrap_or(String::from("rs")),
                    },
                    |highlight, _theme| highlight.to_format()
                ),
            status,
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }


}


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

fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}


fn save_content_to_disk (cont: &str) -> () {
    //let mut secret_file = File::create("/tmp/notes.txt").expect("creation failed");
    println!("saving....");

    fs::write("/tmp/editor.txt",cont).expect("");
}