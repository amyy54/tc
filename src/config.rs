use chrono_tz::TZ_VARIANTS;
use confy::ConfyError;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

const APP_NAME: &str = "tc";

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedTimezones {
    pub timezone_name: String,
    pub nickname: Option<String>,
    pub separator: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct SavedTimezonesV1 {
    timezone_name: String,
    nickname: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedDefines {
    pub version: u8,
    pub timezones: Vec<SavedTimezones>,
}

#[derive(Serialize, Deserialize, Default)]
struct SavedDefinesV1 {
    version: u8,
    timezones: Vec<SavedTimezonesV1>,
}

#[derive(Serialize, Deserialize, Default)]
struct SavedDefinesV0 {
    version: u8,
    timezones: Vec<String>,
}

impl ::std::default::Default for SavedDefines {
    fn default() -> Self {
        Self {
            version: 2,
            timezones: [].to_vec(),
        }
    }
}

pub fn load_config() -> Result<SavedDefines, ConfyError> {
    let config: SavedDefines = match confy::load(APP_NAME, None) {
        Ok(t) => t,
        Err(_e) => {
            // ! Migrating configs is really annoying. There is surely a better way of doing it. For now... enjoy :D
            eprintln!("Older config found, updating config.");
            let v1: SavedDefinesV1 = match confy::load(APP_NAME, None) {
                Ok(t) => t,
                Err(_e) => {
                    let v0: SavedDefinesV0 = match confy::load(APP_NAME, None) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("Error loading config!");
                            return Err(e);
                        }
                    };
                    let mut new_tz_list: Vec<SavedTimezonesV1> = [].to_vec();
                    for timezone in v0.timezones {
                        let new = SavedTimezonesV1 {
                            timezone_name: timezone,
                            nickname: None,
                        };
                        new_tz_list.push(new);
                    }
                    let new_config = SavedDefinesV1 {
                        version: 1,
                        timezones: new_tz_list,
                    };
                    new_config
                }
            };
            let mut new_tz_list: Vec<SavedTimezones> = [].to_vec();
            for timezone in v1.timezones {
                let new = SavedTimezones {
                    timezone_name: timezone.timezone_name,
                    nickname: timezone.nickname,
                    separator: false,
                };
                new_tz_list.push(new);
            }
            let new_config = SavedDefines {
                version: 2,
                timezones: new_tz_list,
            };
            match confy::store(APP_NAME, None, &new_config) {
                Ok(_t) => eprintln!("Update successful, continuing."),
                Err(e) => {
                    eprintln!("Error saving config!");
                    return Err(e);
                }
            };
            new_config
        }
    };
    Ok(config)
}

pub fn saved_list_contains_timezone(defines: &SavedDefines, tz_name: &String) -> (i32, bool) {
    let mut res = false;
    let mut index: i32 = -1;
    for (i, timezone) in defines.timezones.clone().iter().enumerate() {
        if timezone.timezone_name == tz_name.clone() {
            res = true;
            index = i as i32;
            break;
        }
    }
    (index, res)
}

pub fn add_timezone(tz_input: String) -> Option<String> {
    let mut config = match load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };
    for timezone in TZ_VARIANTS {
        if tz_input
            .to_lowercase()
            .contains(&timezone.name().to_lowercase())
        {
            let tz_name = String::from_str(timezone.name()).unwrap();
            if saved_list_contains_timezone(&config, &tz_name).1 {
                eprintln!("Already exists in list!");
                return None;
            }
            let new_timezone = SavedTimezones {
                timezone_name: tz_name.clone(),
                nickname: None,
                separator: false,
            };
            config.timezones.push(new_timezone);
            match confy::store(APP_NAME, None, &config) {
                Ok(_t) => "",
                Err(_e) => {
                    eprintln!("Error saving config!");
                    return None;
                }
            };

            return Some(timezone.name().to_string());
        }
    }
    eprintln!("Timezone not found!");
    return None;
}

pub fn add_nick_to_timezone(tz_input: String, nickname: String) -> Option<String> {
    let mut config = match load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };
    let mut found = false;
    for (i, timezone) in config.timezones.clone().into_iter().enumerate() {
        if tz_input.contains(&timezone.timezone_name) {
            if nickname.len() == 0 {
                config.timezones[i].nickname = None;
            } else {
                config.timezones[i].nickname = Some(nickname);
            }
            found = true;
            break;
        }
    }
    if !found {
        eprintln!("Timezone not found saved in config!");
        return None;
    }
    match confy::store(APP_NAME, None, &config) {
        Ok(_t) => "",
        Err(_e) => {
            eprintln!("Error saving config!");
            return None;
        }
    };

    Some(tz_input)
}

pub fn add_sep_to_timezone(tz_input: String) -> Option<String> {
    let mut config = match load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };
    let mut found = false;
    for (i, timezone) in config.timezones.clone().into_iter().enumerate() {
        if tz_input.contains(&timezone.timezone_name) {
            config.timezones[i].separator = !config.timezones[i].separator;
            found = true;
            break;
        }
    }
    if !found {
        eprintln!("Timezone not found saved in config!");
        return None;
    }
    match confy::store(APP_NAME, None, &config) {
        Ok(_t) => "",
        Err(_e) => {
            eprintln!("Error saving config!");
            return None;
        }
    };
    return Some(tz_input);
}

pub fn remove_timezone(tz_input: String) -> Option<String> {
    let mut config = match load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };
    let mut found = false;
    for (i, timezone) in config.timezones.clone().into_iter().enumerate() {
        if tz_input.contains(&timezone.timezone_name) {
            config.timezones.remove(i);
            found = true;
            break;
        }
    }
    if !found {
        eprintln!("Timezone not found saved in config!");
        return None;
    }
    match confy::store(APP_NAME, None, &config) {
        Ok(_t) => "",
        Err(_e) => {
            eprintln!("Error saving config!");
            return None;
        }
    };
    return Some(tz_input);
}
