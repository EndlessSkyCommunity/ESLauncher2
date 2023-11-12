use iced::widget::{button, container, Text};
use iced::{alignment, Background, BorderRadius, Color, Font, Length, Theme, Vector};

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

pub fn tab_button(active: bool) -> iced::theme::Button {
    iced::theme::Button::Custom(Box::new(ButtonStyle::Tab(active)))
}

pub fn log_container(log: &str) -> iced::theme::Container {
    iced::theme::Container::Custom(Box::new(LogContainer::from(log)))
}

pub enum ButtonStyle {
    Icon,
    Tab(bool),
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        match self {
            ButtonStyle::Icon => button::Appearance {
                text_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..button::Appearance::default()
            },
            Self::Tab(active) => {
                if *active {
                    button::Appearance {
                        background: Some(Background::Color(Color::WHITE)),
                        border_color: Color::from_rgb(0.5, 0.5, 0.5),
                        border_width: 1.0,
                        border_radius: BorderRadius::from(3.0),
                        ..button::Appearance::default()
                    }
                } else {
                    button::Appearance {
                        ..button::Appearance::default()
                    }
                }
            }
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let active = self.active(style);

        button::Appearance {
            text_color: match self {
                Self::Icon => Color::from_rgb(0.2, 0.2, 0.7),
                _ => active.text_color,
            },
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
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
            ..container::Appearance::default()
        }
    }
}
