use crate::timestamp::datetime_duration;
use chrono::{NaiveDate, NaiveDateTime};
use orgize::elements;
use serde::Serialize;

/// Struct for Fullcalendar events.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")] // https://serde.rs/attr-rename.html
pub struct FCEvent {
    /// The calendar's title. (Required)
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// FC RRule
    rrule: Option<FCRRule>,
    /// The calendar's start date (with/without time).
    pub start: FCStart,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The calendar's start date (with time, optional).
    pub end: Option<NaiveDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Duration (for RRULE)
    pub duration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Description (for tooltips)
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Event color
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Text color
    pub text_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

impl FCEvent {
    /// Constructor
    pub fn new(
        title: String,
        start: FCStart,
        end: Option<NaiveDateTime>,
        duration: Option<String>,
        description: Option<String>,
        rrule: Option<FCRRule>,
        color: Option<String>,
        text_color: Option<String>,
        file_path: Option<String>,
    ) -> FCEvent {
        FCEvent {
            title,
            start,
            rrule,
            end,
            duration,
            description,
            color,
            text_color,
            file_path,
        }
    }
}
/// RRule for Fullcalendar events.
#[derive(Serialize, Debug)]
pub struct FCRRule {
    pub dtstart: FCStart,
    /// Time unit: yearly, weekly, daily, hourly
    /// FC frequency are yearly, monthly, daily, hourly, minutely, secondly
    /// But Org surports yearly, weekly, daily, hourly
    pub freq: String,
    /// Repeater interval
    pub interval: usize,
}

/// Date format for FullCalendar events
#[derive(Serialize, Debug)]
#[serde(untagged)] // https://serde.rs/enum-reeapresentations.html
pub enum FCStart {
    /// Timestamp with a date only.
    Day(NaiveDate),
    /// Timestamp with a date and a time.
    DateTime(NaiveDateTime),
}

/// Convert Org reeater to FC RRule
fn repeater_to_fc_rrule(
    start: &orgize::elements::Datetime,
    repeater: &elements::Repeater,
) -> Option<FCRRule> {
    let timeunit: String;
    match repeater.unit {
        elements::TimeUnit::Year => {
            timeunit = "yearly".to_string();
        }
        elements::TimeUnit::Week => {
            timeunit = "weekly".to_string();
        }
        elements::TimeUnit::Month => {
            timeunit = "monthly".to_string();
        }
        elements::TimeUnit::Day => {
            timeunit = "daily".to_string();
        }
        elements::TimeUnit::Hour => {
            timeunit = "hourly".to_string();
        }
    };
    let dtstart: FCStart;
    match start.hour {
        Some(..) => {
            dtstart = FCStart::DateTime(start.into());
        }
        _ => {
            dtstart = FCStart::Day(start.into());
        }
    }
    Some(FCRRule {
        dtstart: dtstart,
        freq: timeunit,
        interval: repeater.value,
    })
}

/// Make a title with a keyword
pub fn make_title_with_keyword(title: &elements::Title, prefix: &str) -> String {
    let title_raw = &title.raw;
    let mut keyword: String = String::new();
    if let Some(key) = &title.keyword {
        keyword = format!("{} ", &key);
    }
    format!("{}{}{}", prefix, keyword, title_raw)
}

/// Make a title without a keyword
pub fn make_title_without_keyword(title: &elements::Title, prefix: &str) -> String {
    let title_raw = &title.raw;
    format!("{}{}", prefix, title_raw)
}

/// make a "description" from "DESCRIPTION" properties

pub fn make_description(title: &&elements::Title) -> Option<String> {
    //    pub fn make_description(properties: &elements::PropertiesMap) -> Option<String> {
    let mut description: String = String::new();
    for (k, v) in title.properties.clone().into_iter() {
        if k == "DESCRIPTION" {
            description.push_str(v.as_ref());
            description.push_str("<br>");
        }
    }

    if description == "" {
        return Some(make_title_with_keyword(&title, ""));
    }

    Some(description)
}

/// Make a event's "color"
pub fn make_color(title: &elements::Title) -> Option<String> {
    let mut color: String = String::new();
    for (k, v) in title.properties.clone().into_iter() {
        if k == "FC_BG_COLOR" {
            color = v.to_string();
        }
    }
    if color == "" {
        return None;
    }
    Some(color)
}

/// Make event's text color
pub fn make_text_color(title: &elements::Title) -> Option<String> {
    let mut text_color: String = String::new();
    for (k, v) in title.properties.clone().into_iter() {
        if k == "FC_TXT_COLOR" {
            text_color = v.to_string();
        }
    }

    if text_color == "" {
        return None;
    }

    Some(text_color)
}

pub fn make_rrule(
    start: &orgize::elements::Datetime,
    repeater: &Option<elements::Repeater>,
) -> Option<FCRRule> {
    match repeater {
        Some(repeater) => {
            return repeater_to_fc_rrule(start, repeater);
        }
        _ => {
            return None;
        }
    }
}

pub fn make_start(start: &elements::Datetime) -> FCStart {
    match start.hour {
        Some(..) => {
            return FCStart::DateTime(start.into());
        }
        None => {
            return FCStart::Day(start.into());
        }
    }
}

fn make_file_path(file_path: &str) -> Option<String> {
    if file_path == "" {
        return None;
    }
    Some(file_path.to_string())
}

fn make_event_from_dl_or_scl_wo_range(
    title: &elements::Title,
    start: &elements::Datetime,
    repeater: &Option<elements::Repeater>,
    prefix: &str,
    file_path: &str,
) -> FCEvent {
    FCEvent::new(
        make_title_with_keyword(&title, prefix),
        make_start(start),
        None,
        None,
        make_description(&title),
        make_rrule(start, repeater),
        make_color(&title),
        make_text_color(&title),
        make_file_path(&file_path),
    )
}

pub fn make_event_from_deadline_wo_range(
    title: &elements::Title,
    start: &elements::Datetime,
    repeater: &Option<elements::Repeater>,
    file_path: &str,
) -> FCEvent {
    make_event_from_dl_or_scl_wo_range(&title, &start, &repeater, "DL: ", file_path)
}

pub fn make_event_from_scheduled_wo_range(
    title: &elements::Title,
    start: &elements::Datetime,
    repeater: &Option<elements::Repeater>,
    file_path: &str,
) -> FCEvent {
    make_event_from_dl_or_scl_wo_range(&title, &start, &repeater, "SCL: ", file_path)
}

fn make_event_from_dl_or_scl_with_range(
    title: &elements::Title,
    start: &elements::Datetime,
    end: &elements::Datetime,
    start_repeater: &Option<elements::Repeater>,
    prefix: &str,
    file_path: &str,
) -> FCEvent {
    let rrule = make_rrule(start, start_repeater);
    let duration = if rrule.is_none() {
        None
    } else {
        Some(datetime_duration(start, end))
    };

    FCEvent::new(
        make_title_with_keyword(title, prefix),
        FCStart::DateTime(start.into()),
        Some(end.into()),
        duration,
        make_description(&title),
        make_rrule(&start, &start_repeater),
        make_color(&title),
        make_text_color(&title),
        make_file_path(&file_path),
    )
}

pub fn make_event_from_deadline_with_range(
    title: &elements::Title,
    start: &elements::Datetime,
    end: &elements::Datetime,
    repeater: &Option<elements::Repeater>,
    file_path: &str,
) -> FCEvent {
    make_event_from_dl_or_scl_with_range(&title, &start, &end, &repeater, "DL: ", file_path)
}

pub fn make_event_from_scheduled_with_range(
    title: &elements::Title,
    start: &elements::Datetime,
    end: &elements::Datetime,
    repeater: &Option<elements::Repeater>,
    file_path: &str,
) -> FCEvent {
    make_event_from_dl_or_scl_with_range(&title, &start, &end, &repeater, "SCL: ", file_path)
}

/// Make events from CLOCK lines
pub fn make_event_from_clocks(
    title: &elements::Title,
    start: &elements::Datetime,
    end: &elements::Datetime,
    file_path: &str,
) -> FCEvent {
    FCEvent::new(
        make_title_without_keyword(&title, ""),
        FCStart::DateTime(start.into()),
        Some(end.into()),
        Some(datetime_duration(start, end)),
        make_description(&title),
        None,
        make_color(&title),
        make_text_color(&title),
        make_file_path(&file_path),
    )
}
