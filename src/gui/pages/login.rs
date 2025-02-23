use iced::theme::Theme;
use iced::widget::{column, container, text, text_input, Button, Text, TextInput};
use iced::{Background, Task};
use iced::{Element, Length};

use crate::common::exception::exception;
use crate::gui::styles::{self, style_conf};
use crate::storage::db;
use crate::storage::tirracrypto::TirraCrypto;

const LOGIN_INPUT_ICED_ID: &str = "pwdinput-id";
pub struct LoginPage {
    pub db_location: String,
    db_found: bool,
    info_text: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    PwdInputChanged(String),
    LoginButtonPressed,
    LoginSuccess,
}
impl LoginPage {
    pub fn new(db_location: &str) -> (Self, Task<Message>) {
        // Check database file existence
        let mut info_text = String::new();
        //info_text.push_str(&format!(" . db: {}\n", db_location));
        let db_found: bool;
        if db::tirra_db_found(db_location) == true {
            db_found = true;
        } else {
            info_text.push_str(&format!(" Database file was not found.\n"));
            db_found = false;
        }
        //return
        (
            Self {
                db_location: String::from(db_location),
                db_found: db_found,
                password: String::from(""),
                info_text: info_text,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Option<Message> {
        match message {
            Message::PwdInputChanged(s) => {
                self.password = s;
                self.info_text = self.password.clone();
                self.info_text = format!("");
                None
            }
            Message::LoginButtonPressed => {
                let crypto = TirraCrypto::new(&self.db_location, self.password.as_bytes());
                if self.db_found == false {
                    // Database is not found, start initialization of a new one at the same location.
                    // intialize the backend
                    if crypto.enc_db_found() == false {
                        // create new db
                        let inited = db::tirra_db_init(&crypto);
                        if let Err(_x) = inited {
                            exception("database init");
                        }
                        // insert first empty entry
                        let empty_added = db::tirra_db_add_entry(
                            db::TIRRA_ENTRY_TYPE_GENERAL,
                            db::TIRRA_FIRST_ENTRY_TEXT,
                            &crypto,
                        );
                        if let Err(_x) = empty_added {
                            exception("database init");
                        }
                        
                        Some(Message::LoginSuccess)
                    } else {
                        panic!("something is up. database is not supposed to be found");
                    }
                } else {
                    if db::tirra_db_try_access(&crypto) {
                        Some(Message::LoginSuccess)
                    } else {
                        self.info_text = format!("Decryption failure: Wrong key");
                        None
                    }
                }
            }
            _ => None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        //DIV : margin-top
        let div_sep = container("")
            .width(Length::Fill)
            .height(50)
            .style(|_theme: &Theme| {
                //let palette = theme.extended_palette();
                let palette = style_conf::palette();
                container::Style::default().background(palette.background_secondary)
            });

        // DIV : target Db
        let div_db = Text::new(&self.db_location)
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .style(|_theme: &Theme| {
                //let palette = theme.extended_palette();
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });
        let div_db_cont = container(div_db).center_x(Length::Fill).padding(20);
        // DIV : Text Input
        let div_pwd = TextInput::new("Passphrase", &self.password)
            .width(300)
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .secure(true)
            .on_submit(Message::LoginButtonPressed)
            .on_input(Message::PwdInputChanged)
            .id(text_input::Id::new(LOGIN_INPUT_ICED_ID));

        let div_pwd_cont = container(div_pwd).center_x(Length::Fill);
        // DIV : Login Button
        let button_text = if self.db_found {
            "Decrypt & Access"
        } else {
            "New Database"
        };
        let div_decrypt_button =
            Button::new(text(button_text).size(style_conf::STYLE_TEXT_SIZE_NORMAL))
                .width(Length::Shrink)
                .style(styles::button::button_main)
                .on_press(Message::LoginButtonPressed);
        let div_dec_cont = container(div_decrypt_button).center_x(Length::Fill);

        // DIV : Information box
        let div_info = Text::new(&self.info_text)
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .style(|_theme: &Theme| {
                //let palette = theme.extended_palette();
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });
        let div_info_cont = container(div_info).center_x(Length::Fill).padding(20);

        // DIV : LoginContainer
        let div_login = container(
            column![
                div_sep,
                div_db_cont,
                div_pwd_cont,
                div_dec_cont,
                div_info_cont
            ]
            .spacing(10),
        )
        .center_x(400)
        .height(Length::Fill)
        .style(|_theme: &Theme| {
            //let palette = theme.extended_palette();
            let palette = style_conf::palette();
            container::Style::default().background(palette.background_secondary)
            //.with_border( Color::BLACK, 1)
        });

        let body = container(div_login)
            .center_x(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                container::Style::default().background(Background::Color(palette.background_main))
            });

        body.into()
    }

    pub fn title(&self) -> String {
        format!("Tirra - Open")
    }

    /**
     * returns the ID of the text input to gain focus when the program starts.
     */
    pub fn text_input_id_to_focus() -> &'static str {
        &LOGIN_INPUT_ICED_ID
    }
}
