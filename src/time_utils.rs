use chrono::prelude::*;

pub fn timestamp_to_time_string(timestamp: f64) -> String {
    let as_string = format!("{}", timestamp);
    let int_part = as_string.split(".").collect::<Vec<_>>()[0]
        .parse::<i64>()
        .unwrap();
    let decimal_part = as_string.split(".").collect::<Vec<_>>()[1]
        .parse::<u32>()
        .unwrap();
    let date_string = Utc.timestamp(int_part, decimal_part);

    format!("{}", date_string.format("%H:%M"))
}

pub fn format_to_prom_date_string(date: DateTime<Utc>) -> String {
    format!("{:?}", date)
}
pub fn get_now() -> String {
    let now = Utc::now();
    format_to_prom_date_string(now)
}
pub fn get_one_hour_before_date(date: &str) -> String {
    let date = date.parse::<DateTime<Utc>>().unwrap();
    let one_hour_before = date - chrono::Duration::hours(1);
    format_to_prom_date_string(one_hour_before)
}

#[cfg(test)]

mod test {
    use super::*;

    #[test]
    fn test_format_to_prom_date_string() {
        // get first of january 2021
        let date = Utc.ymd(2021, 1, 1).and_hms(0, 0, 0);
        assert_eq!(
            format_to_prom_date_string(date),
            "2021-01-01T00:00:00Z".to_string()
        );
    }

    #[test]
    fn test_get_one_hour_before_date() {
        // get first of january 2021
        assert_eq!(
            get_one_hour_before_date("2021-01-01T00:00:00Z"),
            "2020-12-31T23:00:00Z".to_string()
        );
    }

    #[test]
    fn test_timestamp_to_string() {
        let timestamp = 1662621690.781;
        let result = timestamp_to_time_string(timestamp);
        assert_eq!(result, "07:21");
    }
}
