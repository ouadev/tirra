use iced::font::{Family, Stretch, Style, Weight};
use iced::{Color, Font};

pub const FONT_DEJAVU_SANS_MONO_BYTES: &[u8] =
    include_bytes!("../../../resources/dejavu-sans-mono/DejaVuSansMono.ttf");
pub const FONT_DEJAVU_SANS_MONO: Font = Font {
    family: Family::Name("DejaVu Sans Mono"),
    weight: Weight::Normal,
    stretch: Stretch::Normal,
    style: Style::Normal,
};

pub const STYLE_EDITOR_BG_COLOR: iced::Color = Color {
    r: 182.0 / 255.0,
    g: 222.0 / 255.0,
    b: 202.0 / 255.0,
    a: 1.0,
};

pub const STYLE_BUTTON_ADD_COLOR: iced::Color = Color {
    r: 160. / 255.0,
    g: 160. / 255.0,
    b: 160. / 255.0,
    a: 1.0,
};

pub const STYLE_BUTTON_ENTRY_COLOR_SELECTED: iced::Color = STYLE_EDITOR_BG_COLOR;

pub const STYLE_BUTTON_ENTRY_COLOR: iced::Color = Color::TRANSPARENT;
