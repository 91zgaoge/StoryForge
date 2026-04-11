use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub assignee: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Blocked,
}

pub fn create_task(
    id: &str,
    title: &str,
    dependencies: Vec<String>,
    assignee: &str,
) -> Task {
    Task {
        id: id.to_string(),
        title: title.to_string(),
        description: title.to_string(),
        status: if dependencies.is_empty() {
            TaskStatus::Ready
        } else {
            TaskStatus::Pending
        },
        assignee: assignee.to_string(),
        dependencies,
    }
}
