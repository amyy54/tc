use chrono::{DateTime, Datelike};
use chrono_tz::{Tz, TZ_VARIANTS};
use std::str::FromStr;

use crate::config;
mod helpers;

pub fn render_time(
    timezone_input: Option<String>,
    time: Option<String>,
    output_format: Option<String>,
) -> Option<String> {
    let config = match config::load_config() {
        Ok(t) => t,
        Err(_e) => {
            return None;
        }
    };

    let mut output: String = "".to_string();

    let output_fmt: String = match output_format {
        Some(t) => t,
        None => "pretty".to_string(),
    };

    let mut timezone: Option<Tz> = None;

    if timezone_input.is_some() {
        let mut search_term = timezone_input.unwrap();

        for tz in config.timezones.clone() {
            match tz.nickname {
                Some(nick) => {
                    if nick
                        .to_lowercase()
                        .contains(search_term.to_lowercase().as_str())
                    {
                        search_term = tz.timezone_name.clone();
                        break;
                    }
                }
                None => continue,
            }
        }

        for tz in TZ_VARIANTS {
            let tz_name = tz.name().to_string();
            if config::saved_list_contains_timezone(&config, &tz_name).1 {
                if tz_name
                    .to_lowercase()
                    .contains(search_term.to_lowercase().as_str())
                {
                    timezone = Some(tz);
                }
            }
        }
    }

    let offset_comparison_datetime = match helpers::get_comparison_date_time(time, timezone) {
        Ok(t) => t,
        Err(_e) => {
            eprintln!("Something went wrong when parsing the time!");
            return None;
        }
    };

    if offset_comparison_datetime.kind == helpers::CurTimeKind::Tz {
        let time = offset_comparison_datetime.tz_time.unwrap();
        let fmt_string = "Time for ".to_string() + time.timezone().name();
        if output_fmt == "pretty" {
            output += &format!("{0: <25} {1}\n\n", fmt_string, time.time());
        }
    } else {
        let time = offset_comparison_datetime.local_time.unwrap();
        let fmt_string = "Local Time".to_string();
        if output_fmt == "pretty" {
            output += &format!("{0: <25} {1}\n\n", fmt_string, time.time());
        }
    }

    let mut tz_list: Vec<helpers::OutputTime> = [].to_vec();

    for tz in TZ_VARIANTS {
        let tz_name = String::from_str(tz.name()).unwrap();
        let contains = config::saved_list_contains_timezone(&config, &tz_name);
        if contains.1 {
            let converted_time: DateTime<Tz>;
            if offset_comparison_datetime.kind == helpers::CurTimeKind::Tz {
                let time = offset_comparison_datetime.tz_time.unwrap();
                converted_time = time.with_timezone(&tz);
            } else {
                let time = offset_comparison_datetime.local_time.unwrap();
                converted_time = time.with_timezone(&tz);
            }

            let mut offset_string: String;
            let mut day_diff: u32 = 0;
            if offset_comparison_datetime.kind == helpers::CurTimeKind::Tz {
                let offset_time = offset_comparison_datetime.tz_time.unwrap();

                if converted_time.day() != offset_time.day() {
                    let converted_ts = helpers::convert_date_to_timestamp(
                        converted_time.year(),
                        converted_time.ordinal0(),
                    );
                    let local_ts = helpers::convert_date_to_timestamp(
                        offset_time.year(),
                        offset_time.ordinal0(),
                    );
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
                    let converted_ts = helpers::convert_date_to_timestamp(
                        converted_time.year(),
                        converted_time.ordinal0(),
                    );
                    let local_ts = helpers::convert_date_to_timestamp(
                        offset_time.year(),
                        offset_time.ordinal0(),
                    );
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
            tz_list.push(helpers::OutputTime {
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
                timestamp: converted_time.naive_local().and_utc().timestamp(),
                timestring: converted_time.time().to_string(),
                separator: config.timezones[contains.0 as usize].separator,
            });
        }
    }

    tz_list.sort_by_key(|k| k.timestamp);

    if output_fmt == "pretty" {
        for item in tz_list {
            output += &format!(
                "{0: <25} {1} {2}\n",
                item.displayed_name, item.timestring, item.day_offset_str
            );
            if item.separator {
                output += &format!("----------------------------------\n");
            }
        }
    } else if output_fmt == "csv" {
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
    } else if output_fmt == "json" {
        output += &format!("{}", serde_json::to_string(&tz_list).unwrap());
    } else if output_fmt == "json_pretty" {
        output += &format!("{}", serde_json::to_string_pretty(&tz_list).unwrap());
    }
    return Some(output);
}

pub fn get_unix_timestamp(time: Option<String>) -> i64 {
    let datetime = helpers::get_comparison_date_time(time, None);

    return datetime.unwrap().local_time.unwrap().timestamp();
}
