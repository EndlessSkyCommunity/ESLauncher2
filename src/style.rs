use iced::{
    button, Background, Color, Font, HorizontalAlignment, Length, Text, Vector, VerticalAlignment,
};

pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../assets/IcoMoon-Free.ttf"),
};
pub const LOG_FONT: Font = Font::External {
    name: "DejaVuSansMono",
    bytes: include_bytes!("../assets/DejaVuSansMono.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .size(20)
}

pub fn pause_icon() -> Text {
    icon('\u{EA1D}')
}

pub fn play_icon() -> Text {
    icon('\u{EA1C}')
}

pub fn update_icon() -> Text {
    icon('\u{E9C7}')
}

pub fn delete_icon() -> Text {
    icon('\u{E9AD}')
}

pub fn folder_icon() -> Text {
    icon('\u{E930}')
}

pub enum Button {
    Icon,
    Destructive,
    Tab(bool),
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        match self {
            Button::Icon => button::Style {
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..button::Style::default()
            },
            Button::Destructive => button::Style {
                background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
                border_radius: 5,
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 1.0),
                ..button::Style::default()
            },
            Button::Tab(active) => {
                if *active {
                    button::Style {
                        background: Some(Background::Color(Color::WHITE)),
                        border_color: Color::from_rgb(0.5, 0.5, 0.5),
                        border_width: 1,
                        border_radius: 3,
                        ..button::Style::default()
                    }
                } else {
                    button::Style {
                        ..button::Style::default()
                    }
                }
            }
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        button::Style {
            text_color: match self {
                Button::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                _ => active.text_color,
            },
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }
}
