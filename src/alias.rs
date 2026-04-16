use std::{collections::HashMap, io::Read};

// CREATE directive.When provided, creates a new task.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum AliasType {
    Task,
    TaskDetails,
}

/// its args but... only fields that matter
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct ArgsDTO {
    pub team_id: String,
    pub space_id: String,
    pub folder_id: String,
    pub list_id: String,
    pub task_id: String,
    pub status: String,
    pub search: String,
    pub assignee: String,
    pub thread_id: String,
}

/// what you save
#[derive(serde::Deserialize, serde::Serialize)]
pub struct AliasEntity {
    pub alias_type: AliasType,
    pub name: String,
    pub args: ArgsDTO,
}

impl Default for AliasEntity {
    fn default() -> Self {
        Self {
            args: ArgsDTO {
                ..Default::default()
            },
            alias_type: AliasType::Task,
            name: Default::default(),
        }
    }
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

fn write_alias_to_file(
    mapping: HashMap<usize, AliasEntity>,
) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn save_alias(payload: AliasEntity) -> Result<(), Box<dyn std::error::Error>> {
    println!("Alias provided, saving as {}", payload.name);

    //  get existing aliases
    let mut mapping = get_alias_mapping_from_file()?;
    mapping.insert(mapping.len() + 1, payload);

    write_alias_to_file(mapping)
}

pub fn delete_alias(alias_id: usize) -> Result<(), Box<dyn std::error::Error>> {
    //  get existing aliases
    let mut mapping = get_alias_mapping_from_file()?;
    mapping.remove(&alias_id);

    write_alias_to_file(mapping)
}

pub fn print_aliases() -> Result<(), Box<dyn std::error::Error>> {
    println!("Saved aliases:");
    let mappings = get_alias_mapping_from_file()?;

    println!("{:<15} {:<15} {:<15}", "ID", "Type", "Name");
    for (id, alias) in &mappings {
        let name = &alias.name;
        let alias_type: &str = match alias.alias_type {
            AliasType::Task => "task",
            AliasType::TaskDetails => "task_details",
        };
        println!("{:<15} {:<15} {:<15}", id, alias_type, name);
    }

    Ok(())
}

pub fn run_alias(alias_id: &usize) -> Result<(), Box<dyn std::error::Error>> {
    let mappings: HashMap<usize, AliasEntity> = get_alias_mapping_from_file()?;

    if !mappings.contains_key(alias_id) {
        eprintln!("No alias matches ID: {}", alias_id);
        return Ok(());
    }

    let alias: &AliasEntity = mappings.get(alias_id).unwrap();
    let alias_name = &alias.name;

    println!("Running alias: {}", alias_name);
    crate::process_get(&crate::Args {
        folder_id: alias.args.folder_id.to_string(),
        team_id: alias.args.team_id.to_string(),
        space_id: alias.args.space_id.to_string(),
        list_id: alias.args.list_id.to_string(),
        task_id: alias.args.task_id.to_string(),
        status: alias.args.status.to_string(),
        search: alias.args.search.to_string(),
        assignee: alias.args.assignee.to_string(),
        thread_id: alias.args.thread_id.to_string(),
        ..Default::default()
    })?;
    Ok(())
}
