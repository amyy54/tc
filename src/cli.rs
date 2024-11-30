use clap::{arg, crate_authors, crate_version, Command};

const APP_NAME: &str = "tc";

pub fn cli() -> Command {
    Command::new(APP_NAME)
        .about("(T)ime (C)onverter. For those who have to constantly deal with timezones.")
        .version(crate_version!())
        .author(crate_authors!("\n"))
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
}
