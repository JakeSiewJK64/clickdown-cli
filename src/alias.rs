use std::{collections::HashMap, fs::File, io::Read};

// CREATE directive.When provided, creates a new task.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum AliasType {
    Task,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct AliasEntity {
    pub list_id: String,
    pub status: Option<String>,
    pub alias_type: AliasType,
}

fn get_alias_mapping_from_file() -> Result<HashMap<String, AliasEntity>, Box<dyn std::error::Error>>
{
    let clickdown_folder_path = get_alias_file_path_buf()?;
    let mut file_reader = File::open(clickdown_folder_path)?;
    let mut string_content = String::new();
    file_reader.read_to_string(&mut string_content)?;
    let mappings: HashMap<String, AliasEntity> = serde_json::from_str(&string_content)?;

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
    payload: AliasEntity,
) -> Result<(), Box<dyn std::error::Error>> {
    let clickdown_folder_path = get_alias_file_path_buf()?;
    let mut mapping = get_alias_mapping_from_file()?;
    mapping.insert(alias_name.to_string(), payload);

    let serialized_args = serde_json::to_string_pretty(&mapping)?;

    match crate::fs::write(clickdown_folder_path, serialized_args) {
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
    for (name, alias) in &mappings {
        let alias_type: &str = match alias.alias_type {
            AliasType::Task => "task",
        };
        println!("- {name}\t{alias_type}");
    }

    Ok(())
}

pub fn run_alias(
    alias_name: &str,
    table: &mut crate::Table,
) -> Result<(), Box<dyn std::error::Error>> {
    let mappings: HashMap<String, AliasEntity> = get_alias_mapping_from_file()?;
    let alias: &AliasEntity = mappings.get(alias_name).unwrap();

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
