/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::wow::{self, Install, Version, Wtf};
use iced::{alignment, border, Element, Fill, FillPortion, Font, Theme};
use iced::widget::{Container, container, scrollable, row, button, column, text};
use std::{env, path::PathBuf, ffi::OsString, fs, io::Error};
use dark_light;


// todo: change to Option<&T>
#[derive(Debug, Clone)]
pub struct Operation {
    install: Option<Install>,
    src_ver: Option<Version>,
    src_wtf: Option<Wtf>,
    dst_ver: Option<Version>,
    dst_wtf: Option<Wtf>,
    copy_logs: Option<Vec<String>>
}

#[derive(Debug, Clone)]
pub enum Message {
    Install,
    SrcVer(Version),
    SrcWtf(Wtf),
    DstVer(Version),
    DstWtf(Wtf),
    Copy,
    ResetSrc,
    ResetDst,
}


impl std::default::Default for Operation {
    fn default() -> Self {
        let folder: OsString;
        if cfg!(target_os = "windows") {
            folder = OsString::from("C:\\Program Files (x86)\\World of Warcraft");
        } else if cfg!(target_os = "macos") {
            folder = OsString::from("/Applications/World of Warcraft");
        } else if cfg!(target_os = "linux") {
            let home = env::var_os("HOME");
            if home.is_none() {
                return Operation {
                    install: None,
                    src_ver: None,
                    dst_ver: None,
                    src_wtf: None,
                    dst_wtf: None,
                    copy_logs: None,
                }
            }
            folder = PathBuf::from(home.unwrap())
                .join("Games/battlenet/drive_c/Program Files (x86)/World of Warcraft")
                .into_os_string();
        } else {
            return Operation {
                install: None,
                src_ver: None,
                dst_ver: None,
                src_wtf: None,
                dst_wtf: None,
                copy_logs: None,
            }
        }

        match wow::get_wow_install(folder) {
            Ok(install) => {
                Operation {
                    install: Some(install),
                    src_ver: None,
                    dst_ver: None,
                    src_wtf: None,
                    dst_wtf: None,
                    copy_logs: None,
                }
            }
            Err(_) => {
                Operation {
                    install: None,
                    src_ver: None,
                    dst_ver: None,
                    src_wtf: None,
                    dst_wtf: None,
                    copy_logs: None,
                }
            }
        }
    }
}

impl Operation {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Install => {
                let inst = wow::prompt_folder();
                if inst.is_some() {
                    self.install = inst;
                    self.src_ver = None;
                    self.dst_ver = None;
                    self.src_wtf = None;
                    self.dst_wtf = None;
                }
            },
            Message::ResetSrc => {
                self.src_ver = None;
                self.src_wtf = None;
            },
            Message::ResetDst => {
                self.dst_ver = None;
                self.dst_wtf = None;
            },
            Message::SrcVer(ver) => self.src_ver = Some(ver),
            Message::DstVer(ver) => self.dst_ver = Some(ver),
            Message::SrcWtf(wtf) => self.src_wtf = Some(wtf),
            Message::DstWtf(wtf) => self.dst_wtf = Some(wtf),
            Message::Copy => {
                match do_copy(self) {
                    Ok(l) => self.copy_logs = Some(l),
                    // todo: show error dialog, rewind directory state
                    Err(e) => {
                        self.copy_logs = Some(vec![e.to_string()]);
                    }
                }
            },
        }
    }

    pub fn theme(&self) -> Theme  {
        // there's probably a way to do this without this crate but i'm feeling lazy
        match dark_light::detect().unwrap_or_else(|_e| dark_light::Mode::Unspecified) {
            dark_light::Mode::Dark => Theme::SolarizedDark,
            dark_light::Mode::Light => Theme::SolarizedLight,
            dark_light::Mode::Unspecified => Theme::SolarizedDark,
        }
    }

    fn is_ready(&self) -> bool {
        self.install.is_some() 
        && self.src_ver.is_some() 
        && self.src_wtf.is_some() 
        && self.dst_ver.is_some()
        && self.dst_wtf.is_some()
    }

    pub fn view(&self) -> Element<Message> {
        if self.install.is_none() {
            return container(
                column![
                button(text("Select WoW Install Directory"))
                .on_press(Message::Install)
            ]
            .spacing(10))
            .padding(10)
            .center_x(Fill)
            .center_y(Fill)
            .into()
        }

        let install = self.install.as_ref().unwrap();

        let log = if self.copy_logs.is_none() {
            scrollable(text(""))
        } else {
            scrollable(
                text(
                    self.copy_logs.as_ref().unwrap().join("\n")
                ).font(Font::with_name("B612 Mono"))
            )
        };

        // todo: clean this mess up
        container(
            column![
                column![
                    text("Installation Folder: ".to_owned() + install.install_dir.to_str().unwrap()).center(),
                    button("Change").on_press(Message::Install)
                ].spacing(15),
                row![
                    Operation::ver_column(self, true).width(FillPortion(2)), // source
                    Operation::ver_column(self, false).width(FillPortion(2)) // target
                ].height(FillPortion(7)),
                container(column![text("Logs"), log]).padding(8).style(|theme: &Theme| {
                    container::Style::default()
                        .border(border::color(theme.palette().primary).width(2).rounded(5))
                }).height(FillPortion(2)).width(Fill),
                row![button("Go!").padding(5).on_press(Message::Copy).style(button::success)]
            ]
            .spacing(10)
        )
        .padding(10)
        .center(Fill)
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn ver_column(&self, is_source: bool) -> Container<Message> {
        //todo: clean this up, tuple return/destructuring assignment?
        let ver = if is_source {
            &self.src_ver
        } else {
            &self.dst_ver
        };

        let wtf = if is_source {
            &self.src_wtf
        } else {
            &self.dst_wtf
        };

        let ver_msg = if is_source {
            Message::SrcVer
        } else {
            Message::DstVer
        };

        let wtf_msg = if is_source {
            Message::SrcWtf
        } else {
            Message::DstWtf
        };

        let rst_msg = if is_source {
            Message::ResetSrc
        } else {
            Message::ResetDst
        };

        let install = self.install.as_ref().unwrap();

        let widgets = if ver.is_none() {
            column(install.versions.iter().map(|v| {
                button(text(v.to_string()).width(Fill).center())
                .on_press(ver_msg(v.clone()))
                .height(50)
                .into()
            }))
        } else if wtf.is_none() {
            column(
                install.versions
                .iter()
                .find(|v| v.to_string() == ver.as_ref().unwrap().to_string())
                .unwrap()
                .wtfs
                .iter()
                .filter(|w| {
                    println!("{}: {is_source} - {}", w, w.has_vars);
                    !is_source || (is_source && w.has_vars)
                })
                .map(|w| {
                    button(text(w.to_string()).width(Fill).center())
                    .on_press(wtf_msg(w.clone()))
                    .into()
                })
            )
        } else {
            column![text(ver.as_ref().unwrap().to_string()),
                    text(wtf.as_ref().unwrap().to_string()),]
        };

        let source_text = if is_source {
            text("Source")
        } else {
            text("Target")
        };

        container(
            column![
                source_text,
                scrollable(
                    widgets.padding(20).spacing(15)
                ).height(FillPortion(9)),
                button("Reset").on_press(rst_msg)
            ].spacing(10).width(Fill).height(Fill)
        )
        .width(Fill)
        .height(Fill)
        .style(container::bordered_box)
        .padding(10)
        .align_x(alignment::Horizontal::Center)
        .into()
    }
}

// does the actual copying of config files and savedvariables for a given install, source, and destination
fn do_copy(op: &Operation) -> Result<Vec<String>, Error> {
    if !op.is_ready() {
        return Err(Error::other("operation not ready for copying!"))
    }

    let mut log: Vec<String> = vec![];
    let account_files: [&str; 4] = ["bindings-cache.wtf", "config-cache.wtf", "macros-cache.txt", "edit-mode-cache-account.txt"];
    let character_files: [&str; 5] = ["AddOns.txt", "config-cache.wtf", "layout-local.txt", "macros-cache.txt", "edit-mode-cache-character.txt"];

    let src_account = &op.src_wtf.as_ref().unwrap().account;
    let dst_account = &op.dst_wtf.as_ref().unwrap().account;

    let src_root = PathBuf::from(&op.install.as_ref().unwrap().install_dir)
        .join(&op.src_ver.as_ref().unwrap().name)
        .join("WTF")
        .join("Account")
        .join(&src_account);

    let dst_root = PathBuf::from(&op.install.as_ref().unwrap().install_dir)
        .join(&op.dst_ver.as_ref().unwrap().name)
        .join("WTF")
        .join("Account")
        .join(&dst_account);

    if src_account == dst_account {
        log.push(String::from("skipping account copy - accounts are the same"));
    } else {
        // client configuration
        for file in account_files {
            let src = src_root.join(file);
            let dst = dst_root.join(file);
            let output = match fs::copy(&src, &dst) {
                Ok(_) => format!("copied {:?}", src.file_name().unwrap_or_default()),
                Err(e) => format!("error copying {:?}: {}", src.as_os_str(), e.to_string())
            };
            log.push(output);
        }

        // account saved variables
        // {install_dir}/WTF/Account/{account number}/SavedVariables
        let src_savedvars = src_root.join("SavedVariables");
        let dst_savedvars = dst_root.join("SavedVariables");

        let entries = fs::read_dir(&src_savedvars)?
        .collect::<Result<Vec<_>, Error>>()?;

        for e in entries {
            match e.path().extension(){
                Some(n) => {
                    if n.to_str().unwrap_or("") != "lua" {
                        continue
                    }
                    n
                },
                None => continue
            };
            let src = src_savedvars.join(e.file_name());
            let dst = dst_savedvars.join(e.file_name());
            let output = match fs::copy(&src, &dst) {
                Ok(_) => format!("copied {:?}", src.file_name().unwrap_or_default()),
                Err(e) => format!("error copying {:?}: {}", src.as_os_str(), e.to_string())
            };
            log.push(output);
        }

        let cache = dst_root.join("cache.md5");
        let output = match fs::remove_file(&cache) {
            Ok(_) => format!("removed {:?}", cache.file_name().unwrap_or_default()),
            Err(e) => format!("error removing {:?}: {}", cache.as_os_str(), e.to_string())
        };
        log.push(output);
    }

    // character configuration
    // {install}/WTF/Account/{account}/{realm}/{character}

    let src_character = src_root
    .join(&op.src_wtf.as_ref().unwrap().realm)
    .join(&op.src_wtf.as_ref().unwrap().character);

    let dst_character = dst_root
    .join(&op.dst_wtf.as_ref().unwrap().realm)
    .join(&op.dst_wtf.as_ref().unwrap().character);

    for file in character_files {
        let src = src_character.join(file);
        let dst = dst_character.join(file);
        let output = match fs::copy(&src, &dst) {
            Ok(_) => format!("copied {:?}", dst.file_name().unwrap_or_default()),
            Err(e) => format!("error copying {:?}: {}", dst.as_os_str(), e.to_string())
        };
        log.push(output);
    }

    // character saved variables
    let src_savedvars = src_character.join("SavedVariables");
    let dst_savedvars = dst_character.join("SavedVariables");
    
    if !dst_savedvars.try_exists()? {
        log.push(format!("destination savedvariables dir missing, creating: {:?}", dst_savedvars.as_os_str()));
        fs::create_dir_all(&dst_savedvars)?;
    }

    let entries = fs::read_dir(&src_savedvars)?
    .collect::<Result<Vec<_>, Error>>()?;

    for e in entries {
        match e.path().extension(){
            Some(n) => {
                if n.to_str().unwrap_or("") != "lua" {
                    continue
                }
                n
            },
            None => continue
        };
        let src = src_savedvars.join(e.file_name());
        let dst = dst_savedvars.join(e.file_name());
        let output = match fs::copy(&src, &dst) {
            Ok(_) => format!("copied {:?}", dst.file_name().unwrap_or_default()),
            Err(e) => format!("error copying {:?}: {}", dst.as_os_str(), e.to_string())
        };
        log.push(output);
    }

    let cache = dst_character.join("cache.md5");
    let output = match fs::remove_file(&cache) {
        Ok(_) => format!("removed {:?}", cache.file_name().unwrap_or_default()),
        Err(e) => format!("error removing {:?}: {}", cache.as_os_str(), e.to_string())
    };
    log.push(output);

    Ok(log)
}