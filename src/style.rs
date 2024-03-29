use iced::border::Radius;
use iced::widget::{button, container, Text};
use iced::{alignment, Background, Border, Color, Font, Length, Theme, Vector};
use std::default::Default;
use std::rc::Rc;

fn icon(unicode: char) -> Text<'static> {
    Text::new(unicode.to_string())
        .font(Font::with_name("IcoMoon-Free"))
        .width(Length::Fixed(20.))
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center)
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

pub fn icon_button() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(ButtonStyle::Icon))
}
pub fn text_button() -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(ButtonStyle::Text))
}

pub fn tab_bar() -> iced_aw::style::tab_bar::TabBarStyles {
    iced_aw::style::tab_bar::TabBarStyles::Custom(Rc::new(CustomTabBar))
}

pub fn log_container(log: &str) -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(LogContainer::from(log)))
}

/// graphic design is my passion
pub enum ButtonStyle {
    Icon,
    Text,
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        match self {
            ButtonStyle::Icon => button::Appearance {
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..Default::default()
            },
            ButtonStyle::Text => button::Appearance {
                background: Some(Background::Color(Color::WHITE)),
                border: Border {
                    color: Color::from_rgb(0.8, 0.8, 0.8),
                    width: 0.3,
                    radius: Radius::from(2.0),
                },
                shadow_offset: Vector::new(0.3, 0.3),
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        match self {
            ButtonStyle::Icon => button::Appearance {
                text_color: Color::from_rgb(0.3, 0.3, 0.3),
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            },
            ButtonStyle::Text => button::Appearance {
                border: Border {
                    color: Color::from_rgb(0.4, 0.4, 0.4),
                    ..Default::default()
                },
                shadow_offset: active.shadow_offset + Vector::new(0.1, 0.3),
                ..active
            },
        }
    }

    fn disabled(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);
        match self {
            ButtonStyle::Text => button::Appearance {
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..active
            },
            _ => active,
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
                Some(Color::new(1., 1., 0.5, 0.5))
            } else if log.starts_with("ERROR") {
                Some(Color::new(1., 0.5, 0.5, 0.5))
            } else {
                None
            },
        }
    }
}

impl container::StyleSheet for LogContainer {
    type Style = Theme;

    fn appearance(&self, _: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
            background: self.background.map(Background::Color),
            ..Default::default()
        }
    }
}

pub struct CustomTabBar;

impl iced_aw::style::tab_bar::StyleSheet for CustomTabBar {
    type Style = Theme;

    fn active(&self, _: &Self::Style, is_active: bool) -> iced_aw::style::tab_bar::Appearance {
        let tab_label_background = Background::Color(if is_active {
            Color::WHITE
        } else {
            Color::from_rgb(0.87, 0.87, 0.87)
        });
        iced_aw::style::tab_bar::Appearance {
            tab_label_background,
            tab_label_border_width: 0.,
            ..Default::default()
        }
    }

    fn hovered(&self, _: &Self::Style, is_active: bool) -> iced_aw::style::tab_bar::Appearance {
        let tab_label_background = Background::Color(if is_active {
            Color::WHITE
        } else {
            Color::from_rgb(0.94, 0.94, 0.94)
        });
        iced_aw::style::tab_bar::Appearance {
            tab_label_background,
            tab_label_border_width: 0.,
            ..Default::default()
        }
    }
}
