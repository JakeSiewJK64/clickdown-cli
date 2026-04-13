use std::collections::HashMap;

use colored::Colorize;

#[derive(serde::Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct Status {
    pub id: String,
    pub status: String,
    pub color: String,
}

#[derive(serde::Deserialize)]
pub struct AuthorizedWorkspaces {
    pub teams: Vec<Team>,
}

fn get_request_header() -> Result<reqwest::header::HeaderMap, reqwest::header::InvalidHeaderValue> {
    let token = crate::token_handler::get_clickdown_token().unwrap_or("".to_string());
    let mut headers = reqwest::header::HeaderMap::new();
    let auth_header = reqwest::header::HeaderValue::from_str(&token)?;

    headers.insert("Authorization", auth_header);
    Ok(headers)
}

pub fn get_authorized_workspaces() -> Result<AuthorizedWorkspaces, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get("https://api.clickup.com/api/v2/team")
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: AuthorizedWorkspaces = request.json()?;
    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    pub statuses: Vec<Status>,
}

#[derive(serde::Deserialize)]
pub struct Spaces {
    pub spaces: Vec<Space>,
}

pub fn get_spaces(team_id: &str) -> Result<Spaces, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get(format!(
            "https://api.clickup.com/api/v2/team/{}/space",
            team_id
        ))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Spaces = request.json()?;
    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub task_count: Option<String>,
    pub statuses: Option<Vec<Status>>,
}

#[derive(serde::Deserialize)]
pub struct Folders {
    pub folders: Vec<Folder>,
}

pub fn get_folders(space_id: &str) -> Result<Folders, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get(format!(
            "https://api.clickup.com/api/v2/space/{}/folder",
            space_id
        ))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Folders = request.json()?;
    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct List {
    pub id: String,
    pub name: String,
    pub task_count: Option<usize>,
}

#[derive(serde::Deserialize)]
pub struct Lists {
    pub lists: Vec<List>,
}

pub fn get_lists(folder_id: &str) -> Result<Lists, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get(format!(
            "https://api.clickup.com/api/v2/folder/{}/list",
            folder_id
        ))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Lists = request.json()?;
    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: Status,
    pub folder: Folder,
    pub date_created: String,
    pub assignees: Vec<User>,
}

#[derive(serde::Deserialize)]
pub struct Tasks {
    pub tasks: Vec<Task>,
}

#[derive(serde::Serialize)]
pub struct TaskListsFilters {
    pub assignees: Vec<usize>,
}

pub fn get_tasks(
    list_id: &str, // get_lists can just take ownership of the variable, parent might not need it
    filters: TaskListsFilters,
) -> Result<Tasks, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let Ok(mut url) = reqwest::Url::parse(
        format!("https://api.clickup.com/api/v2/list/{}/task", list_id).as_str(),
    ) else {
        println!("Error: there was a problem parsing request url.");
        return Ok(Tasks { tasks: Vec::new() });
    };

    if !filters.assignees.is_empty() {
        for assignee in filters.assignees {
            url.set_query(Some(format!("assignees[]={}", assignee).as_str()));
        }
    }

    let url_string = url.as_str();
    let headers = get_request_header()?;
    let request = client.get(url_string).headers(headers).send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Tasks = request.json()?;
    Ok(response)
}

pub fn create_task(list_id: &str, name: &str) -> Result<Task, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;

    // supports only name for now, probably may want to
    // convert this into a struct in the future.
    let mut body = HashMap::new();
    body.insert("name", name);

    let request = client
        .post(format!(
            "https://api.clickup.com/api/v2/list/{}/task",
            list_id
        ))
        .json(&body)
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Task = request.json()?;
    Ok(response)
}

pub fn get_task(task_id: &str) -> Result<Task, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get(format!("https://api.clickup.com/api/v2/task/{}", task_id))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Task = request.json()?;
    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct User {
    pub id: usize,
    pub username: String,
    pub color: String,
}

#[derive(serde::Deserialize)]
pub struct Comment {
    pub id: String,
    pub comment_text: String,
    pub user: User,
    pub date: String,
    pub reply_count: usize,
}

#[derive(serde::Deserialize)]
pub struct Comments {
    pub comments: Vec<Comment>,
}

pub fn get_task_comments(task_id: &str) -> Result<Comments, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get(format!(
            "https://api.clickup.com/api/v2/task/{}/comment",
            task_id
        ))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Comments = request.json()?;
    Ok(response)
}

pub fn get_thread(thread_id: &str) -> Result<Comments, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;
    let request = client
        .get(format!(
            "https://api.clickup.com/api/v2/comment/{}/reply",
            thread_id
        ))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Comments = request.json()?;
    Ok(response)
}

#[derive(serde::Serialize)]
pub struct UpdateTaskPayload {
    pub status: String,
    pub name: String,
}

pub fn update_task(
    task_id: &str,
    payload: UpdateTaskPayload,
) -> Result<Task, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;

    let request = client
        .put(format!("https://api.clickup.com/api/v2/task/{}", task_id))
        .headers(headers)
        .json(&payload)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Task = request.json()?;
    Ok(response)
}

#[derive(serde::Serialize)]
pub struct SubmitCommentPayload {
    pub comment_text: String,
    pub notify_all: bool,
}

pub fn submit_comment(
    task_id: &str,
    payload: SubmitCommentPayload,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;

    let request = client
        .post(format!(
            "https://api.clickup.com/api/v2/task/{}/comment",
            task_id
        ))
        .headers(headers)
        .json(&payload)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    Ok(())
}

pub fn print_task_details(task: Task, comments: Comments) {
    println!("Title: {} ({})", task.name, task.id);
    println!("URL: {}", task.url);
    println!(
        "Created on: {}",
        crate::utils::unix_date_to_readable(task.date_created.as_str())
    );
    print!("Status: ");
    println!(
        "{}",
        task.status.status.to_uppercase().color(task.status.color)
    );

    if comments.comments.is_empty() {
        return;
    }

    println!("Comments: ");
    println!("_________________________________________________________________________");
    for comment in comments.comments.into_iter() {
        print_comment(comment);
    }
}

pub fn get_folder(folder_id: &str) -> Result<Folder, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let headers = get_request_header()?;

    let request = client
        .get(format!(
            "https://api.clickup.com/api/v2/folder/{}",
            folder_id
        ))
        .headers(headers)
        .send()?;

    if request.status() != reqwest::StatusCode::OK {
        let response = request.text()?;
        eprintln!("{}", response);
        std::process::exit(1);
    }

    let response: Folder = request.json()?;
    Ok(response)
}

pub fn print_comment(comment: Comment) {
    // print comment author
    println!(
        "{} ({}) {} replies",
        comment
            .user
            .username
            .color(comment.user.color)
            .bold()
            .on_white(),
        comment.id,
        comment.reply_count
    );
    println!(
        "Date: {}",
        crate::utils::unix_date_to_readable(comment.date.as_str())
    );
    println!("{}", comment.comment_text);
    println!("_________________________________________________________________________");
}
