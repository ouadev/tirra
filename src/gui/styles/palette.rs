use iced::color;
use iced::Color;

pub struct Palette {
    pub background_main: Color,
    pub background_secondary: Color,
    pub control_main: Color,
    pub control_secondary: Color,
    pub control_disabled: Color,
    pub text: Color,
    // ...
}

impl Palette {
    pub const LIGHT: Self = Self {
        background_main: color!(0xffffff),
        background_secondary: color!(0xd9d9d9),
        control_main: color!(0xb6deca),
        control_secondary: color!(0xb6deca),
        control_disabled: color!(0x000000),
        text: color!(0x000000),
    };

    pub const LIGHT_CATPPUCCIN: Self = Self {
        background_main: color!(0xeff1f5),
        background_secondary: color!(0xdce0e8),
        control_main: color!(0x04a5e5),
        control_secondary: color!(0xb6deca),
        control_disabled: color!(0x000000),
        text: color!(0x4c4f69),
    };

    pub const DARK: Self = Self {
        background_main: color!(0x282A36),
        background_secondary: color!(0x44475a),
        control_main: color!(0x6272a4),
        control_secondary: color!(0xb6deca),
        control_disabled: color!(0x000000),
        text: color!(0xFFFFFF),
    };
}
