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

pub fn render_table(columns: Vec<&str>, rows: Vec<Vec<crate::Cell>>) {
    let mut table = crate::Table::new();
    table
        .load_preset(crate::UTF8_FULL)
        .apply_modifier(crate::UTF8_ROUND_CORNERS)
        .set_content_arrangement(crate::ContentArrangement::Dynamic);
    table.set_header(crate::Row::from(columns));
    table.add_rows(rows);
    println!("{}", table);
}

pub fn render_task_table(tasks: Vec<crate::clickup::Task>, total: usize) {
    let header = vec!["", "ID", "Created on", "Assigned", "Name"];
    let mut rows: Vec<Vec<crate::Cell>> = vec![];

    for task in tasks.iter() {
        let id: &str = &task.id;
        let name: &str = &task.name;
        let date_created: &str = &task.date_created;
        let status = &task
            .status
            .status
            .to_uppercase()
            .chars()
            .next()
            .unwrap_or('-');
        let assignees: Vec<String> = task
            .assignees
            .iter()
            .map(|assignee| format!("{} {}", assignee.username, assignee.id).to_string())
            .collect();

        // break hex color to ansi
        let hex_color = task.status.color.trim_start_matches("#");
        let r = u8::from_str_radix(&hex_color[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex_color[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex_color[4..6], 16).unwrap_or(0);
        let row = vec![
            crate::Cell::new(status)
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(comfy_table::Color::Rgb { r, g, b }),
            crate::Cell::from(id),
            crate::Cell::new(unix_date_to_readable(date_created)),
            crate::Cell::new(assignees.join(",")),
            crate::Cell::new(name),
        ];
        rows.push(row);
    }
    render_table(header, rows);
    println!("Showing {} of {}.", tasks.len(), total);
}
