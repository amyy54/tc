use chrono::format::ParseError;
use chrono::{offset, DateTime, Datelike, Local, NaiveTime, TimeZone, Timelike};
use chrono_tz::Tz;
use serde_derive::Serialize;

#[derive(PartialEq)]
pub enum CurTimeKind {
    Local,
    Tz,
}

pub struct CurTime {
    pub kind: CurTimeKind,
    pub local_time: Option<DateTime<Local>>,
    pub tz_time: Option<DateTime<Tz>>,
}

#[derive(Serialize, Clone)]
pub struct OutputTime {
    pub timezone_name: String,
    pub timezone_nickname: Option<String>,
    pub displayed_name: String,
    pub day_offset: u32,
    pub day_offset_str: String,
    pub timestamp: i64,
    pub timestring: String,
    pub separator: bool,
}

pub fn tz_offset_from_local_time(
    time: NaiveTime,
    now: DateTime<Local>,
    tz: Option<Tz>,
) -> NaiveTime {
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

pub fn get_comparison_date_time(
    time_option: Option<String>,
    tz: Option<Tz>,
) -> Result<CurTime, ParseError> {
    let now = offset::Local::now();

    let time = match time_option {
        Some(t) => {
            let collection: Vec<&str> = t.split(":").collect();
            match collection.len() {
                3 => NaiveTime::parse_from_str(t.as_str(), "%H:%M:%S")?,
                2 => NaiveTime::parse_from_str(t.as_str(), "%H:%M")?,
                1 => {
                    let newstring = collection[0].to_string() + ":00";
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

pub fn convert_date_to_timestamp(year: i32, ordinal: u32) -> u32 {
    ordinal + ((year - 1970) * 365) as u32
}
