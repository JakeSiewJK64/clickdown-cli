use chrono::DateTime;
use clap::{Parser, ValueEnum};
use comfy_table::{Cell, ContentArrangement, Row, Table, presets::NOTHING};
use inquire::{Select, Text};
use std::path::PathBuf;
use std::collections::HashMap;

use crate::utils::render_table;

pub mod alias;
pub mod clickup;
pub mod token_handler;
pub mod utils;

// when updating a task, you can choose the following to update
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Domain {
    Status,
    Name,
    Comment,
    Thread,
    Assignee,
}

// CREATE directive.When provided, creates a new task.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Add {
    Task,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Delete {
    Alias,
}

#[derive(Parser, Default)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value = "")]
    token: String,

    #[arg(long)]
    modify: Option<Domain>,

    #[arg(long)]
    add: Option<Add>,

    #[arg(long)]
    delete: Option<Delete>,

    #[arg(long, default_value = "")]
    team_id: String,

    #[arg(short, long, default_value = "")]
    space_id: String,

    #[arg(short, long, default_value = "")]
    folder_id: String,

    #[arg(short, long, default_value = "", required_if_eq("add", "task"))]
    list_id: String,

    #[arg(
        short, 
        long, 
        default_value = "", 
        required_if_eq_all([
            ("modify", "assignee"),
        ])
    )]
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

    /// if provided, saves request to given name.
    #[arg(long, default_value = "")]
    alias: String,

    /// if provided true, prints all saved aliases.
    #[arg(long, default_value = "false")]
    list_alias: bool,

    /// if provided, executes a stored alias, does nothing if no matches found.
    #[arg(long, default_value = "")]
    run: String,

    #[arg(long, default_value = "", required_if_eq("delete", "alias"))]
    alias_id: String,
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

                process_get(&Args {
                    list_id: list_id.to_string(),
                    ..Default::default()
                })?;
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
                            clickup::SubmitCommentPayload {
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
            Domain::Thread => {
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
                        clickup::submit_thread_comment(
                            &args.thread_id,
                            clickup::SubmitCommentPayload {
                                notify_all: false,
                                comment_text: comment,
                            },
                        )
                        .expect("There was a problem submitting comment");

                        let Ok(comments) = clickup::get_thread(&args.thread_id) else {
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
            Domain::Assignee => {
                // todo: fetch task by id
                let Ok(task) = clickup::get_task(&args.task_id) else {
                    return Ok(());
                };

                // todo: fetch all assignees
                let Ok(members) = clickup::get_task_members(&task.list.id) else {
                    return Ok(());
                };

                let mut member_mappings: HashMap<String, usize> = HashMap::new();

                for member in members.members {
                    member_mappings.insert(member.username, member.id);
                }

                // todo: get member input
                let Ok(target_member) = Select::new(
                    "Select member to assign: ".to_string().as_str(),
                    member_mappings.keys().collect(),
                )
                .prompt() else {
                    return Ok(());
                };

                // todo: check if user is already assigned, if so skip
                if task.assignees.iter().any(|user|user.username == *target_member) {
                    println!("{} already assigned to task {}", target_member, task.name);
                    return Ok(())
                }

                if let Some(member_id) = member_mappings.get(target_member) {
                    println!("Selected member: {} ({})", target_member, member_id);
                    clickup::assign_task(
                        &task.id,
                        vec![*member_id],
                    )?;
                }

                // todo: display task details
                clickup::print_task_details(task, clickup::Comments { comments: vec![] });

                return Ok(())
            }
        }
    }

    Ok(())
}

fn process_get(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
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
        let headers: Vec<&str> = vec!["ID", "Name"];
        let mut rows: Vec<Vec<Cell>> = vec![];
        for space in spaces.spaces.iter() {
            let id: &str = &space.id;
            let name: &str = &space.name;
            rows.push(vec![Cell::from(id), Cell::from(name)]);
        }
        render_table(headers, rows);
        return Ok(());
    }

    if !args.folder_id.is_empty() {
        let lists =
            clickup::get_lists(&args.folder_id).unwrap_or(clickup::Lists { lists: Vec::new() });
        let header = vec!["ID", "Name", "Task count"];
        let mut rows: Vec<Vec<Cell>> = vec![];

        for space in lists.lists.iter() {
            let id: &str = &space.id;
            let name: &str = &space.name;
            let task_count: usize = space.task_count.unwrap_or_default();
            rows.push(vec![
                Cell::from(id),
                Cell::from(name),
                Cell::from(task_count.to_string().as_str()),
            ]);
        }
        render_table(header, rows);
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
            statuses: if args.status.is_empty() {
                vec![]
            } else {
                vec![args.status.to_string()]
            },
        };
        let mut tasks = clickup::get_tasks(&args.list_id, filters).unwrap();
        let total = tasks.tasks.len();

        // filter tasks by search query
        if !args.search.is_empty() {
            tasks.tasks.retain(|task| {
                task.name
                    .to_lowercase()
                    .contains(args.search.to_lowercase().as_str())
            })
        };

        utils::render_task_table(tasks.tasks, total);

        // check if save alias is provided, if so, save to file
        if !args.alias.is_empty() {
            alias::save_alias(alias::AliasEntity {
                name: args.alias.to_string(),
                alias_type: alias::AliasType::Task,
                args: crate::alias::ArgsDTO {
                    list_id: args.list_id.to_string(),
                    status: args.status.to_string(),
                    assignee: args.assignee.to_string(),
                    ..Default::default()
                },
            })?;
        }

        return Ok(());
    }

    if !args.space_id.is_empty() {
        let folders = clickup::get_folders(args.space_id.as_str()).unwrap_or(clickup::Folders {
            folders: Vec::new(),
        });

        let header: Vec<&str> = vec!["ID", "Name", "Task count"];
        let mut rows: Vec<Vec<Cell>> = vec![];
        for folder in folders.folders.iter() {
            let id: &str = &folder.id;
            let name: &str = &folder.name;
            let task_count: &str = folder.task_count.as_deref().unwrap_or_default();
            rows.push(vec![
                Cell::from(id),
                Cell::from(name),
                Cell::from(task_count),
            ]);
        }
        render_table(header, rows);
        return Ok(());
    }

    if !args.task_id.is_empty() {
        let Ok(task) = clickup::get_task(args.task_id.as_str()) else {
            println!("Could not find task with ID {}", args.task_id);
            return Ok(());
        };
        let comments = clickup::get_task_comments(args.task_id.as_str()).unwrap();

        clickup::print_task_details(task, comments);

        // check if save alias is provided, if so, save to file
        if !args.alias.is_empty() {
            alias::save_alias(alias::AliasEntity {
                name: args.alias.to_string(),
                alias_type: alias::AliasType::TaskDetails,
                args: alias::ArgsDTO {
                    task_id: args.task_id.to_string(),
                    ..Default::default()
                },
            })?;
        }

        return Ok(());
    }

    // default if no ids given, just fetch for workspaces
    let workspaces = clickup::get_authorized_workspaces()
        .unwrap_or(clickup::AuthorizedWorkspaces { teams: Vec::new() });
    let headers: Vec<&str> = vec!["ID", "Name"];
    let mut rows: Vec<Vec<Cell>> = vec![];
    for workspace in workspaces.teams.iter() {
        let name: &str = &workspace.name;
        let id: &str = &workspace.id;
        rows.push(vec![Cell::from(id), Cell::from(name)]);
    }
    render_table(headers, rows);
    Ok(())
}

fn process_delete(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(delete) = &args.delete {
        match delete {
            Delete::Alias => {
                if !&args.alias_id.parse::<usize>().is_ok() {
                    eprintln!("Invalid alias id provided: {}", &args.alias_id);
                    return Ok(());
                }

                let alias_id = args.alias_id.parse::<usize>().unwrap();
                alias::delete_alias(alias_id)?;
                return Ok(());
            }
        }
    }

    Ok(())
}

fn main() -> color_eyre::Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    // get input arguments
    let args = Args::parse();

    token_handler::save_token(&args.token)?;

    if args.add.is_some() {
        process_add(&args)?;
        return Ok(());
    }

    if args.delete.is_some() {
        process_delete(&args)?;
        return Ok(());
    }

    if args.modify.is_some() {
        process_modify(&args)?;
        return Ok(());
    }

    if args.list_alias {
        alias::print_aliases()?;
        return Ok(());
    }

    if args.run.parse::<usize>().is_ok() {
        let alias_id = &args.run.parse::<usize>()?;
        alias::run_alias(alias_id)?;
        return Ok(());
    }

    process_get(&args)?;
    Ok(())
}
