/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::borrow::Cow;

use iced::{font, Font};

use crate::operation::Operation;

mod operation;
mod wow;

fn fonts() -> Vec<Cow<'static, [u8]>> {
    vec![
        include_bytes!("../assets/fonts/IBMPlexSerif-Regular.ttf").as_slice().into(),
        include_bytes!("../assets/fonts/IBMPlexSerif-Bold.ttf").as_slice().into(),
        include_bytes!("../assets/fonts/AnonymousPro-Regular.ttf").as_slice().into(),
        ]
}

fn main() -> iced::Result {
    let settings = iced::Settings {
        id: Some(String::from("wow-profile-copy-ng")),
        fonts: fonts(),
        default_font: Font {
            family: font::Family::Name("IBM Plex Serif"),
            weight: font::Weight::Normal,
            stretch: font::Stretch::Normal,
            style: font::Style::Normal
        },
        default_text_size: iced::Pixels(16.0),
        antialiasing: true,
        vsync: true
    };
    iced::application(Operation::default, Operation::update, Operation::view)
    .title("wow-profile-copy-ng")
    .settings(settings)
    .theme(Operation::theme)
    .run()
}