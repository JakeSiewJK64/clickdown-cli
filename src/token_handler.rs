pub fn save_token(token: &str) -> Result<&str, std::io::Error> {
    if token.is_empty() {
        return Ok("");
    }

    let mut clickdown_folder_path = crate::PathBuf::from(crate::utils::get_home_dir());
    clickdown_folder_path.push(".config/clickdown/token");

    match std::fs::write(clickdown_folder_path, token) {
        Ok(value) => {
            println!("Saved token at ~/.config/clickdown");
            value
        }
        Err(err) => {
            println!("There was a proble saving token.\n{}", err)
        }
    };

    Ok(token)
}

pub fn get_clickdown_token() -> Result<String, std::io::Error> {
    let mut clickdown_folder_path = crate::PathBuf::from(crate::utils::get_home_dir());
    clickdown_folder_path.push(".config/clickdown");

    // create clickdown .config folder if missing
    std::fs::create_dir_all(&clickdown_folder_path)?;

    // append filename
    clickdown_folder_path.push("token");

    Ok(std::fs::read_to_string(clickdown_folder_path).unwrap_or("".to_string()))
}

#[cfg(test)]
mod test {
    use crate::token_handler::save_token;

    #[test]
    fn test_skip_empty_token() {
        assert!(save_token("").is_ok());
        assert!(save_token("token").is_ok());

        let res = save_token("token").unwrap_or("");
        assert_eq!(res, "token");
    }
}
