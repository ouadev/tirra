use iced::theme::Theme;
use iced::widget::{column, container, text_input, Button, Text, TextInput};
use iced::{theme, Background, Command};
use iced::{Element, Length};

use crate::db;
use crate::gui::styles::button::TirraButtonStyle;
use crate::gui::styles::button::TirraButtonType;
use crate::gui::styles::style_constants;
use crate::tirracrypto::TirraCrypto;

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
    pub fn new(db_location: &str) -> (Self, Command<Message>) {
        // Check database file existence
        let mut info_text = String::new();
        info_text.push_str(&format!(" . db: {}\n", db_location));
        let db_found: bool;
        if db::tirra_db_found(db_location) == true {
            db_found = true;
        } else {
            info_text.push_str(&format!(" . DATABASE FILE NOT FOUND.\n"));
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
            Command::none(),
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
                        db::tirra_db_init(&crypto).expect("database init error");
                        // insert first empty entry
                        db::tirra_db_add_entry("Mar7baaaa ...", &crypto)
                            .expect("first entry add failed");
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
    //Command::none()

    pub fn view(&self) -> Element<Message> {
        //DIV : margin-top
        let div_sep = container("")
            .width(Length::Fill)
            .height(50)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                container::Appearance::default().with_background(palette.background.strong.color)
            });
        // DIV : Text Input
        let div_pwd = TextInput::new("Passphrase", &self.password)
            .width(300)
            .secure(true)
            .on_submit(Message::LoginButtonPressed)
            .on_input(Message::PwdInputChanged)
            .id(text_input::Id::new(LOGIN_INPUT_ICED_ID));

        let div_pwd_cont = container(div_pwd).width(Length::Fill).center_x();
        // DIV : Login Button
        let button_text = if self.db_found {
            "Decrypt & Access"
        } else {
            "New Database"
        };
        let div_decrypt_button = Button::new(button_text)
            .width(Length::Shrink)
            .style(theme::Button::custom(TirraButtonStyle {
                button_type: TirraButtonType::Main,
                selected: false,
            }))
            .on_press(Message::LoginButtonPressed);
        let div_dec_cont = container(div_decrypt_button).width(Length::Fill).center_x();

        // DIV : Information box
        let div_info = Text::new(&self.info_text);
        let div_info_cont = container(div_info).center_x().padding(20);

        // DIV : LoginContainer
        let div_login =
            container(column![div_sep, div_pwd_cont, div_dec_cont, div_info_cont].spacing(10))
                .width(400)
                .height(Length::Fill)
                .center_x()
                .style(|theme: &Theme| {
                    let palette = theme.extended_palette();
                    container::Appearance::default()
                        .with_background(palette.background.strong.color)
                });

        let body = container(div_login)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .style(|_theme: &Theme| {
                container::Appearance::default()
                    .with_background(Background::Color(style_constants::STYLE_EDITOR_BG_COLOR))
            });

        body.into()
    }

    pub fn title(&self) -> String {
        format!("Tirra - Access ...")
    }

    /**
     * returns the ID of the text input to gain focus when the program starts.
     */
    pub fn text_input_id_to_focus() -> &'static str {
        &LOGIN_INPUT_ICED_ID
    }
}
