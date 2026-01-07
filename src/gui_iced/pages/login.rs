use iced::theme::Theme;
use iced::widget::{column, container, row, space, text, Button, Id, Text, TextInput};
use iced::{alignment, Background, Task};
use iced::{Element, Length};

use crate::gui_iced::styles::{self, style_conf};
use crate::ui::ui::{self, LoginUi, TirraInterface};

const LOGIN_INPUT_ICED_ID: &str = "pwdinput-id";
pub struct LoginPage {
    pub login_ui: LoginUi,
}

#[derive(Debug, Clone)]
pub enum Message {
    PwdInputChanged(String),
    LoginButtonPressed,
    LoginSuccess,
}
impl LoginPage {
    pub fn new(db_location: &str) -> (Self, Task<Message>) {
        //instantiate Login Tirra UI
        let ui = LoginUi::new(db_location);
        //return
        (Self { login_ui: ui }, Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PwdInputChanged(s) => {
                self.login_ui.on_pwd(s);
                Task::none()
            }
            Message::LoginButtonPressed => {
                self.login_ui.on_login();
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        //DIV : margin-top
        // Header logo thing // ⵜⵉⵔⵔⴰ
        let div_logo = Text::new("ⵜⵔ")
            .size(style_conf::STYLE_TEXT_SIZE_LOGO)
            .font(style_conf::FONT_LOGIN_LOGO)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .shaping(text::Shaping::Advanced)
            .style(|_theme: &Theme| {
                //let palette = theme.extended_palette();
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.control_main),
                }
            });
        let div_logo_cont = container(div_logo).center_x(Length::Fill).padding(20);

        // DIV : target Db
        let div_db = Text::new(&self.login_ui.db_location)
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });
        let div_db_cont = container(div_db).center_x(Length::Fill).padding(20);
        // DIV : Text Input
        let div_pwd = TextInput::new(
            ui::LoginUi::UI_LOGIN_PWDINPUT_PLACEHOLDER,
            &self.login_ui.password,
        )
        .width(300)
        .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
        .secure(true)
        .on_submit(Message::LoginButtonPressed)
        .on_input(Message::PwdInputChanged)
        .style(styles::text_input::main_style)
        .id(Id::new(LOGIN_INPUT_ICED_ID));

        //let div_pwd_cont = container(div_pwd).center_x(Length::Fill);
        // DIV : Login Button
        let button_text = if self.login_ui.db_found {
            ui::LoginUi::UI_LOGIN_BUTTON_TEXT_DECRYPT
        } else {
            ui::LoginUi::UI_LOGIN_BUTTON_TEXT_NEWDB
        };
        let div_decrypt_button = Button::new(
            text(button_text)
                .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
                .align_x(text::Alignment::Center)
                .align_y(alignment::Vertical::Center),
        )
        .width(Length::Fixed(60.))
        .style(styles::button::button_main)
        .on_press(Message::LoginButtonPressed);
        //let div_dec_cont = container(div_decrypt_button).center_x(Length::Fill);
        let div_dec_cont =
            container(row![div_pwd, space().width(10), div_decrypt_button]).center_x(Length::Fill);

        // DIV : Information box
        let div_info = Text::new(&self.login_ui.info_text)
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
        let div_login =
            container(column![div_logo_cont, div_db_cont, div_dec_cont, div_info_cont].spacing(10))
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
        self.login_ui.title()
    }

    /**
     * returns the ID of the text input to gain focus when the program starts.
     */
    pub fn text_input_id_to_focus() -> &'static str {
        &LOGIN_INPUT_ICED_ID
    }
}
