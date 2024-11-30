use chrono::format::ParseError;
use chrono::{offset, DateTime, Datelike, Local, NaiveTime, TimeZone, Timelike};
use chrono_tz::{Tz, TZ_VARIANTS};
use clap::{arg, ArgMatches, Command};
use confy::ConfyError;
use pancurses::{endwin, initscr, Input};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

const APP_NAME: &str = "tc";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone)]
struct SavedTimezones {
    timezone_name: String,
    nickname: Option<String>,
    separator: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct SavedTimezonesV1 {
    timezone_name: String,
    nickname: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct SavedDefines {
    version: u8,
    timezones: Vec<SavedTimezones>,
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

#[derive(PartialEq)]
enum CurTimeKind {
    Local,
    Tz,
}

struct CurTime {
    kind: CurTimeKind,
    local_time: Option<DateTime<Local>>,
    tz_time: Option<DateTime<Tz>>,
}

#[derive(Serialize, Clone)]
struct OutputTime {
    timezone_name: String,
    timezone_nickname: Option<String>,
    displayed_name: String,
    day_offset: u32,
    day_offset_str: String,
    timestamp: i64,
    timestring: String,
    separator: bool,
}

impl ::std::default::Default for SavedDefines {
    fn default() -> Self {
        Self {
            version: 2,
            timezones: [].to_vec(),
        }
    }
}

fn cli() -> Command {
    Command::new(APP_NAME)
        .about("(T)ime (C)onverter. For those who have to constantly deal with timezones.")
        .subcommand(
            Command::new("t")
                .about("Default - Get time based on defined timezones")
                .arg(arg!(time: [TIME]))
                .arg(arg!(timezone: -t --timezone [TIMEZONE] "Offset by timezone"))
                .arg(
                    arg!(output: -o --output [OUTPUT] "Set output format")
                        .value_parser(["pretty", "json", "json_pretty", "csv"])
                        .default_value("pretty")
                        .default_missing_value("pretty"),
                )
                .arg(arg!(curses: -c --curses "Keep active and looping with curses")),
        )
        .subcommand(
            Command::new("d")
                .about("Define timezone to include on list")
                .subcommand(
                    Command::new("add")
                        .about("Add a new timezone to the list")
                        .arg(arg!(timezone: [TIMEZONE])),
                )
                .subcommand(
                    Command::new("nick")
                        .about("Add a nickname to a timezone")
                        .arg(arg!(timezone: [TIMEZONE]))
                        .arg(arg!(nickname: [NICKNAME] "Leave blank to clear nickname")),
                )
                .subcommand(
                    Command::new("sep")
                        .about(
                            "Add a separator after the provided timezone when using pretty output",
                        )
                        .arg(arg!(timezone: [TIMEZONE])),
                )
                .subcommand(Command::new("list").about("List added timezones"))
                .subcommand(
                    Command::new("remove")
                        .about("Remove added timezone")
                        .arg(arg!(timezone: [TIMEZONE])),
                )
                .subcommand(Command::new("list-available").about("List possible timezones to add")),
        )
        .subcommand(
            Command::new("u")
                .about("Turn provided time into UNIX timestamp")
                .arg(arg!(discord: -d --discord "Format for Discord timestamp"))
                .arg(arg!(time: [TIME])),
        )
        .arg(arg!(version: --version "Print version"))
}

fn load_config() -> Result<SavedDefines, ConfyError> {
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

fn get_nick_string(nick: Option<String>) -> String {
    match nick {
        Some(t) => t,
        None => "".to_string(),
    }
}

fn print_defines_list() -> Result<(), ConfyError> {
    let config = match load_config() {
        Ok(t) => t,
        Err(e) => {
            return Err(e);
        }
    };
    for timezone in config.timezones {
        println!(
            "{0: <25} {1}",
            timezone.timezone_name,
            get_nick_string(timezone.nickname)
        );
    }
    Ok(())
}

fn saved_list_contains_timezone(defines: &SavedDefines, tz_name: &String) -> (i32, bool) {
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

fn tz_offset_from_local_time(time: NaiveTime, now: DateTime<Local>, tz: Option<Tz>) -> NaiveTime {
    match tz {
        Some(t) => {
            let datetime = offset::Local
                .with_ymd_and_hms(
                    now.year(),
                    now.month(),
                    now.day(),
                    time.hour(),
                    time.minute(),
                    time.second(),
                )
                .unwrap()
                .with_timezone(&t);

            NaiveTime::from_hms_opt(datetime.hour(), datetime.minute(), datetime.second()).unwrap()
        }
        None => time,
    }
}

fn get_comparison_date_time(
    time_option: Option<&String>,
    tz: Option<Tz>,
) -> Result<CurTime, ParseError> {
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
                }
                _ => tz_offset_from_local_time(
                    NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap(),
                    now,
                    tz,
                ),
            }
        } // Handle if not okay.
        None => tz_offset_from_local_time(
            NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap(),
            now,
            tz,
        ),
    };

    let mut res = CurTime {
        kind: CurTimeKind::Local,
        local_time: None,
        tz_time: None,
    };

    match tz {
        Some(t) => {
            res.tz_time = Some(
                t.with_ymd_and_hms(
                    now.year(),
                    now.month(),
                    now.day(),
                    time.hour(),
                    time.minute(),
                    time.second(),
                )
                .unwrap(),
            );
            res.kind = CurTimeKind::Tz;
        }
        None => {
            res.local_time = Some(
                offset::Local
                    .with_ymd_and_hms(
                        now.year(),
                        now.month(),
                        now.day(),
                        time.hour(),
                        time.minute(),
                        time.second(),
                    )
                    .unwrap(),
            );
        }
    }

    Ok(res)
}

fn convert_date_to_timestamp(year: i32, ordinal: u32) -> u32 {
    ordinal + ((year - 1970) * 365) as u32
}

fn t_command(sub_matches: Option<&ArgMatches>) -> Option<String> {
    let config = match load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };

    let mut output: String = "".to_owned();

    let output_file: String = match sub_matches {
        Some(val) => match val.get_one::<String>("output") {
            Some(t) => t.to_string(),
            None => "pretty".to_owned(),
        },
        None => "pretty".to_owned(),
    };

    let timezone: Option<Tz> = match sub_matches {
        Some(val) => match val.get_one::<String>("timezone") {
            Some(t) => {
                let mut tz_input = t.clone();
                let mut res: Option<Tz> = None;
                for timezone in config.timezones.clone() {
                    match timezone.nickname {
                        Some(nick) => {
                            if nick.to_lowercase().contains(&t.to_lowercase()) {
                                tz_input = timezone.timezone_name.clone();
                                break;
                            }
                        }
                        None => {
                            continue;
                        }
                    }
                }
                for timezone in TZ_VARIANTS {
                    let tz_name = String::from_str(timezone.name()).unwrap();
                    if saved_list_contains_timezone(&config, &tz_name).1 {
                        if tz_name.contains(&tz_input) {
                            res = Some(timezone);
                            break;
                        }
                    }
                }
                res
            }
            None => None,
        },
        None => None,
    };

    let time_val = match sub_matches {
        Some(val) => val.get_one::<String>("time"),
        None => None,
    };

    let offset_comparison_datetime = match get_comparison_date_time(time_val, timezone) {
        Ok(t) => t,
        Err(_e) => {
            eprintln!("Something went wrong when parsing the time!");
            return None;
        }
    };

    if offset_comparison_datetime.kind == CurTimeKind::Tz {
        let time = offset_comparison_datetime.tz_time.unwrap();
        let fmt_string = "Time for ".to_owned() + time.timezone().name();
        if output_file == "pretty" {
            output += &format!("{0: <25} {1}\n\n", fmt_string, time.time());
        }
    } else {
        let time = offset_comparison_datetime.local_time.unwrap();
        let fmt_string = "Local Time".to_owned();
        if output_file == "pretty" {
            output += &format!("{0: <25} {1}\n\n", fmt_string, time.time());
        }
    }

    let mut tz_list: Vec<OutputTime> = [].to_vec();

    for timezone in TZ_VARIANTS {
        let tz_name = String::from_str(timezone.name()).unwrap();
        let contains = saved_list_contains_timezone(&config, &tz_name);
        if contains.1 {
            let converted_time: DateTime<Tz>;
            if offset_comparison_datetime.kind == CurTimeKind::Tz {
                let time = offset_comparison_datetime.tz_time.unwrap();
                converted_time = time.with_timezone(&timezone);
            } else {
                let time = offset_comparison_datetime.local_time.unwrap();
                converted_time = time.with_timezone(&timezone);
            }

            let mut offset_string: String;
            let mut day_diff: u32 = 0;
            if offset_comparison_datetime.kind == CurTimeKind::Tz {
                let offset_time = offset_comparison_datetime.tz_time.unwrap();

                if converted_time.day() != offset_time.day() {
                    let converted_ts =
                        convert_date_to_timestamp(converted_time.year(), converted_time.ordinal0());
                    let local_ts =
                        convert_date_to_timestamp(offset_time.year(), offset_time.ordinal0());
                    if converted_ts > local_ts {
                        day_diff = converted_ts - local_ts;
                        offset_string = format!("(+{}", day_diff);
                    } else {
                        day_diff = local_ts - converted_ts;
                        offset_string = format!("(-{}", day_diff);
                    }
                    if day_diff == 1 {
                        offset_string += " day)";
                    } else {
                        offset_string += " days)";
                    }
                } else {
                    offset_string = "".to_string();
                }
            } else {
                let offset_time = offset_comparison_datetime.local_time.unwrap();

                if converted_time.day() != offset_time.day() {
                    let converted_ts =
                        convert_date_to_timestamp(converted_time.year(), converted_time.ordinal0());
                    let local_ts =
                        convert_date_to_timestamp(offset_time.year(), offset_time.ordinal0());
                    if converted_ts > local_ts {
                        day_diff = converted_ts - local_ts;
                        offset_string = format!("(+{}", day_diff);
                    } else {
                        day_diff = local_ts - converted_ts;
                        offset_string = format!("(-{}", day_diff);
                    }
                    if day_diff == 1 {
                        offset_string += " day)";
                    } else {
                        offset_string += " days)";
                    }
                } else {
                    offset_string = "".to_string();
                }
            }
            tz_list.push(OutputTime {
                timezone_name: tz_name.clone(),
                timezone_nickname: match &config.timezones[contains.0 as usize].nickname {
                    Some(t) => Some(t.to_string()),
                    None => None,
                },
                displayed_name: match &config.timezones[contains.0 as usize].nickname {
                    Some(t) => format!("[{}] {}", t.to_string(), tz_name),
                    None => tz_name,
                },
                day_offset: day_diff,
                day_offset_str: offset_string,
                timestamp: converted_time.naive_local().timestamp(),
                timestring: converted_time.time().to_string(),
                separator: config.timezones[contains.0 as usize].separator,
            });
        }
    }

    tz_list.sort_by_key(|k| k.timestamp);

    if output_file == "pretty" {
        for item in tz_list {
            output += &format!(
                "{0: <25} {1} {2}\n",
                item.displayed_name, item.timestring, item.day_offset_str
            );
            if item.separator {
                output += &format!("----------------------------------\n");
            }
        }
    } else if output_file == "csv" {
        output += "Timezone Name,Timezone Nickname,Day Offset,Timestring,Timestamp\n";
        for item in tz_list {
            let nickname = match item.timezone_nickname {
                Some(t) => t,
                None => "null".to_owned(),
            };
            output += &format!(
                "{0},{1},{2},{3},{4}\n",
                item.timezone_name, nickname, item.day_offset, item.timestring, item.timestamp
            );
        }
    } else if output_file == "json" {
        output += &format!("{}", serde_json::to_string(&tz_list).unwrap());
    } else if output_file == "json_pretty" {
        output += &format!("{}", serde_json::to_string_pretty(&tz_list).unwrap());
    }
    return Some(output);
}

fn main() -> Result<(), ParseError> {
    let matches = cli().get_matches();

    let req_version = match matches.get_one::<bool>("version") {
        Some(t) => *t,
        None => false,
    };

    if req_version {
        println!("tc: v{}", VERSION);
        return Ok(());
    }

    match matches.subcommand() {
        Some(("u", sub_matches)) => {
            let datetime =
                match get_comparison_date_time(sub_matches.get_one::<String>("time"), None) {
                    Ok(t) => t.local_time.unwrap(),
                    Err(_e) => {
                        eprintln!("Something went wrong when parsing the time!");
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
                        eprintln!("Timezone not specified!");
                        return Ok(());
                    }
                };

                if tz_input.len() > 0 {
                    let mut config = match load_config() {
                        Ok(t) => t,
                        Err(_e) => {
                            return Ok(());
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
                                return Ok(());
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
                                    return Ok(());
                                }
                            };
                            println!("Added timezone {}", timezone.name());
                            return Ok(());
                        }
                    }
                    eprintln!("Timezone not found!");
                }
            }
            Some(("nick", sub_matches_nick)) => {
                let tz_input = match sub_matches_nick.get_one::<String>("timezone") {
                    Some(t) => t,
                    None => {
                        eprintln!("Timezone not specified!");
                        return Ok(());
                    }
                };
                if tz_input.len() > 0 {
                    let mut config = match load_config() {
                        Ok(t) => t,
                        Err(_e) => {
                            return Ok(());
                        }
                    };
                    let mut found = false;
                    for (i, timezone) in config.timezones.clone().into_iter().enumerate() {
                        if tz_input.contains(&timezone.timezone_name) {
                            let nick: Option<String> =
                                sub_matches_nick.get_one::<String>("nickname").cloned();
                            config.timezones[i].nickname = nick;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        eprintln!("Timezone not found saved in config!");
                        return Ok(());
                    }
                    match confy::store(APP_NAME, None, &config) {
                        Ok(_t) => "",
                        Err(_e) => {
                            eprintln!("Error saving config!");
                            return Ok(());
                        }
                    };
                    println!("Added nickname to {}", tz_input);
                }
            }
            Some(("sep", sub_matches_sep)) => {
                let tz_input = match sub_matches_sep.get_one::<String>("timezone") {
                    Some(t) => t,
                    None => {
                        eprintln!("Timezone not specified!");
                        return Ok(());
                    }
                };
                if tz_input.len() > 0 {
                    let mut config = match load_config() {
                        Ok(t) => t,
                        Err(_e) => {
                            return Ok(());
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
                        return Ok(());
                    }
                    match confy::store(APP_NAME, None, &config) {
                        Ok(_t) => "",
                        Err(_e) => {
                            eprintln!("Error saving config!");
                            return Ok(());
                        }
                    };
                    println!("Added separator after {}", tz_input);
                }
            }
            Some(("list", _)) => {
                match print_defines_list() {
                    Ok(t) => return Ok(t),
                    Err(_e) => return Ok(()),
                };
            }
            Some(("remove", sub_matches_remove)) => {
                let tz_input = match sub_matches_remove.get_one::<String>("timezone") {
                    Some(t) => t,
                    None => {
                        eprintln!("Timezone not specified!");
                        return Ok(());
                    }
                };

                if tz_input.len() > 0 {
                    let mut config = match load_config() {
                        Ok(t) => t,
                        Err(_e) => {
                            return Ok(());
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
                        return Ok(());
                    }
                    match confy::store(APP_NAME, None, &config) {
                        Ok(_t) => "",
                        Err(_e) => {
                            eprintln!("Error saving config!");
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
                eprintln!("Invalid Command!");
            }
            None => {
                match print_defines_list() {
                    Ok(t) => return Ok(t),
                    Err(_e) => return Ok(()),
                };
            }
        },
        Some(("t", sub_matches)) => {
            let curses = match sub_matches.get_one::<bool>("curses") {
                Some(t) => *t,
                None => false,
            };
            if curses {
                let window = initscr();
                window.nodelay(true);
                loop {
                    window.clear();
                    match t_command(Some(sub_matches)) {
                        Some(t) => window.addstr(t),
                        None => break,
                    };
                    match window.getch() {
                        Some(Input::KeyCancel) => break,
                        Some(_i) => (),
                        None => (),
                    };
                }
                endwin();
            } else {
                match t_command(Some(sub_matches)) {
                    Some(t) => println!("{}", t),
                    None => return Ok(()),
                };
            }
        }
        Some((&_, _)) => {
            eprintln!("Invalid Command!");
        }
        None => {
            match t_command(None) {
                Some(t) => println!("{}", t),
                None => return Ok(()),
            };
        }
    };

    Ok(())
}
