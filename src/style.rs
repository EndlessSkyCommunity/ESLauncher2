// use iced::container::Style;
use iced::widget::{button, container, Text};
use iced::{alignment, Background, Color, Font, Length, Vector};

pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../assets/IcoMoon-Free.ttf"),
};
pub const LOG_FONT: Font = Font::External {
    name: "DejaVuSansMono",
    bytes: include_bytes!("../assets/DejaVuSansMono.ttf"),
};

fn icon(unicode: char) -> Text<'static> {
    Text::new(unicode.to_string())
        .font(ICONS)
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

// pub struct Theme {}
//
// pub enum Button {
//     Icon,
//     Destructive,
//     Tab(bool),
// }
//
// impl Default for Button {
//     fn default() -> Self {
//         Self::Icon
//     }
// }
//
// impl button::StyleSheet for Button {
//     type Style = Button;
//
//     fn active(&self) -> button::Style {
//         match self {
//             Self::Icon => button::Style {
//                 text_color: Color::from_rgb(0.5, 0.5, 0.5),
//                 ..button::Style::default()
//             },
//             Self::Destructive => button::Style {
//                 background: Some(Background::Color(Color::from_rgb(0.8, 0.2, 0.2))),
//                 border_radius: 5.0,
//                 text_color: Color::WHITE,
//                 shadow_offset: Vector::new(1.0, 1.0),
//                 ..button::Style::default()
//             },
//             Self::Tab(active) => {
//                 if *active {
//                     button::Style {
//                         background: Some(Background::Color(Color::WHITE)),
//                         border_color: Color::from_rgb(0.5, 0.5, 0.5),
//                         border_width: 1.0,
//                         border_radius: 3.0,
//                         ..button::Style::default()
//                     }
//                 } else {
//                     button::Style {
//                         ..button::Style::default()
//                     }
//                 }
//             }
//         }
//     }
//
//     fn hovered(&self) -> button::Style {
//         let active = self.active();
//
//         button::Style {
//             text_color: match self {
//                 Self::Icon => Color::from_rgb(0.2, 0.2, 0.7),
//                 _ => active.text_color,
//             },
//             shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
//             ..active
//         }
//     }
// }
//
// pub struct Container {
//     background: Option<iced::Color>,
// }
//
// impl Container {
//     pub fn for_log(log: &str) -> Self {
//         Self {
//             background: if log.starts_with("WARN") {
//                 Some(iced::Color::new(1., 1., 0.5, 0.5))
//             } else if log.starts_with("ERROR") {
//                 Some(iced::Color::new(1., 0.5, 0.5, 0.5))
//             } else {
//                 None
//             },
//         }
//     }
// }
//
// impl container::StyleSheet for Container {
//     fn style(&self) -> Style {
//         iced::container::Style {
//             background: self.background.map(Background::Color),
//             ..iced::container::Style::default()
//         }
//     }
// }
