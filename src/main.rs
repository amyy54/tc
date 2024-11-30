use chrono::format::ParseError;
use chrono_tz::TZ_VARIANTS;
use clap::{arg, ArgMatches, Command};
use confy::ConfyError;
use pancurses::{endwin, initscr, Input};

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
            None => "".to_string(),
        };
        println!("{0: <25} {1}", timezone.timezone_name, nick);
    }
    Ok(())
}

fn t_command(sub_matches: Option<&ArgMatches>) -> Option<String> {
    if sub_matches.is_some() {
        let matches = sub_matches.unwrap();
        let timezone_input = match matches.get_one::<String>("timezone") {
            Some(t) => Some(t.to_string()),
            None => None,
        };
        let time = match matches.get_one::<String>("time") {
            Some(t) => Some(t.to_string()),
            None => None,
        };
        let output_format = match matches.get_one::<String>("output") {
            Some(t) => Some(t.to_string()),
            None => None,
        };
        return time_helpers::render_time(timezone_input, time, output_format);
    } else {
        return time_helpers::render_time(None, None, None);
    }
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
            let time = sub_matches.get_one::<String>("time");

            let input: Option<String> = match time {
                Some(t) => Some(t.to_string()),
                None => None,
            };

            let timestamp = time_helpers::get_unix_timestamp(input);

            let discord_ts = match sub_matches.get_one::<bool>("discord") {
                Some(t) => *t,
                None => false,
            };

            if discord_ts {
                println!("<t:{}:t>", timestamp);
            } else {
                println!("{}", timestamp);
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
                    None => return Ok(()),
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
                    None => nickname = "".to_string(),
                }

                let res = config::add_nick_to_timezone(tz_input.clone(), nickname);
                match res {
                    Some(t) => println!("Added nickname to {}", t),
                    None => return Ok(()),
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
                    None => return Ok(()),
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
                    None => return Ok(()),
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
