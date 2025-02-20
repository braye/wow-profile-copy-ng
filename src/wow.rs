/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::{ffi::OsString, io::{self, Error, ErrorKind}, fs::{self, DirEntry}};
use rfd::FileDialog;

#[derive(Default, Debug, Clone)]
pub struct Install {
    pub install_dir: OsString,
    pub versions: Vec<Version>
}

#[derive(Default, Debug, Clone)]
pub struct Version {
    pub name: OsString,
    pub wtfs: Vec<Wtf>
}

#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct Wtf {
    pub account: OsString,
    pub realm: OsString,
    pub character: OsString,
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
        write!(f, "{} - {}",
        self.character.to_str().unwrap_or_default(),
        self.realm.to_str().unwrap_or_default()
    )
    }
}

// tries reading a directory and finding information about a WoW install
// errors if the directory doesn't appear to contain a WoW install
pub fn get_wow_install(dir: OsString) -> Result<Install, io::Error> {
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
pub fn prompt_folder() -> Option<Install> {
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
