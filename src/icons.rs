use iced::{
    widget::{button, container, text, tooltip},
    Element, Font,
};

use crate::Message;

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

pub fn action<'a>(
    content: Element<'a, Message>,
    label: &'a str,
    on_press: Option<Message>,
    home: bool,
) -> Element<'a, Message> {
    let action = button(container(content).width(if home {15} else {30}).center_x());

    if let Some(on_press) = on_press {
        tooltip(
            action.on_press(on_press),
            label,
            tooltip::Position::FollowCursor,
        )
        .style(container::rounded_box)
        .into()
    } else {
        action.style(button::secondary).into()
    }
}

pub fn new_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e800}')
}

pub fn save_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e801}')
}

pub fn open_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0f115}')
}

pub fn settings_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e803}')
}

pub fn home_icon<'a, Message>() -> Element<'a, Message> {
    icon('\u{0e804}')
}

// pub fn history_icon<'a, Message>() -> Element<'a, Message> {
//     icon('\u{0f1da}')
// }

