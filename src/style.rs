use iced::border::Radius;
use iced::widget::{button, container, Text};
use iced::{alignment, color, Background, Border, Color, Font, Length, Shadow, Theme, Vector};
// use iced_aw::tab_bar;

fn icon(unicode: char) -> Text<'static> {
    Text::new(unicode.to_string())
        .font(Font::with_name("IcoMoon-Free"))
        .width(Length::Fixed(20.))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .size(20)
}

pub fn pause_icon() -> Text<'static> {
    icon('\u{EA1D}')
}

pub fn debug_icon() -> Text<'static> {
    icon('\u{E999}')
}

pub fn play_icon() -> Text<'static> {
    icon('\u{EA1C}')
}
pub fn href_icon() -> Text<'static> {
    icon('\u{EA7E}')
}

pub fn update_icon() -> Text<'static> {
    icon('\u{E9C7}')
}

pub fn delete_icon() -> Text<'static> {
    icon('\u{E9AD}')
}

pub fn folder_icon() -> Text<'static> {
    icon('\u{E930}')
}

pub fn reset_icon() -> Text<'static> {
    icon('\u{E965}')
}

// TODO: This is whack. doesn't feel like intended => rethink
pub fn icon_button(_theme: &Theme, status: button::Status) -> button::Style {
    use button::Catalog;
    ButtonStyle::Icon.style(&ButtonStyle::Icon, status)
}
pub fn text_button(_theme: &Theme, status: button::Status) -> button::Style {
    use button::Catalog;
    ButtonStyle::Text.style(&ButtonStyle::Text, status)
}

// pub fn tab_bar(theme: &Theme, status: tab_bar::Status) -> tab_bar::Style {
//     use iced_aw::tab_bar::*;
//     let background = theme.palette().background;
//     let primary = theme.extended_palette().primary;
//     let secondary = theme.extended_palette().secondary;
//
//     let default = Style {
//         tab_label_background: background.into(),
//         tab_label_border_color: secondary.weak.color,
//         text_color: theme.palette().text,
//         ..Default::default()
//     };
//
//     match status {
//         Status::Active => Style {
//             tab_label_background: primary.base.color.into(),
//             text_color: primary.base.text,
//             ..default
//         },
//         Status::Hovered => Style {
//             tab_label_background: primary.weak.color.into(),
//             text_color: primary.weak.text,
//             ..default
//         },
//         Status::Disabled => Style { ..default },
//         _ => Style {
//             // We don't use these - make it jarring, so if we ever do, it's noticeable
//             tab_label_background: color!(0xff0000).into(),
//             text_color: color!(0x00ff00),
//             ..default
//         },
//     }
// }

pub fn log_container(log: &str) -> container::StyleFn<Theme> {
    use container::Catalog;
    Box::new(move |_| LogContainer::from(log).style(&LogContainer::default()))
}

/// graphic design is my passion
pub enum ButtonStyle {
    Icon,
    Text,
}

impl button::Catalog for ButtonStyle {
    type Class<'a> = ButtonStyle;

    fn default<'a>() -> Self::Class<'a> {
        ButtonStyle::Text
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let active = if button::Status::Active == status {
            // Avoid Stack overflow
            button::Style::default()
        } else {
            self.style(class, button::Status::Active)
        };

        match status {
            button::Status::Active => match self {
                Self::Icon => button::Style {
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..Default::default()
                },
                Self::Text => button::Style {
                    background: Some(Background::Color(Color::WHITE)),
                    border: Border {
                        color: Color::from_rgb(0.8, 0.8, 0.8),
                        width: 0.3,
                        radius: Radius::from(2.0),
                    },
                    shadow: Shadow {
                        offset: Vector::new(0.3, 0.3),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            button::Status::Hovered => match self {
                Self::Icon => button::Style {
                    text_color: Color::from_rgb(0.3, 0.3, 0.3),
                    shadow: Shadow {
                        offset: active.shadow.offset + Vector::new(0.0, 1.0),
                        ..Default::default()
                    },
                    ..active
                },
                Self::Text => button::Style {
                    border: Border {
                        color: Color::from_rgb(0.4, 0.4, 0.4),
                        ..Default::default()
                    },
                    shadow: Shadow {
                        offset: active.shadow.offset + Vector::new(0.1, 0.3),
                        ..Default::default()
                    },
                    ..active
                },
            },
            button::Status::Pressed => Default::default(),
            button::Status::Disabled => match self {
                Self::Text => button::Style {
                    text_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..active
                },
                _ => active,
            },
        }
    }
}

pub struct LogContainer {
    background: Option<Color>,
}

impl From<&str> for LogContainer {
    fn from(log: &str) -> Self {
        Self {
            background: if log.starts_with("WARN") {
                Some(Color::from_rgba(1.0, 1.0, 0.5, 0.5))
            } else if log.starts_with("ERROR") {
                Some(Color::from_rgba(1.0, 0.5, 0.5, 0.5))
            } else {
                None
            },
        }
    }
}

impl container::Catalog for LogContainer {
    type Class<'a> = LogContainer;

    fn default<'a>() -> Self::Class<'a> {
        LogContainer { background: None }
    }

    fn style(&self, _: &Self::Class<'_>) -> container::Style {
        container::Style {
            text_color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
            background: self.background.map(Background::Color),
            ..Default::default()
        }
    }
}
