use iced::border::Radius;
use iced::widget::{button, container, Text};
use iced::{alignment, Background, Border, Color, Font, Length, Shadow, Vector};
use iced_aw::tab_bar;

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

// TODO: This is whack. Status is hardcoded, and generally this doesn't feel like intended
pub fn icon_button() -> button::Style {
    use button::Catalog;
    ButtonStyle::Icon.style(&ButtonStyle::Icon, button::Status::Active)
}
pub fn text_button() -> button::Style {
    use button::Catalog;
    ButtonStyle::Text.style(&ButtonStyle::Text, button::Status::Active)
}

pub fn tab_bar() -> tab_bar::Style {
    use tab_bar::Catalog;
    CustomTabBar::default().style(&CustomTabBar::default(), tab_bar::Status::Active)
}

pub fn log_container(log: &str) -> container::Style {
    use container::Catalog;
    LogContainer::from(log).style(&LogContainer::default())
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
        let active = self.style(class, button::Status::Active);
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
                Some(Color::new(1., 1., 0.5, 0.5))
            } else if log.starts_with("ERROR") {
                Some(Color::new(1., 0.5, 0.5, 0.5))
            } else {
                None
            },
        }
    }
}

impl container::Catalog for LogContainer {
    type Class<'a> = LogContainer;

    fn default<'a>() -> Self::Class<'a> {
        LogContainer::default()
    }

    fn style(&self, _: &Self::Class<'_>) -> container::Style {
        container::Style {
            text_color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
            background: self.background.map(Background::Color),
            ..Default::default()
        }
    }
}

pub struct CustomTabBar;

impl tab_bar::Catalog for CustomTabBar {
    type Class<'a> = CustomTabBar;

    fn default<'a>() -> Self::Class<'a> {
        CustomTabBar::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: tab_bar::Status) -> tab_bar::Style {
        match status {
            tab_bar::Status::Active => tab_bar::Style {
                tab_label_background: Background::Color(Color::from_rgb(0.87, 0.87, 0.87)),
                tab_label_border_width: 0.,
                ..Default::default()
            },
            tab_bar::Status::Hovered => tab_bar::Style {
                tab_label_background: Background::Color(Color::from_rgb(0.94, 0.94, 0.94)),
                tab_label_border_width: 0.,
                ..Default::default()
            },
            _ => Default::default(),
        }
    }
}
