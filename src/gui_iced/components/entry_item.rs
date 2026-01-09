use crate::gui_iced::styles::{self, style_conf};
use iced::widget::button;
use iced::widget::button::Status;
use iced::widget::button::Style;
use iced::widget::space;
use iced::{
    widget::{container, row, text, Button, Container},
    Length,
};
use iced::{Background, Border, Color, Shadow, Theme};

pub fn entry_item_component<'a, PressMessage>(
    content_text: String,
    selected: bool,
    on_click: PressMessage,
) -> Container<'a, PressMessage>
where
    PressMessage: Clone + 'a,
{
    // entry link text
    let link_text = text(content_text)
        .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
        .shaping(text::Shaping::Advanced);

    //entry button
    let entry_button = Button::new(link_text)
        .style(if selected {
            style_button_title_selected
        } else {
            style_button_title
        })
        .clip(true)
        .width(Length::Fill)
        .on_press(on_click);

    // additional
    let additional_info;

    if selected {
        additional_info = text("x")
            .size(style_conf::STYLE_TEXT_SIZE_NORMAL)
            .shaping(text::Shaping::Advanced)
            .style(|_theme: &Theme| {
                let palette = style_conf::palette();
                text::Style {
                    color: Some(palette.text),
                }
            });
    } else {
        additional_info = text("").width(0);
    }

    let additional_container =
        container(additional_info)
            .padding(2)
            .style(move |_theme: &Theme| {
                let palette = style_conf::palette();
                if selected {
                    container::Style::default()
                        .background(Background::Color(palette.background_main))
                } else {
                    container::Style::default().background(Background::Color(Color::TRANSPARENT))
                }
            });

    //final container
    let entry_row = row![entry_button, additional_container];
    let container = container(entry_row);
    container
}

fn style_button_title(_theme: &Theme, status: Status) -> Style {
    let palette = style_conf::palette();
    let base = Style {
        background: Some(Background::Color(Color::TRANSPARENT)),
        text_color: palette.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.background_main)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(palette.control_disabled)),
            ..base
        },
    }
}

fn style_button_title_selected(_theme: &Theme, status: Status) -> Style {
    //let palette = theme.extended_palette();
    let palette = style_conf::palette();
    let base = Style {
        background: Some(Background::Color(palette.background_main)),
        text_color: palette.text,
        border: Border::default(),
        shadow: Shadow::default(),
        snap: false,
    };

    match status {
        Status::Active | Status::Pressed => base,
        Status::Hovered => Style {
            background: Some(Background::Color(palette.background_main)),
            ..base
        },
        Status::Disabled => Style {
            background: Some(Background::Color(palette.control_disabled)),
            ..base
        },
    }
}
