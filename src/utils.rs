// format date created to readable date string
pub fn unix_date_to_readable(date_milis: &str) -> String {
    let timestamp = date_milis.parse::<i64>().unwrap();
    let datetime = crate::DateTime::from_timestamp_millis(timestamp).unwrap();

    datetime.format("%Y-%m-%d").to_string()
}

pub fn get_home_dir() -> String {
    let mut home_dir = String::from("");
    if let Some(home) = std::env::home_dir() {
        home_dir = home
            .into_os_string()
            .into_string()
            .unwrap_or(String::from(""));
    }

    home_dir
}
