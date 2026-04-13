pub fn save_token(token: &str) -> Result<(), std::io::Error> {
    let mut clickdown_folder_path = crate::PathBuf::from(crate::utils::get_home_dir());
    clickdown_folder_path.push(".config/clickdown/token");

    match crate::fs::write(clickdown_folder_path, token) {
        Ok(value) => {
            println!("Saved token at ~/.config/clickdown");
            value
        }
        Err(err) => {
            println!("There was a proble saving token.\n{}", err)
        }
    };

    Ok(())
}

pub fn get_clickdown_token() -> Result<String, std::io::Error> {
    let mut clickdown_folder_path = crate::PathBuf::from(crate::utils::get_home_dir());
    clickdown_folder_path.push(".config/clickdown");

    // create clickdown .config folder if missing
    crate::fs::create_dir_all(&clickdown_folder_path)?;

    // append filename
    clickdown_folder_path.push("token");

    Ok(crate::fs::read_to_string(clickdown_folder_path).unwrap_or("".to_string()))
}
