// TODO: write a parser

use chrono::{DateTime, Utc};
use regex::Regex;
use std::{collections::HashMap, convert::Infallible, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReservationConflictInfo {
    Parsed(ReservationConflict),
    Unparsed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationConflict {
    pub new: ReservationWindow,
    pub old: ReservationWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservationWindow {
    pub rid: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl FromStr for ReservationConflictInfo {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(conflict) = s.parse() {
            Ok(ReservationConflictInfo::Parsed(conflict))
        } else {
            Ok(ReservationConflictInfo::Unparsed(s.to_string()))
        }
    }
}

impl FromStr for ReservationConflict {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ParsedInfo::from_str(s)?.try_into()
    }
}

impl TryFrom<ParsedInfo> for ReservationConflict {
    type Error = ();

    fn try_from(value: ParsedInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            new: value.new.try_into()?,
            old: value.old.try_into()?,
        })
    }
}

impl TryFrom<HashMap<String, String>> for ReservationWindow {
    type Error = ();

    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let timespan_str = value.get("timespan").ok_or(())?.replace('"', "");
        let mut split = timespan_str.splitn(2, ',');
        let start = parse_datetime(split.next().ok_or(())?)?;
        let end = parse_datetime(split.next().ok_or(())?)?;
        Ok(Self {
            rid: value.get("resource_id").ok_or(())?.to_string(),
            start,
            end,
        })
    }
}

struct ParsedInfo {
    new: HashMap<String, String>,
    old: HashMap<String, String>,
}

impl FromStr for ParsedInfo {
    type Err = ();

    // ("Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-16 22:00:00+00\",\"2023-08-30 22:00:00+00\"))
    // conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-15 22:00:00+00\",\"2023-08-25 22:00:00+00\"))."
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // use regular expression to parse the string
        let re = Regex::new(r#"\((?P<k1>[a-zA-Z0-9_-]+)\s*,\s*(?P<k2>[a-zA-Z0-9_-]+)\)=\((?P<v1>[a-zA-Z0-9_-]+)\s*,\s*\[(?P<v2>[^\)\]]+)"#).unwrap();
        let mut maps = vec![];
        for cap in re.captures_iter(s) {
            let mut map = HashMap::new();
            map.insert(cap["k1"].to_string(), cap["v1"].to_string());
            map.insert(cap["k2"].to_string(), cap["v2"].to_string());
            maps.push(Some(map));
        }
        if maps.len() != 2 {
            return Err(());
        }
        Ok(ParsedInfo {
            new: maps[0].take().unwrap(),
            old: maps[1].take().unwrap(),
        })
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%#z")
        .map_err(|_| ())?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    // thread 'manager::tests::reserve_conflict_reservation_should_reject' panicked at 'called `Result::unwrap()` on an `Err` value:
    // Database(PgDatabaseError { severity: Error, code: "23P01", message: "conflicting key value violates exclusion constraint \"reservations_conflict\"",
    // detail: Some("Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-16 22:00:00+00\",\"2023-08-30 22:00:00+00\"))
    // conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-15 22:00:00+00\",\"2023-08-25 22:00:00+00\"))."),
    // hint: None, position: None, where: None, schema: Some("rsvp"), table: Some("reservations"), column: None, data_type: None,
    // constraint: Some("reservations_conflict"), file: Some("execIndexing.c"), line: Some(856), routine: Some("check_exclusion_or_unique_constraint") })',
    //  reservation/src/manager.rs:35:10
    // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

    const ERR_MSG: &str = "Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-16 22:00:00+00\",\"2023-08-30 22:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-08-15 22:00:00+00\",\"2023-08-25 22:00:00+00\")).";

    #[test]
    fn parse_datetime_should_work() {
        let dt = parse_datetime("2023-07-23 22:00:00+00").unwrap();
        assert_eq!(dt.to_rfc3339(), "2023-07-23T22:00:00+00:00")
    }

    #[test]
    fn parsed_info_should_work() {
        let info: ParsedInfo = ERR_MSG.parse().unwrap();
        assert_eq!(info.new["resource_id"], "ocean-view-room-713");
        assert_eq!(
            info.new["timespan"],
            "\"2023-08-16 22:00:00+00\",\"2023-08-30 22:00:00+00\""
        );
        assert_eq!(info.old["resource_id"], "ocean-view-room-713");
        assert_eq!(
            info.old["timespan"],
            "\"2023-08-15 22:00:00+00\",\"2023-08-25 22:00:00+00\""
        );
    }

    #[test]
    fn hash_map_to_reservation_window_should_work() {
        let mut map = HashMap::new();
        map.insert("resource_id".to_string(), "ocean-view-room-713".to_string());
        map.insert(
            "timespan".to_string(),
            "\"2023-12-26 22:00:00+00\",\"2023-12-30 19:00:00+00\"".to_string(),
        );
        let window: ReservationWindow = map.try_into().unwrap();
        assert_eq!(window.rid, "ocean-view-room-713");
        assert_eq!(window.start.to_rfc3339(), "2023-12-26T22:00:00+00:00");
        assert_eq!(window.end.to_rfc3339(), "2023-12-30T19:00:00+00:00");
    }

    #[test]
    fn conflict_error_message_should_parse() {
        let info = ERR_MSG.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.new.rid, "ocean-view-room-713");
                assert_eq!(conflict.new.start.to_rfc3339(), "2023-08-16T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2023-08-30T22:00:00+00:00");
                assert_eq!(conflict.old.rid, "ocean-view-room-713");
                assert_eq!(conflict.old.start.to_rfc3339(), "2023-08-15T22:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2023-08-25T22:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("should be parsed"),
        }
    }

    #[test]
    fn reserve_conflict_reservation_test() {
        const ERR_MSG1: &str = "Key (resource_id, timespan)=(ocean-view-room-713, [\"2023-07-26 22:00:00+00\",\"2023-07-30 19:00:00+00\")) conflicts with existing key (resource_id, timespan)=(ocean-view-room-713, [\"2023-07-25 22:00:00+00\",\"2023-07-28 19:00:00+00\")).";

        let info = ERR_MSG1.parse().unwrap();
        match info {
            ReservationConflictInfo::Parsed(conflict) => {
                assert_eq!(conflict.old.rid, "ocean-view-room-713");
                assert_eq!(conflict.old.start.to_rfc3339(), "2023-07-25T22:00:00+00:00");
                assert_eq!(conflict.old.end.to_rfc3339(), "2023-07-28T19:00:00+00:00");
                assert_eq!(conflict.new.rid, "ocean-view-room-713");
                assert_eq!(conflict.new.start.to_rfc3339(), "2023-07-26T22:00:00+00:00");
                assert_eq!(conflict.new.end.to_rfc3339(), "2023-07-30T19:00:00+00:00");
            }
            ReservationConflictInfo::Unparsed(_) => panic!("测试测试panic了哦弄个"),
        }
    }
}
