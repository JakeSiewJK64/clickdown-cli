// format date created to readable date string
pub fn unix_date_to_readable(date_milis: &str) -> String {
    let timestamp = date_milis.parse::<i64>().unwrap();
    let datetime = crate::DateTime::from_timestamp_millis(timestamp).unwrap();

    datetime.format("%Y-%m-%d").to_string()
}
