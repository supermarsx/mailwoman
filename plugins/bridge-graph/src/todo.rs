//! Microsoft To-Do over Graph — task lists + tasks. Like calendar, there is no To-Do
//! export in the frozen WIT world, so this rides no plugin boundary today; it is
//! implemented + fixture-tested so the mapping is ready for a future task WIT seam.

use crate::graph::{GraphClient, Result, Transport};
use crate::model::{TodoListsResponse, TodoTasksResponse};

/// A To-Do list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoListInfo {
    pub id: String,
    pub name: String,
    pub owner: bool,
}

/// A To-Do task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoTaskInfo {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub due: Option<String>,
}

/// `GET /me/todo/lists` → the account's To-Do lists.
pub fn list_lists<T: Transport>(client: &GraphClient<'_, T>) -> Result<Vec<TodoListInfo>> {
    let resp: TodoListsResponse = client.get_json("/me/todo/lists")?;
    Ok(resp
        .value
        .into_iter()
        .map(|l| TodoListInfo {
            id: l.id,
            name: l.display_name.unwrap_or_default(),
            owner: l.is_owner.unwrap_or(false),
        })
        .collect())
}

/// `GET /me/todo/lists/{id}/tasks` → the tasks in a list.
pub fn list_tasks<T: Transport>(
    client: &GraphClient<'_, T>,
    list_id: &str,
) -> Result<Vec<TodoTaskInfo>> {
    let resp: TodoTasksResponse = client.get_json(&format!("/me/todo/lists/{list_id}/tasks"))?;
    Ok(resp
        .value
        .into_iter()
        .map(|t| TodoTaskInfo {
            id: t.id,
            title: t.title.unwrap_or_default(),
            completed: t.status.as_deref() == Some("completed"),
            due: t.due_date_time.and_then(|d| d.date_time),
        })
        .collect())
}
