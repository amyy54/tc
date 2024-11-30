use chrono::format::ParseError;
use chrono::{DateTime, Datelike};
use chrono_tz::{Tz, TZ_VARIANTS};
use clap::{arg, ArgMatches, Command};
use confy::ConfyError;
use pancurses::{endwin, initscr, Input};
use std::str::FromStr;

mod config;
mod time_helpers;

const APP_NAME: &str = "tc";
const VERSION: &str = env!("CARGO_PKG_VERSION");

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

fn print_defines_list() -> Result<(), ConfyError> {
    let config = match config::load_config() {
        Ok(t) => t,
        Err(e) => {
            return Err(e);
        }
    };
    for timezone in config.timezones {
        let nick = match timezone.nickname {
            Some(t) => t,
            None => "".to_string()
        };
        println!(
            "{0: <25} {1}",
            timezone.timezone_name,
            nick
        );
    }
    Ok(())
}

fn t_command(sub_matches: Option<&ArgMatches>) -> Option<String> {
    let config = match config::load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };

    let mut output: String = "".to_string();

    let output_file: String = match sub_matches {
        Some(val) => match val.get_one::<String>("output") {
            Some(t) => t.to_string(),
            None => "pretty".to_string(),
        },
        None => "pretty".to_string(),
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
                    if config::saved_list_contains_timezone(&config, &tz_name).1 {
                        if tz_name.to_lowercase().contains(&tz_input.to_lowercase()) {
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

    let offset_comparison_datetime = match time_helpers::get_comparison_date_time(time_val, timezone) {
        Ok(t) => t,
        Err(_e) => {
            eprintln!("Something went wrong when parsing the time!");
            return None;
        }
    };

    if offset_comparison_datetime.kind == time_helpers::CurTimeKind::Tz {
        let time = offset_comparison_datetime.tz_time.unwrap();
        let fmt_string = "Time for ".to_string() + time.timezone().name();
        if output_file == "pretty" {
            output += &format!("{0: <25} {1}\n\n", fmt_string, time.time());
        }
    } else {
        let time = offset_comparison_datetime.local_time.unwrap();
        let fmt_string = "Local Time".to_string();
        if output_file == "pretty" {
            output += &format!("{0: <25} {1}\n\n", fmt_string, time.time());
        }
    }

    let mut tz_list: Vec<time_helpers::OutputTime> = [].to_vec();

    for timezone in TZ_VARIANTS {
        let tz_name = String::from_str(timezone.name()).unwrap();
        let contains = config::saved_list_contains_timezone(&config, &tz_name);
        if contains.1 {
            let converted_time: DateTime<Tz>;
            if offset_comparison_datetime.kind == time_helpers::CurTimeKind::Tz {
                let time = offset_comparison_datetime.tz_time.unwrap();
                converted_time = time.with_timezone(&timezone);
            } else {
                let time = offset_comparison_datetime.local_time.unwrap();
                converted_time = time.with_timezone(&timezone);
            }

            let mut offset_string: String;
            let mut day_diff: u32 = 0;
            if offset_comparison_datetime.kind == time_helpers::CurTimeKind::Tz {
                let offset_time = offset_comparison_datetime.tz_time.unwrap();

                if converted_time.day() != offset_time.day() {
                    let converted_ts =
                        time_helpers::convert_date_to_timestamp(converted_time.year(), converted_time.ordinal0());
                    let local_ts =
                        time_helpers::convert_date_to_timestamp(offset_time.year(), offset_time.ordinal0());
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
                        time_helpers::convert_date_to_timestamp(converted_time.year(), converted_time.ordinal0());
                    let local_ts =
                        time_helpers::convert_date_to_timestamp(offset_time.year(), offset_time.ordinal0());
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
            tz_list.push(time_helpers::OutputTime {
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
                None => "null".to_string(),
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
                match time_helpers::get_comparison_date_time(sub_matches.get_one::<String>("time"), None) {
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

                let res = config::add_timezone(tz_input.clone());
                match res {
                    Some(t) => println!("Added timezone {}", t),
                    None => return Ok(())
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

                let nickname: String;

                let nick: Option<String> = sub_matches_nick.get_one::<String>("nickname").cloned();
                match nick {
                    Some(t) => nickname = t,
                    None => nickname = "".to_string()
                }

                let res = config::add_nick_to_timezone(tz_input.clone(), nickname);
                match res {
                    Some(t) => println!("Added nickname to {}", t),
                    None => return Ok(())
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

                let res = config::add_sep_to_timezone(tz_input.clone());
                match res {
                    Some(t) => println!("Added separator after {}", t),
                    None => return Ok(())
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

                let res = config::remove_timezone(tz_input.clone());
                match res {
                    Some(t) => println!("Removed timezone {}", t),
                    None => return Ok(())
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
