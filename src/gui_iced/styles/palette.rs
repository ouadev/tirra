use iced::color;
use iced::Color;

pub struct Palette {
    pub background_neutral: Color,
    pub background_main: Color,
    pub background_secondary: Color,
    pub control_main: Color,
    pub control_disabled: Color,
    pub text: Color,
    // ...
}

impl Palette {
    pub const LIGHT_TIRRA: Self = Self {
        background_neutral: color!(0xffffff),
        background_main: color!(0xdfdce4),
        background_secondary: color!(0x0f3f0f6),

        text: color!(0x000000),

        control_main: color!(0x72b775),
        control_disabled: color!(0x0e6174),
    };

    pub const DARK_TIRRA: Self = Self {
        background_neutral: color!(0x1e1f23),
        background_main: color!(0x474d59),
        background_secondary: color!(0x31333f),

        text: color!(0xFFFFFF),

        control_main: color!(0x6272a4),
        control_disabled: color!(0x000000),
    };
}
