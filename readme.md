## Description

Simple CLI application to interface with ClickUp APIs.

Built for my own use but feel free to try.

## Features

- [x] List workspaces, spaces, folders, lists, task and task details.
- [x] List task comments and threads.
- [x] Update task status.
- [x] Update task title.
- [x] Filter task by status or title.
- [x] Add task comment.
- [ ] (optional) Pagination.
- [ ] (optional) Respond to comments.
- [ ] (optional) Render images to terminal.
- [ ] (optional) Assign members to task.
- [ ] (optional) Caching because each call is expensive if the task list is long.

## Usage

```
Usage: clickdown-cli [OPTIONS]

Options:
      --token <TOKEN>
      --modify
      --domain <DOMAIN>        [default: status] [possible values: status, name]
      --team-id <TEAM_ID>      [default: ""]
  -s, --space-id <SPACE_ID>    [default: ""]
  -f, --folder-id <FOLDER_ID>  [default: ""]
  -l, --list-id <LIST_ID>      [default: ""]
  -t, --task-id <TASK_ID>      [default: ""]
      --status <STATUS>        if provided, filters tasks by status [default: ""]
      --search <SEARCH>        if provided, filters tasks by search query [default: ""]
      --thread-id <THREAD_ID>  if provided, gets comments for a thread [default: ""]
  -h, --help                   Print help
  -V, --version                Print version

```

## Screenshots

### Task List:

![ss1](./screenshots/ss1.png)

### Task details:

![ss2](./screenshots/ss2.png)
