use crate::fullcalendar::make_event_from_clocks;
use crate::fullcalendar::make_event_from_deadline_with_range;
use crate::fullcalendar::make_event_from_deadline_wo_range;
use crate::fullcalendar::make_event_from_scheduled_with_range;
use crate::fullcalendar::make_event_from_scheduled_wo_range;
use crate::fullcalendar::FCEvent;
use crate::timestamp::date_is_n_days_before_or_after_now;
use glob::glob;
use orgize::elements;
use orgize::elements::PropertiesMap;
use orgize::Event;
use orgize::Org;
use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Pase Org mode text and output a vector of FCEvent
pub fn parsed_org_to_fc_vec(
    org: Org,
    ignore_before_days: i64,
    ignore_after_days: i64,
    file_path: &str,
) -> Vec<FCEvent> {
    let mut fullcalendar_events: Vec<FCEvent> = Vec::new();
    for event in org.iter() {
        // println!("{:?}", event);
        match event {
            Event::Start(element) => match element {
                elements::Element::Title(title) => match &title.planning {
                    Some(plan) => {
                        match &plan.deadline {
                            Some(deadline) => match deadline {
                                elements::Timestamp::Active {
                                    start, repeater, ..
                                } => {
                                    push_deadline_wo_range(
                                        &mut fullcalendar_events,
                                        &title,
                                        &start,
                                        &repeater,
                                        ignore_before_days,
                                        ignore_after_days,
                                        &file_path,
                                    );
                                }
                                elements::Timestamp::ActiveRange {
                                    start,
                                    end,
                                    start_repeater,
                                    ..
                                } => {
                                    if date_is_n_days_before_or_after_now(
                                        start,
                                        ignore_before_days,
                                        ignore_after_days,
                                    ) {
                                        fullcalendar_events.push(
                                            make_event_from_deadline_with_range(
                                                &title,
                                                &start,
                                                &end,
                                                &start_repeater,
                                                &file_path,
                                            ),
                                        );
                                    }
                                }

                                _ => {}
                            },
                            None => {}
                        }
                        match &plan.scheduled {
                            Some(schedule) => match schedule {
                                elements::Timestamp::Active {
                                    start, repeater, ..
                                } => {
                                    if date_is_n_days_before_or_after_now(
                                        start,
                                        ignore_before_days,
                                        ignore_after_days,
                                    ) {
                                        fullcalendar_events.push(
                                            make_event_from_scheduled_wo_range(
                                                &title, &start, &repeater, &file_path,
                                            ),
                                        );
                                    }
                                }
                                elements::Timestamp::ActiveRange {
                                    start,
                                    end,
                                    start_repeater,
                                    ..
                                } => {
                                    if date_is_n_days_before_or_after_now(
                                        start,
                                        ignore_before_days,
                                        ignore_after_days,
                                    ) {
                                        fullcalendar_events.push(
                                            make_event_from_scheduled_with_range(
                                                &title,
                                                &start,
                                                &end,
                                                &start_repeater,
                                                &file_path,
                                            ),
                                        );
                                    }
                                }

                                _ => {}
                            },
                            None => {}
                        }
                    }
                    None => {}
                },

                _ => {}
            },
            Event::End(element) => match element {
                elements::Element::Headline { .. } => {}
                _ => {}
            },
        }
    }
    fullcalendar_events
}

/// Pase Org mode text and output a vector of FCEvent onfly for clock
pub fn parsed_org_to_fc_vec_clock(
    org: Org,
    ignore_before_days: i64,
    ignore_after_days: i64,
    file_path: &str,
) -> Vec<FCEvent> {
    let mut fullcalendar_events: Vec<FCEvent> = Vec::new();

    let mut title: &elements::Title = &elements::Title {
        priority: None,
        tags: Vec::new(),
        keyword: None,
        raw: Cow::Borrowed(""),
        planning: None,
        properties: PropertiesMap::new(),
        post_blank: 0,
        level: 1,
    };

    for event in org.iter() {
        // println!("{:?}", event);

        match event {
            Event::Start(element) => match element {
                elements::Element::Title(title_orig) => {
                    title = title_orig;
                }
                _ => {}
            },
            _ => {}
        }

        if let Event::Start(element) = event {
            if let elements::Element::Clock(clock) = element {
                if let elements::Clock::Closed { start, end, .. } = clock {
                    if date_is_n_days_before_or_after_now(
                        start,
                        ignore_before_days,
                        ignore_after_days,
                    ) {
                        fullcalendar_events
                            .push(make_event_from_clocks(&title, &start, &end, &file_path));
                    }
                }
            }
        }
    }
    fullcalendar_events
}

/// Parse the Org mode content and output FullCalendar JSON file
pub fn org_to_fc_json(
    contents: &str,
    ignore_before_days: i64,
    ignore_after_days: i64,
    file_path: &str,
) -> String {
    let org = Org::parse(&contents);
    let fullcalendar_events =
        parsed_org_to_fc_vec(org, ignore_before_days, ignore_after_days, &file_path);
    serde_json::to_string_pretty(&fullcalendar_events).unwrap()
}

/// Parse the Org mode content for only CLOCK and output FullCalendar JSON file
pub fn org_to_fc_clock_json(
    contents: &str,
    ignore_before_days: i64,
    ignore_after_days: i64,
    file_path: &str,
) -> String {
    let org = Org::parse(&contents);
    let fullcalendar_events =
        parsed_org_to_fc_vec_clock(org, ignore_before_days, ignore_after_days, &file_path);
    serde_json::to_string_pretty(&fullcalendar_events).unwrap()
}

pub fn push_deadline_wo_range(
    events: &mut Vec<FCEvent>,
    title: &elements::Title,
    start: &elements::Datetime,
    repeater: &Option<elements::Repeater>,
    ignore_before_days: i64,
    ignore_after_days: i64,
    file_path: &str,
) {
    if date_is_n_days_before_or_after_now(start, ignore_before_days, ignore_after_days) {
        events.push(make_event_from_deadline_wo_range(
            &title, &start, &repeater, &file_path,
        ));
    }
}

/// Make a FC Event vector from one Org file
pub fn orgfile_to_fc_json_vec(
    file: &str,
    ignore_before_days: i64,
    ignore_after_days: i64,
) -> std::result::Result<Vec<FCEvent>, Box<dyn Error>> {
    let file_path = Path::new(file).canonicalize();

    let file_path = match file_path {
        Ok(path) => path,
        Err(e) => return Err(Box::new(e)),
    };

    //    println!("PATH: {}", file_path.to_str().unwrap());
    let f = File::open(file_path);
    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(Box::new(e)),
    };

    let mut s = String::new();

    match f.read_to_string(&mut s) {
        Ok(_) => {}
        Err(e) => return Err(Box::new(e)),
    };

    let org = Org::parse(&s);
    let fullcalendar_events =
        parsed_org_to_fc_vec(org, ignore_before_days, ignore_after_days, &file);
    Ok(fullcalendar_events)
}

/// Make a FC Event vector from all Org files in a directory.
pub fn org_dir_to_fc_json_vec(
    dir: &str,
    ignore_before_days: i64,
    ignore_after_days: i64,
) -> std::result::Result<Vec<FCEvent>, Box<dyn Error>> {
    let mut parsed_json_vec: Vec<FCEvent> = Vec::new();
    let glob_pattern = format!("{}/*.org", dir);
    let files = glob(&glob_pattern);
    let files = match files {
        Ok(f) => f.map(|e| e.unwrap()).collect::<Vec<_>>(),
        Err(e) => {
            println!("Application error: {e}");
            std::process::exit(1);
        }
    };
    for file in files {
        eprintln!("{:?}", file);
        match orgfile_to_fc_json_vec(
            &file.into_os_string().into_string().unwrap(),
            ignore_before_days,
            ignore_after_days,
        ) {
            Ok(data) => {
                parsed_json_vec.extend(data);
            }
            Err(e) => {
                println!("Application error: {e}");
                std::process::exit(1);
            }
        }
    }
    Ok(parsed_json_vec)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use chrono::Local;

    use super::*;

    #[test]
    fn print_deadline_wo_time_wo_range_wo_repeater_wo_desc() {
        let input = r#"
* Deadlin, Withtout Time, Without Range, Repeater: None
DEADLINE: <2022-07-25 Mon>
:PROPERTIES:
:FC_BG_COLOR:  black
:FC_TXT_COLOR: red
:END:
"#;
        let expected = r#"[
  {
    "title": "DL: Deadlin, Withtout Time, Without Range, Repeater: None",
    "start": "2022-07-25",
    "description": "Deadlin, Withtout Time, Without Range, Repeater: None",
    "color": "black",
    "textColor": "red"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_deadline_wo_time_wo_range_wt_repeater_wo_desc() {
        let input = r#"
** Deadlin, Withtout Time, Without Range, Repeater: +3d
DEADLINE: <2022-07-25 Mon +3d>
"#;
        let expected = r#"[
  {
    "title": "DL: Deadlin, Withtout Time, Without Range, Repeater: +3d",
    "rrule": {
      "dtstart": "2022-07-25",
      "freq": "daily",
      "interval": 3
    },
    "start": "2022-07-25",
    "description": "Deadlin, Withtout Time, Without Range, Repeater: +3d"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_deadline_wo_time_wt_range_wo_repeater_wo_desc() {
        let input = r#"
** Deadlin, Withtout Time, With Range,    Repeater: None
DEADLINE: <2022-07-26 Tue>--<2022-07-28 Thu>
:PROPERTIES:
:FC_BG_COLOR:  red
:FC_TXT_COLOR: black
:END:
"#;
        let expected = r#"[
  {
    "title": "DL: Deadlin, Withtout Time, With Range,    Repeater: None",
    "start": "2022-07-26T00:00:00",
    "end": "2022-07-28T00:00:00",
    "description": "Deadlin, Withtout Time, With Range,    Repeater: None",
    "color": "red",
    "textColor": "black"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_deadline_wt_time_wo_range_wo_repeater_wo_desc() {
        let input = r#"
** Deadline With Time,     Without Range and with a Time
DEADLINE: <2022-07-16 Sat 12:00>
  "#;
        let expected = r#"[
  {
    "title": "DL: Deadline With Time,     Without Range and with a Time",
    "start": "2022-07-16T12:00:00",
    "description": "Deadline With Time,     Without Range and with a Time"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_deadline_wt_time_wt_range_wo_repeater_wt_desc() {
        let input = r#"
** Deadlin, With Time,    With Range, Repeater: None
DEADLINE: <2022-07-25 Mon 18:00-22:00>
:PROPERTIES:
:DESCRIPTION: Deadlin, Withtout Time, Without Range, Repeater: None
:END:
"#;
        let expected = r#"[
  {
    "title": "DL: Deadlin, With Time,    With Range, Repeater: None",
    "start": "2022-07-25T18:00:00",
    "end": "2022-07-25T22:00:00",
    "description": "Deadlin, Withtout Time, Without Range, Repeater: None<br>"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_deadline_wt_time_wt_range_wt_repeater_wt_desc() {
        let input = r#"
** Deadlin, With Time,    With Range, Repeater: +1w
DEADLINE: <2022-07-26 Tue 10:00-11:00 +1w>
:PROPERTIES:
:DESCRIPTION: Deadlin, Withtout Time, Without Range, Repeater: +2d
:END:
"#;
        let expected = r#"[
  {
    "title": "DL: Deadlin, With Time,    With Range, Repeater: +1w",
    "rrule": {
      "dtstart": "2022-07-26T10:00:00",
      "freq": "weekly",
      "interval": 1
    },
    "start": "2022-07-26T10:00:00",
    "end": "2022-07-26T11:00:00",
    "duration": "1:00:00",
    "description": "Deadlin, Withtout Time, Without Range, Repeater: +2d<br>"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_schedule_wo_time_wo_range_wo_repeater_wo_desc() {
        let input = r#"
** Scheduled, Withtout Time, Without Range, Repeater: None
SCHEDULED: <2022-07-19 Tue>
:PROPERTIES:
:FC_BG_COLOR:  blue
:FC_TXT_COLOR: green
:END:
"#;
        let expected = r#"[
  {
    "title": "SCL: Scheduled, Withtout Time, Without Range, Repeater: None",
    "start": "2022-07-19",
    "description": "Scheduled, Withtout Time, Without Range, Repeater: None",
    "color": "blue",
    "textColor": "green"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_schedule_wo_time_wo_range_wt_repeater_wo_desc() {
        let input = r#"
** Scheduled, Withtout Time, Without Range, Repeater: +1w
SCHEDULED: <2022-07-27 Wed +1w>
"#;
        let expected = r#"[
  {
    "title": "SCL: Scheduled, Withtout Time, Without Range, Repeater: +1w",
    "rrule": {
      "dtstart": "2022-07-27",
      "freq": "weekly",
      "interval": 1
    },
    "start": "2022-07-27",
    "description": "Scheduled, Withtout Time, Without Range, Repeater: +1w"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_schedule_wo_time_wt_range_wo_repeater_wo_desc() {
        let input = r#"
** Scheduled, Withtout Time, With Range,    Repeater: None
SCHEDULED: <2022-07-18 Mon>--<2022-07-21 Thu>
:PROPERTIES:
:FC_BG_COLOR:  green
:FC_TXT_COLOR: blue
:END:
"#;
        let expected = r#"[
  {
    "title": "SCL: Scheduled, Withtout Time, With Range,    Repeater: None",
    "start": "2022-07-18T00:00:00",
    "end": "2022-07-21T00:00:00",
    "description": "Scheduled, Withtout Time, With Range,    Repeater: None",
    "color": "green",
    "textColor": "blue"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_schedule_wt_time_wo_range_wo_repeater_wo_desc() {
        let input = r#"
** Scheduled, With Time,     Without Range, Repeater: None
SCHEDULED: <2022-07-28 Thu 12:00> 
"#;
        let expected = r#"[
  {
    "title": "SCL: Scheduled, With Time,     Without Range, Repeater: None",
    "start": "2022-07-28T12:00:00",
    "description": "Scheduled, With Time,     Without Range, Repeater: None"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_schedule_wt_time_wt_range_wo_repeater_wo_desc() {
        let input = r#"
** Scheduled, With Time,     With Range,    Repeater: None
SCHEDULED: <2022-07-27 Wed 10:00-11:30>
"#;
        let expected = r#"[
  {
    "title": "SCL: Scheduled, With Time,     With Range,    Repeater: None",
    "start": "2022-07-27T10:00:00",
    "end": "2022-07-27T11:30:00",
    "description": "Scheduled, With Time,     With Range,    Repeater: None"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_schedule_wt_time_wt_range_wt_repeater_wt_desc() {
        let input = r#"
** Scheduled, With Time,     With Range,    Repeater: +1d, With Multiline DEDC
SCHEDULED: <2022-07-27 Wed 17:00-19:21 +1d> 
:PROPERTIES:
:DESCRIPTION: description for this header
:DESCRIPTION+: additional description (into next line
:ID:       2e8a1478-54f7-45f6-9fa2-2db7472a66a2
:END:
"#;
        let expected = r#"[
  {
    "title": "SCL: Scheduled, With Time,     With Range,    Repeater: +1d, With Multiline DEDC",
    "rrule": {
      "dtstart": "2022-07-27T17:00:00",
      "freq": "daily",
      "interval": 1
    },
    "start": "2022-07-27T17:00:00",
    "end": "2022-07-27T19:21:00",
    "duration": "2:21:00",
    "description": "description for this header<br>additional description (into next line<br>"
  }
]"#;
        assert_eq!(org_to_fc_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_clock_1_wt_time_wt_range_wt_desc() {
        let input = r#"
** TODO Clock1
DEADLINE: <2022-07-20 Wed>
:PROPERTIES:
:DESCRIPTION: Clock test1
:END:
:LOGBOOK:
CLOCK: [2022-07-18 Mon 15:54]--[2022-07-18 Mon 17:07] =>  1:13
:END:
"#;
        let expected = r#"[
  {
    "title": "Clock1",
    "start": "2022-07-18T15:54:00",
    "end": "2022-07-18T17:07:00",
    "duration": "1:13:00",
    "description": "Clock test1<br>"
  }
]"#;
        assert_eq!(org_to_fc_clock_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_clock_2_wt_time_wt_range_wt_desc() {
        let input = r#"
** TODO Clock2
DEADLINE: <2022-07-20 Wed>
:PROPERTIES:
:DESCRIPTION: Clock test2
:END:
:LOGBOOK:
CLOCK: [2022-07-18 Mon 13:54]--[2022-07-18 Mon 14:07] =>  0:13
CLOCK: [2022-07-17 Sun 13:54]--[2022-07-18 Mon 15:07] => 25:13
:END:
"#;
        let expected = r#"[
  {
    "title": "Clock2",
    "start": "2022-07-18T13:54:00",
    "end": "2022-07-18T14:07:00",
    "duration": "0:13:00",
    "description": "Clock test2<br>"
  },
  {
    "title": "Clock2",
    "start": "2022-07-17T13:54:00",
    "end": "2022-07-18T15:07:00",
    "duration": "25:13:00",
    "description": "Clock test2<br>"
  }
]"#;
        assert_eq!(org_to_fc_clock_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn print_clock_1_wt_time_wt_range_wo_desc() {
        let input = r#"
** TODO Clock3
DEADLINE: <2022-07-22 Fri>
:PROPERTIES:
:FC_BG_COLOR:  yellow
:FC_TXT_COLOR: white
:END:
:LOGBOOK:
CLOCK: [2022-07-20 Wed 15:54]--[2022-07-20 Wed 17:07] =>  1:13
:END:
   "#;
        let expected = r#"[
  {
    "title": "Clock3",
    "start": "2022-07-20T15:54:00",
    "end": "2022-07-20T17:07:00",
    "duration": "1:13:00",
    "description": "TODO Clock3",
    "color": "yellow",
    "textColor": "white"
  }
]"#;
        assert_eq!(org_to_fc_clock_json(input, 0, 0, ""), expected);
    }

    #[test]
    fn ignore_deadline_after_1_day() {
        let now_daytime: DateTime<Local> = Local::now();
        let tommorow = now_daytime.naive_local() + chrono::Duration::minutes(24 * 60 + 1);

        let input = format!(
            "** Ignore a DEADLINE 1 day after now\nDEADLINE: <{}>",
            tommorow.format("%Y-%m-%d %a %H:%M").to_string()
        );

        let expected = r#"[]"#;
        assert_eq!(org_to_fc_json(&input, 0, 1, ""), expected);
    }

    #[test]
    fn ignore_clock_befre_1_day() {
        let now_daytime: DateTime<Local> = Local::now();
        let start = now_daytime.naive_local() - chrono::Duration::minutes(24 * 60);
        let end = start + chrono::Duration::minutes(30);

        let input = format!(
            "* test2\n:LOGBOOK:\nCLOCK: [{}]--[{}] => 24:00\n:END:\n",
            start.format("%Y-%m-%d %a %H:%M").to_string(),
            end.format("%Y-%m-%d %a %H:%M").to_string()
        );

        let expected = r#"[]"#;
        assert_eq!(org_to_fc_clock_json(&input, 1, 0, ""), expected);
    }

    #[test]
    fn ignore_headline_only() {
        let input = "* Headline Only";
        let expected = r#"[]"#;
        assert_eq!(org_to_fc_clock_json(&input, 1, 0, ""), expected);
    }
}
