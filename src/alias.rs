use std::{collections::HashMap, io::Read};

// CREATE directive.When provided, creates a new task.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum AliasType {
    Task,
}

/// what you save
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AliasEntity {
    pub list_id: String,
    pub status: Option<String>,
    pub alias_type: AliasType,
    pub name: String,
}

/// what you consume
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AliasEntityDTO {
    pub list_id: String,
    pub status: Option<String>,
    pub alias_type: AliasType,
    pub name: String,
}

fn get_alias_mapping_from_file() -> Result<HashMap<usize, AliasEntity>, Box<dyn std::error::Error>>
{
    let clickdown_folder_path = get_alias_file_path_buf()?;
    let mut file_reader = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&clickdown_folder_path)?;

    // seed sample alias entry
    let mut string_content = String::from("");
    let mut mappings: HashMap<usize, AliasEntity> = HashMap::new();
    file_reader.read_to_string(&mut string_content)?;

    if !string_content.is_empty() {
        mappings = serde_json::from_str(&string_content)?
    }

    Ok(mappings)
}

fn get_alias_file_path_buf() -> Result<crate::PathBuf, Box<dyn std::error::Error>> {
    let home_directory = crate::utils::get_home_dir();
    let mut clickdown_folder_path = crate::PathBuf::from(home_directory);

    // append aliases file
    clickdown_folder_path.push(".config/clickdown/aliases.json");
    Ok(clickdown_folder_path)
}

pub fn save_alias(
    alias_name: &str,
    payload: AliasEntityDTO,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Alias provided, saving as {}", alias_name);

    //  get existing aliases
    let mut mapping = get_alias_mapping_from_file()?;
    mapping.insert(
        mapping.len(),
        AliasEntity {
            name: payload.name,
            list_id: payload.list_id,
            alias_type: payload.alias_type,
            status: Some(payload.status.unwrap_or("".to_string())),
        },
    );

    // save content to json file
    println!("Saving content alias, please wait...");
    let serialized_args = serde_json::to_string_pretty(&mapping)?;
    let clickdown_folder_path = get_alias_file_path_buf()?;

    match std::fs::write(clickdown_folder_path, serialized_args) {
        Ok(value) => {
            println!("Saved alias at ~/.config/clickdown/aliases.json");
            value
        }
        Err(err) => {
            println!("There was a proble saving alias.\n{}", err)
        }
    };

    Ok(())
}

pub fn print_aliases() -> Result<(), Box<dyn std::error::Error>> {
    println!("Saved aliases:");
    let mappings = get_alias_mapping_from_file()?;

    println!("{:<15} {:<15} {:<15}", "ID", "Type", "Name");
    for (id, alias) in &mappings {
        let name = &alias.name;
        let alias_type: &str = match alias.alias_type {
            AliasType::Task => "task",
        };
        println!("{:<15} {:<15} {:<15}", id, alias_type, name);
    }

    Ok(())
}

pub fn run_alias(
    alias_id: &usize,
    table: &mut crate::Table,
) -> Result<(), Box<dyn std::error::Error>> {
    let mappings: HashMap<usize, AliasEntity> = get_alias_mapping_from_file()?;
    let alias: &AliasEntity = mappings.get(alias_id).unwrap();
    let alias_name = &alias.name;

    println!("Running alias: {}", alias_name);

    match &alias.alias_type {
        AliasType::Task => {
            let status_filter = &alias.status;
            let tasks = crate::clickup::get_tasks(
                &alias.list_id,
                crate::clickup::TaskListsFilters {
                    assignees: vec![],
                    statuses: match status_filter {
                        Some(status) => vec![status.to_string()],
                        None => {
                            vec![]
                        }
                    },
                },
            )?;

            let total = tasks.tasks.len();
            crate::utils::render_task_table(table, tasks.tasks, total);
            Ok(())
        }
    }
}
