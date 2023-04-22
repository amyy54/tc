use chrono::format::ParseError;
use chrono::{offset, DateTime, Datelike, NaiveTime, TimeZone, Timelike};
use chrono_tz::TZ_VARIANTS;
use clap::{arg, Command};
use confy::ConfyError;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

const APP_NAME: &str = "tc";

#[derive(Serialize, Deserialize)]
struct SavedDefines {
    version: u8,
    timezones: Vec<String>,
}

impl ::std::default::Default for SavedDefines {
    fn default() -> Self {
        Self {
            version: 0,
            timezones: [].to_vec(),
        }
    }
}

fn cli() -> Command {
    Command::new(APP_NAME)
        .about("(T)ime (C)onverter. For those who have to constantly deal with timezones.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .subcommand(
            Command::new("u")
                .about("Turn provided time into UNIX timestamp.")
                .arg(arg!(discord: -d --discord "Format for Discord timestamp."))
                .arg(arg!(time: [TIME])),
        )
        .subcommand(
            Command::new("d")
                .about("Define timezone to include on list.")
                .subcommand(
                    Command::new("add")
                        .about("Add a new timezone to the list.")
                        .arg(arg!(timezone: [TIMEZONE])),
                )
                .subcommand(Command::new("list").about("List added timezones."))
                .subcommand(
                    Command::new("remove")
                        .about("Remove added timezone.")
                        .arg(arg!(timezone: [TIMEZONE])),
                )
                .subcommand(
                    Command::new("list-available").about("List possible timezones to add."),
                ),
        )
        .subcommand(
            Command::new("t")
                .about("Get time based on defined timezones.")
                .arg(arg!(time: [TIME]))
                .arg(arg!(timezone: [TIMEZONE])),
        )
}

fn print_defines_list() -> Result<(), ConfyError> {
    let config: SavedDefines = match confy::load(APP_NAME, None) {
        Ok(t) => t,
        Err(e) => {
            println!("Error loading config!");
            return Err(e);
        }
    };
    for timezone in config.timezones {
        println!("{}", timezone);
    }
    Ok(())
}

fn get_local_date_time(
    time_option: Option<&String>,
) -> Result<DateTime<offset::Local>, ParseError> {
    let now = offset::Local::now();

    let time = match time_option {
        Some(t) => {
            let collection: Vec<&str> = t.split(":").collect();
            match collection.len() {
                3 => NaiveTime::parse_from_str(t, "%H:%M:%S")?,
                2 => NaiveTime::parse_from_str(t, "%H:%M")?,
                1 => {
                    let newstring = collection[0].to_owned() + ":00";
                    NaiveTime::parse_from_str(&newstring, "%H:%M")?
                },
                _ => NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap(),
            }
        } // Handle if not okay.
        None => NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap(),
    };

    Ok(offset::Local
        .with_ymd_and_hms(
            now.year(),
            now.month(),
            now.day(),
            time.hour(),
            time.minute(),
            time.second(),
        )
        .unwrap())
}

fn main() -> Result<(), ParseError> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("u", sub_matches)) => {
            let datetime = match get_local_date_time(sub_matches.get_one::<String>("time")) {
                Ok(t) => t,
                Err(_e) => {
                    println!("Something went wrong when parsing the time!");
                    return Ok(());
                }
            };

            let discord_ts = match sub_matches.get_one::<bool>("discord") {
                Some(t) => *t,
                None => false,
            };

            if discord_ts {
                println!("<t:{}:t>", datetime.timestamp());
            } else {
                println!("{}", datetime.timestamp());
            }
        }
        Some(("d", sub_matches)) => match sub_matches.subcommand() {
            Some(("add", sub_matches_add)) => {
                let tz_input = match sub_matches_add.get_one::<String>("timezone") {
                    Some(t) => t,
                    None => {
                        println!("Timezone not specified!");
                        return Ok(());
                    }
                };

                if tz_input.len() > 0 {
                    let mut config: SavedDefines = match confy::load(APP_NAME, None) {
                        Ok(t) => t,
                        Err(_e) => {
                            println!("Error loading config!");
                            return Ok(());
                        }
                    };
                    for timezone in TZ_VARIANTS {
                        if tz_input
                            .to_lowercase()
                            .contains(&timezone.name().to_lowercase())
                        {
                            let tz_name = String::from_str(timezone.name()).unwrap();
                            if config.timezones.contains(&tz_name) {
                                println!("Already exists in list!");
                                return Ok(());
                            }
                            config.timezones.push(tz_name);
                            match confy::store(APP_NAME, None, &config) {
                                Ok(_t) => "",
                                Err(_e) => {
                                    println!("Error saving config!");
                                    return Ok(());
                                }
                            };
                            println!("Added timezone {}", timezone.name());
                            return Ok(());
                        }
                    }
                    println!("Timezone not found!");
                }
            }
            Some(("list", _)) => {
                match print_defines_list() {
                    Ok(t) => return Ok(t),
                    Err(_e) => return Ok(())
                };
            }
            Some(("remove", sub_matches_remove)) => {
                let tz_input = match sub_matches_remove.get_one::<String>("timezone") {
                    Some(t) => t,
                    None => {
                        println!("Timezone not specified!");
                        return Ok(());
                    }
                };

                if tz_input.len() > 0 {
                    let mut config: SavedDefines = match confy::load(APP_NAME, None) {
                        Ok(t) => t,
                        Err(_e) => {
                            println!("Error loading config!");
                            return Ok(());
                        }
                    };
                    let mut found = false;
                    for (i, timezone) in config.timezones.clone().into_iter().enumerate() {
                        if tz_input.contains(&timezone) {
                            config.timezones.remove(i);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        println!("Timezone not found saved in config!");
                        return Ok(());
                    }
                    match confy::store(APP_NAME, None, &config) {
                        Ok(_t) => "",
                        Err(_e) => {
                            println!("Error saving config!");
                            return Ok(());
                        }
                    };
                    println!("Removed timezone {}", tz_input);
                }
            }
            Some(("list-available", _)) => {
                for timezone in TZ_VARIANTS {
                    println!("{}", timezone.name());
                }
            }
            Some((&_, _)) => {
                println!("Invalid Command!");
            }
            None => {
                match print_defines_list() {
                    Ok(t) => return Ok(t),
                    Err(_e) => return Ok(())
                };
            }
        },
        Some(("t", sub_matches)) => {
            let config: SavedDefines = match confy::load(APP_NAME, None) {
                Ok(t) => t,
                Err(_e) => {
                    println!("Error loading config!");
                    return Ok(());
                }
            };
            let offset_local_datetime =
                match get_local_date_time(sub_matches.get_one::<String>("time")) {
                    Ok(t) => t,
                    Err(_e) => {
                        println!("Something went wrong when parsing the time!");
                        return Ok(());
                    }
                };

            println!("Time: {}\n", offset_local_datetime.time());

            let mut tz_list: Vec<(String, String, String, i64)> = [].to_vec();

            for timezone in TZ_VARIANTS {
                let tz_name = String::from_str(timezone.name()).unwrap();
                if config.timezones.contains(&tz_name) {
                    let converted_time = offset_local_datetime.with_timezone(&timezone);

                    let mut offset_string: String;
                    if converted_time.day() != offset_local_datetime.day() {
                        let day_diff: u32;
                        if converted_time.day() > offset_local_datetime.day() {
                            day_diff = converted_time.day() - offset_local_datetime.day();
                        } else {
                            day_diff = offset_local_datetime.day() - converted_time.day();
                        }
                        offset_string = format!("(+{}", day_diff);
                        if day_diff == 1 {
                            offset_string += " day)";
                        } else {
                            offset_string += " days)";
                        }
                    } else {
                        offset_string = "".to_string();
                    }
                    tz_list.push((
                        tz_name,
                        converted_time.time().to_string(),
                        offset_string,
                        converted_time.naive_local().timestamp(),
                    ));
                }
            }

            tz_list.sort_by_key(|k| k.3);
            for item in tz_list {
                println!("{} : {} {}", item.0, item.1, item.2);
            }
        }
        Some((&_, _)) => {
            println!("Invalid Command!");
        }
        None => {
            println!("Invalid Command!");
        }
    };

    Ok(())
}
