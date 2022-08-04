use chrono::{DateTime, Local, NaiveDateTime};

/// Calcurate duration and format as min:sec:00
pub fn datetime_duration(
    start: &orgize::elements::Datetime,
    end: &orgize::elements::Datetime,
) -> String {
    let end_daytime: NaiveDateTime = end.into();
    let dur_sec = end_daytime
        .signed_duration_since(start.into())
        .num_seconds();
    let hour: i64 = dur_sec / 3600;
    let min: i64 = (dur_sec - hour * 3600) / 60;
    format!("{}:{:02}:00", hour, min)
}

/// Is the date ndays before or after now?
pub fn date_is_n_days_before_or_after_now(
    start: &orgize::elements::Datetime,
    before_ndays: i64,
    after_ndays: i64,
) -> bool {
    !date_is_n_days_before_now(start, before_ndays) && !date_is_n_days_after_now(start, after_ndays)
}

/// Is the date ndays before now?
fn date_is_n_days_before_now(timestamp: &orgize::elements::Datetime, ndays: i64) -> bool {
    if ndays <= 0 {
        return false;
    }

    let now_daytime: DateTime<Local> = Local::now();
    let naive_now_daytime = now_daytime.naive_local();
    let timestamp_chrono: chrono::NaiveDateTime = timestamp.into();

    let dur_sec = naive_now_daytime
        .signed_duration_since(timestamp_chrono)
        .num_seconds();

    if dur_sec / 60 / 60 / 24 >= ndays {
        return true;
    }
    false
}

/// Is the date ndays after now?
fn date_is_n_days_after_now(timestamp: &orgize::elements::Datetime, ndays: i64) -> bool {
    if ndays <= 0 {
        return false;
    }

    let now_daytime: DateTime<Local> = Local::now();
    let naive_now_daytime = now_daytime.naive_local();
    let timestamp_chrono: chrono::NaiveDateTime = timestamp.into();
    let dur_sec = timestamp_chrono
        .signed_duration_since(naive_now_daytime)
        .num_seconds();

    if dur_sec / 60 / 60 / 24 >= ndays {
        return true;
    }
    false
}
