/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::operation::Operation;

mod operation;
mod wow;

fn main() -> iced::Result {
    iced::run("wow-profile-copy-ng", Operation::update, Operation::view)
}