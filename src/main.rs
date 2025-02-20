/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{path::PathBuf, ffi::OsString, fs::{self, DirEntry}, io::{self, Error, ErrorKind}};
use iced::{alignment, Element, Fill};
use iced::widget::{Container, container, scrollable, row, button, column, text};
use rfd::FileDialog;


#[derive(Default, Debug, Clone)]
struct Install {
    install_dir: OsString,
    versions: Vec<Version>
}

#[derive(Default, Debug, Clone)]
struct Version {
    name: OsString,
    wtfs: Vec<Wtf>
}

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct Wtf {
    account: OsString,
    realm: OsString,
    character: OsString,
}

// todo: change to Option<&T>
#[derive(Debug, Clone)]
struct Operation {
    install: Option<Install>,
    src_ver: Option<Version>,
    src_wtf: Option<Wtf>,
    dst_ver: Option<Version>,
    dst_wtf: Option<Wtf>,
    copy_logs: Option<Vec<String>>
}

#[derive(Debug, Clone)]
enum Message {
    Install,
    SrcVer(Version),
    SrcWtf(Wtf),
    DstVer(Version),
    DstWtf(Wtf),
    Copy,
    ResetSrc,
    ResetDst,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ver = self.name.to_str().unwrap_or_default();
        f.write_str(match ver {
            "_retail_" => "Retail",
            "_ptr_" => "Retail PTR",
            "_classic_" => "Classic",
            "_classic_era_" => "Classic Era",
            "_classic_ptr_" => "Classic PTR",
            _ => ver
        })
    }
}

impl std::fmt::Display for Wtf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {} - {}",
        self.character.to_str().unwrap_or_default(),
        self.realm.to_str().unwrap_or_default(),
        self.account.to_str().unwrap_or_default(),
    )
    }
}

impl std::default::Default for Operation {
    fn default() -> Self {
        let folder: OsString;
        if cfg!(target_os = "windows") {
            folder = OsString::from("C:\\Program Files (x86)\\World of Warcraft");
        } else if cfg!(target_os = "macos") {
            folder = OsString::from("/Applications/World of Warcraft");
        } else if cfg!(target_os = "linux") {
            folder = OsString::from("/home/brett/Games/battlenet/drive_c/Program Files (x86)/World of Warcraft");
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

        match get_wow_install(folder) {
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
    fn update(&mut self, message: Message) {
        match message {
            Message::Install => {
                let inst = prompt_folder();
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

    fn is_ready(&self) -> bool {
        self.install.is_some() 
        && self.src_ver.is_some() 
        && self.src_wtf.is_some() 
        && self.dst_ver.is_some()
        && self.dst_wtf.is_some()
    }

    fn view(&self) -> Element<Message> {
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
            text("")
        } else {
            text(
                self.copy_logs.as_ref().unwrap().join("\n")
            )
        };

        container(
            column![
                row![
                    text(install.install_dir.to_str().unwrap()).center().size(20),
                    button("Change").on_press(Message::Install)
                ].spacing(15),
                row![
                    Operation::ver_column(self, true), // source
                    Operation::ver_column(self, false) // target
                ],
                scrollable(
                    log
                ).height(100),
                row![button("Go!").padding(5).on_press(Message::Copy)]
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
                button(text(v.to_string()).center())
                .on_press(ver_msg(v.clone()))
                .width(300)
                .height(75)
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
                .map(|w| {
                    button(text(w.to_string()))
                    .on_press(wtf_msg(w.clone()))
                    .width(400)
                    .into()
                })
            )
        } else {
            column![text(ver.as_ref().unwrap().to_string()),
                    text(wtf.as_ref().unwrap().to_string()),]
        };

        let source_text = if is_source {
            text("Source").center().size(18)
        } else {
            text("Target").center().size(18)
        };

        container(
            column![
                row![source_text, button("Reset").on_press(rst_msg)].spacing(5),
                scrollable(
                    widgets.padding(20).spacing(15)
                )
            ].spacing(10)
        )
        .width(500)
        .max_height(500)
        .style(container::bordered_box)
        .padding(10)
        .align_x(alignment::Horizontal::Center)
        .into()
    }
}

fn get_wow_install(dir: OsString) -> Result<Install, io::Error> {
    let mut found_install: bool = false;
    let entries = fs::read_dir(&dir)?
        .collect::<Result<Vec<_>, io::Error>>()?;

    let mut versions: Vec<Version> = Vec::new();

    for e in entries {
        let file_name = e.file_name().into_string().unwrap();
        // version folders have names like _classic_
        if e.file_type()?.is_dir() && file_name.starts_with("_") && file_name.ends_with("_") {
            found_install = true;
            let wtfs = match get_wtf_configurations(&e) {
                Ok(wtfs) => wtfs,
                Err(error) => match error.kind() {
                    ErrorKind::NotFound => continue, // ignore missing directories, probably garbage from old installs
                    _ => Err(error),
                }?,
            };
            versions.push(Version {
                name: e.file_name(),
                wtfs: wtfs
            })
        }
    }

    if !found_install {
        return Err(Error::other("didn't find a wow install here"))
    }

    Ok(Install {
        install_dir: dir,
        versions: versions
    })
}

// read a version folder and see what character configurations are present
fn get_wtf_configurations(version: &DirEntry) -> Result<Vec<Wtf>, io::Error> {
    let mut wtfs: Vec<Wtf> = Vec::new();
    let acc = version.path().join("WTF").join("Account");
    // search all files in (version)/WTF/Account to list accounts
    let acc_entries = fs::read_dir(&acc)?
        .collect::<Result<Vec<_>, io::Error>>()?;

    for account in acc_entries {
        if account.file_type()?.is_dir() && account.file_name() != "SavedVariables" { // assume that any dir that isn't SavedVariables here is a realm
            let realm_entries = fs::read_dir(&account.path())?
            .collect::<Result<Vec<_>, io::Error>>()?;
            for realm in realm_entries {
                if realm.file_type()?.is_dir() && realm.file_name() != "SavedVariables" {
                    // any subdirectories of the realm directory are characters, they have arbitrary names
                    let char_entries = fs::read_dir(&realm.path())?
                    .collect::<Result<Vec<_>, io::Error>>()?;
                    for char in char_entries {
                        if char.file_type()?.is_dir() {
                            wtfs.push(Wtf {
                                account: account.file_name(),
                                realm: realm.file_name(),
                                character: char.file_name(),
                            })
                        }
                    }
                }
            }
        }
    }

    wtfs.sort();

    Ok(wtfs)
}

// handles prompting the user to pick their wow install directory
fn prompt_folder() -> Option<Install> {
    let folder = FileDialog::new()
    .set_title("Choose WoW Installation Directory")
    .pick_folder()?
    .into_os_string();

    match get_wow_install(folder) {
        Ok(install) => Some(install),
        Err(_) => {
            // todo: display error information dialog
            prompt_folder()
        },
    }
}

fn do_copy(op: &Operation) -> Result<Vec<String>, io::Error> {
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
                Ok(_) => format!("copied {:?}", src.as_os_str()),
                Err(e) => format!("error copying {:?}: {}", src.as_os_str(), e.to_string())
            };
            log.push(output);
        }

        // account saved variables
        // {install_dir}/WTF/Account/{account number}/SavedVariables
        let src_savedvars = src_root.join("SavedVariables");
        let dst_savedvars = dst_root.join("SavedVariables");

        let entries = fs::read_dir(&src_savedvars)?
        .collect::<Result<Vec<_>, io::Error>>()?;

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
                Ok(_) => format!("copied {:?}", src.as_os_str()),
                Err(e) => format!("error copying {:?}: {}", src.as_os_str(), e.to_string())
            };
            log.push(output);
        }

        let cache = dst_root.join("cache.md5");
        let output = match fs::remove_file(&cache) {
            Ok(_) => format!("removed {:?}", cache.as_os_str()),
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
            Ok(_) => format!("copied {:?}", dst.as_os_str()),
            Err(e) => format!("error copying {:?}: {}", dst.as_os_str(), e.to_string())
        };
        log.push(output);
    }

    // character saved variables
    let src_savedvars = src_character.join("SavedVariables");
    let dst_savedvars = dst_character.join("SavedVariables");
    
    if !dst_savedvars.try_exists()? {
        fs::create_dir_all(&dst_savedvars)?;
    }

    let entries = fs::read_dir(&src_savedvars)?
    .collect::<Result<Vec<_>, io::Error>>()?;

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
            Ok(_) => format!("copied {:?}", dst.as_os_str()),
            Err(e) => format!("error copying {:?}: {}", dst.as_os_str(), e.to_string())
        };
        log.push(output);
    }

    let cache = dst_character.join("cache.md5");
    let output = match fs::remove_file(&cache) {
        Ok(_) => format!("removed {:?}", cache.as_os_str()),
        Err(e) => format!("error removing {:?}: {}", cache.as_os_str(), e.to_string())
    };
    log.push(output);

    Ok(log)
}
fn main() -> iced::Result {
    iced::run("wow-profile-copy-ng", Operation::update, Operation::view)
}