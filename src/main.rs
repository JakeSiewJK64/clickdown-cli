use chrono::DateTime;
use clap::{Parser, ValueEnum};
use comfy_table::{
    Cell, ContentArrangement, Row, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use inquire::{Select, Text};
use std::{
    fs::{self},
    path::PathBuf,
};

use crate::clickup::SubmitCommentPayload;

pub mod clickup;
pub mod token_handler;
pub mod utils;

// when updating a task, you can choose the following to update
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Domain {
    Status,
    Name,
    Comment,
}

// CREATE directive.When provided, creates a new task.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Add {
    Task,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    token: Option<String>,

    #[arg(long)]
    modify: Option<Domain>,

    #[arg(long)]
    add: Option<Add>,

    #[arg(long, default_value = "")]
    team_id: String,

    #[arg(short, long, default_value = "")]
    space_id: String,

    #[arg(short, long, default_value = "")]
    folder_id: String,

    #[arg(short, long, default_value = "", required_if_eq("add", "task"))]
    list_id: String,

    #[arg(short, long, default_value = "")]
    task_id: String,

    /// if provided, filters tasks by status
    #[arg(long, default_value = "")]
    status: String,

    /// if provided, filters tasks by search query
    #[arg(long, default_value = "")]
    search: String,

    /// if provided, filters tasks by asignee
    /// currently filtering by only 1 assignee supported.
    #[arg(long, default_value = "")]
    assignee: String,

    /// if provided, gets comments for a thread
    #[arg(long, default_value = "")]
    thread_id: String,
}

fn process_add(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // handle create actions
    if let Some(add) = &args.add {
        match add {
            Add::Task => {
                let list_id = &args.list_id;
                let task_name = inquire::Text::new("Enter name of task: ")
                    .prompt()
                    .expect("There was a problem reading input for task name.");

                clickup::create_task(list_id, &task_name)
                    .expect("There was a problem creating task.");

                return Ok(());
            }
        }
    }
    Ok(())
}

fn process_modify(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    // handle modify actions
    // skip this if statement if modify is not provided
    if let Some(modify) = &args.modify {
        match modify {
            Domain::Name => {
                let Ok(task) = clickup::get_task(&args.task_id) else {
                    return Ok(());
                };
                let Ok(name_input) =
                    Text::new(format!("Enter new title for {}: ", task.name).as_str()).prompt()
                else {
                    return Ok(());
                };
                let updated_task = clickup::update_task(
                    &task.id,
                    clickup::UpdateTaskPayload {
                        status: task.status.status,
                        name: name_input,
                    },
                )
                .expect("There was a problem updating this task.");

                clickup::print_task_details(
                    updated_task,
                    clickup::Comments {
                        comments: Vec::new(),
                    },
                );

                return Ok(());
            }
            Domain::Status => {
                // get task by id
                let Ok(task) = clickup::get_task(&args.task_id) else {
                    return Ok(());
                };
                // get statuses in a folder
                let Ok(folder) = clickup::get_folder(&task.folder.id) else {
                    return Ok(());
                };
                let statuses: Vec<&str> = folder
                    .statuses
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .map(|status| status.status.as_str())
                    .collect();

                let Ok(status_input) = Select::new(
                    format!("Select status for {}: ", task.name).as_str(),
                    statuses,
                )
                .prompt() else {
                    return Ok(());
                };

                let updated_task = clickup::update_task(
                    &args.task_id,
                    clickup::UpdateTaskPayload {
                        status: status_input.to_string(),
                        name: task.name,
                    },
                )
                .expect("There was a problem updating task status.");
                clickup::print_task_details(
                    updated_task,
                    clickup::Comments {
                        comments: Vec::new(),
                    },
                );
                return Ok(());
            }
            Domain::Comment => {
                // get comment input
                let Ok(comment) = inquire::Editor::new("Enter comment").prompt() else {
                    return Ok(());
                };

                println!("your comment: {}", comment);

                match inquire::Confirm::new("Send comment?")
                    .with_help_message("'y' for yes or 'n' for no")
                    .prompt()
                {
                    Ok(true) => {
                        // send comment
                        clickup::submit_comment(
                            &args.task_id,
                            SubmitCommentPayload {
                                notify_all: false,
                                comment_text: comment,
                            },
                        )
                        .expect("There was a problem submitting comment");

                        // fetching for task by task id
                        let Ok(comments) = clickup::get_task_comments(&args.task_id) else {
                            return Ok(());
                        };

                        for comment in comments.comments.into_iter() {
                            clickup::print_comment(comment);
                        }
                    }
                    Ok(false) => {
                        println!("cancelled")
                    }
                    Err(_) => {
                        println!("there was a problem submitting comment.")
                    }
                }

                return Ok(());
            }
        }
    }

    Ok(())
}

fn process_get(args: &Args, table: &mut Table) -> Result<(), Box<dyn std::error::Error>> {
    // handle fetch actions
    if !args.thread_id.is_empty() {
        let thread = clickup::get_thread(args.thread_id.as_str()).unwrap_or(clickup::Comments {
            comments: Vec::new(),
        });

        for comment in thread.comments.into_iter() {
            clickup::print_comment(comment);
        }

        return Ok(());
    }

    // if space id provided, fetch for spaces given team_id
    if !args.team_id.is_empty() {
        let spaces = clickup::get_spaces(args.team_id.as_str())
            .unwrap_or(clickup::Spaces { spaces: Vec::new() });
        table.set_header(Row::from(vec!["ID", "Name"]));
        for space in spaces.spaces.iter() {
            let id: &str = &space.id;
            let name: &str = &space.name;
            table.add_row(vec![id, name]);
        }
        println!("{}", table);
        return Ok(());
    }

    if !args.folder_id.is_empty() {
        let lists =
            clickup::get_lists(&args.folder_id).unwrap_or(clickup::Lists { lists: Vec::new() });
        table.set_header(Row::from(vec!["ID", "Name", "Task count"]));
        for space in lists.lists.iter() {
            let id: &str = &space.id;
            let name: &str = &space.name;
            let task_count: usize = space.task_count.unwrap_or_default();
            table.add_row(vec![id, name, task_count.to_string().as_str()]);
        }
        println!("{}", table);
        return Ok(());
    }

    if !args.list_id.is_empty() {
        // populate assignees from terminal input
        let filters = clickup::TaskListsFilters {
            assignees: if args.assignee.is_empty() {
                Vec::new()
            } else {
                let Ok(assignee_ids) = args.assignee.parse::<usize>() else {
                    println!("There was a problem parsing assignee ID");
                    return Ok(());
                };

                vec![assignee_ids]
            },
        };
        let mut tasks = clickup::get_tasks(&args.list_id, filters).unwrap();
        let total = tasks.tasks.len();
        table.set_header(Row::from(vec!["", "ID", "Created on", "Assigned", "Name"]));

        // filters tasks by status
        if !args.status.is_empty() {
            tasks.tasks.retain(|task| {
                task.status
                    .status
                    .to_lowercase()
                    .contains(args.status.to_lowercase().as_str())
            })
        };

        // filter tasks by search query
        if !args.search.is_empty() {
            tasks.tasks.retain(|task| {
                task.name
                    .to_lowercase()
                    .contains(args.search.to_lowercase().as_str())
            })
        };

        for task in tasks.tasks.iter() {
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
            table.add_row(vec![
                Cell::new(status)
                    .add_attribute(comfy_table::Attribute::Bold)
                    .fg(comfy_table::Color::Rgb { r, g, b }),
                Cell::from(id),
                Cell::new(utils::unix_date_to_readable(date_created)),
                Cell::new(assignees.join(",")),
                Cell::new(name),
            ]);
        }
        println!("{}", table);
        println!("Showing {} of {}.", tasks.tasks.len(), total);
        return Ok(());
    }

    if !args.space_id.is_empty() {
        let folders = clickup::get_folders(args.space_id.as_str()).unwrap_or(clickup::Folders {
            folders: Vec::new(),
        });

        table.set_header(Row::from(vec!["ID", "Name", "Task count"]));
        for folder in folders.folders.iter() {
            let id: &str = &folder.id;
            let name: &str = &folder.name;
            let task_count: &str = folder.task_count.as_deref().unwrap_or_default();
            table.add_row(vec![id, name, task_count]);
        }
        println!("{}", table);
        return Ok(());
    }

    if !args.task_id.is_empty() {
        let Ok(task) = clickup::get_task(args.task_id.as_str()) else {
            println!("Could not find task with ID {}", args.task_id);
            return Ok(());
        };
        let comments = clickup::get_task_comments(args.task_id.as_str()).unwrap();

        clickup::print_task_details(task, comments);
        return Ok(());
    }

    // default if no ids given, just fetch for workspaces
    let workspaces = clickup::get_authorized_workspaces()
        .unwrap_or(clickup::AuthorizedWorkspaces { teams: Vec::new() });
    table.set_header(Row::from(vec!["ID", "Name"]));
    for workspace in workspaces.teams.iter() {
        let name: &str = &workspace.name;
        let id: &str = &workspace.id;
        table.add_row(vec![id, name]);
    }

    println!("{}", table);
    Ok(())
}

fn main() -> color_eyre::Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    // get input arguments
    let args = Args::parse();
    let token_input = &args.token;

    if let Some(token) = token_input {
        token_handler::save_token(token.as_str())?;
    }

    if args.add.is_some() {
        process_add(&args)?;
        return Ok(());
    }

    if args.modify.is_some() {
        process_modify(&args)?;
        return Ok(());
    }

    process_get(&args, &mut table)?;
    Ok(())
}
