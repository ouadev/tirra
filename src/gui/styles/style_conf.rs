use iced::font::{Family, Stretch, Style, Weight};
use iced::{Color, Font};
/*
pub const FONT_EXTERNAL_BYTES: &[u8] =
    include_bytes!("../../../resources/dejavu-serif/DejaVuSerif.ttf");
*/

pub const FONT_EDITOR: Font = Font {
    family: Family::Name("Arial"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_DEFAULT: Font = Font {
    family: Family::Name("Arial"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_COMMAND_LINE: Font = Font {
    family: Family::Monospace,
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const FONT_COMMAND_LINE_BOLD: Font = Font {
    family: Family::Monospace,
    weight: Weight::Medium,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const STYLE_EDITOR_BG_COLOR: iced::Color = Color {
    r: 255. / 255.0,
    g: 255. / 255.0,
    b: 255. / 255.0,
    a: 1.0,
};

pub const STYLE_DISABLED_BUTTON_COLOR: iced::Color = Color {
    r: 75. / 255.0,
    g: 75. / 255.0,
    b: 75. / 255.0,
    a: 1.0,
};

pub const COLOR_BUTTON_MAIN: iced::Color = Color {
    r: 182.0 / 255.0,
    g: 222.0 / 255.0,
    b: 202.0 / 255.0,
    a: 1.0,
};

pub const STYLE_COLOR_PAN_BG: iced::Color = Color {
    r: 217.0 / 255.0,
    g: 217.0 / 255.0,
    b: 217.0 / 255.0,
    a: 1.0,
};

pub const STYLE_BUTTON_ENTRY_COLOR_SELECTED: iced::Color = STYLE_EDITOR_BG_COLOR;

pub const STYLE_BUTTON_ENTRY_COLOR: iced::Color = Color::TRANSPARENT;

pub const STYLE_TEXT_SIZE_EDITOR: iced::Pixels = iced::Pixels(23.0);
pub const STYLE_TEXT_SIZE_NORMAL: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_COMMAND: iced::Pixels = iced::Pixels(20.);
pub const STYLE_TEXT_SIZE_EDITOR_STATUS_HIGHLIGHT: iced::Pixels = iced::Pixels(15.);
pub const STYLE_TEXT_SIZE_EDITOR_STATUS: iced::Pixels = iced::Pixels(15.);
