//! `serde` structs for the Microsoft Graph JSON surface the bridge consumes. Only
//! the fields the mapping uses are modeled; unknown fields are ignored (`serde`
//! default), so the bridge tolerates Graph adding properties.

use serde::Deserialize;

// ── Mail folders ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailFolder {
    pub id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub well_known_name: Option<String>,
    #[serde(default)]
    pub parent_folder_id: Option<String>,
    #[serde(default)]
    pub total_item_count: u32,
    #[serde(default)]
    pub unread_item_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MailFoldersResponse {
    #[serde(default)]
    pub value: Vec<MailFolder>,
    #[serde(rename = "@odata.nextLink", default)]
    pub next_link: Option<String>,
}

// ── Messages (delta + metadata) ───────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct FollowupFlag {
    #[serde(rename = "flagStatus", default)]
    pub flag_status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphMessage {
    pub id: String,
    /// Present on a removed entry in a delta response (`@removed`).
    #[serde(rename = "@removed", default)]
    pub removed: Option<serde_json::Value>,
    #[serde(default)]
    pub is_read: Option<bool>,
    #[serde(default)]
    pub flag: Option<FollowupFlag>,
    /// `focused` | `other` — the Focused-Inbox classification (plan §2.5).
    #[serde(default)]
    pub inference_classification: Option<String>,
    #[serde(default)]
    pub received_date_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagesDeltaResponse {
    #[serde(default)]
    pub value: Vec<GraphMessage>,
    #[serde(rename = "@odata.deltaLink", default)]
    pub delta_link: Option<String>,
    #[serde(rename = "@odata.nextLink", default)]
    pub next_link: Option<String>,
}

// ── Contacts / directory / people (GAL) ───────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct EmailAddress {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email_addresses: Vec<EmailAddress>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactsResponse {
    #[serde(default)]
    pub value: Vec<Contact>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub scored_email_addresses: Vec<ScoredEmail>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoredEmail {
    #[serde(default)]
    pub address: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PeopleResponse {
    #[serde(default)]
    pub value: Vec<Person>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryUser {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub mail: Option<String>,
    #[serde(default)]
    pub user_principal_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsersResponse {
    #[serde(default)]
    pub value: Vec<DirectoryUser>,
}

// ── Calendar / events / rooms / free-busy ─────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Calendar {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub can_edit: Option<bool>,
    #[serde(default)]
    pub owner: Option<EmailAddress>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarsResponse {
    #[serde(default)]
    pub value: Vec<Calendar>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DateTimeTimeZone {
    #[serde(rename = "dateTime", default)]
    pub date_time: Option<String>,
    #[serde(default)]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    #[serde(rename = "@removed", default)]
    pub removed: Option<serde_json::Value>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub start: Option<DateTimeTimeZone>,
    #[serde(default)]
    pub end: Option<DateTimeTimeZone>,
    #[serde(default)]
    pub location: Option<Location>,
    #[serde(default)]
    pub is_all_day: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventsDeltaResponse {
    #[serde(default)]
    pub value: Vec<Event>,
    #[serde(rename = "@odata.deltaLink", default)]
    pub delta_link: Option<String>,
    #[serde(rename = "@odata.nextLink", default)]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub email_address: Option<String>,
    #[serde(default)]
    pub capacity: Option<u32>,
    #[serde(default)]
    pub building: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoomsResponse {
    #[serde(default)]
    pub value: Vec<Room>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInformation {
    #[serde(default)]
    pub schedule_id: Option<String>,
    #[serde(default)]
    pub availability_view: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetScheduleResponse {
    #[serde(default)]
    pub value: Vec<ScheduleInformation>,
}

// ── To-Do ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoList {
    pub id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub is_owner: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TodoListsResponse {
    #[serde(default)]
    pub value: Vec<TodoList>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoTask {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub due_date_time: Option<DateTimeTimeZone>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TodoTasksResponse {
    #[serde(default)]
    pub value: Vec<TodoTask>,
}
