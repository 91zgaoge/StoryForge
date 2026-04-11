use crate::task::{Task, TaskStatus};
use std::collections::VecDeque;

pub struct TaskQueue {
    tasks: VecDeque<Task>,
    completed: Vec<String>,
    failed: Vec<String>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            completed: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push_back(task);
    }

    pub fn next_ready_task(&mut self) -> Option<Task> {
        let ready_index = self.tasks.iter().position(|t| {
            t.status == TaskStatus::Ready || 
            (t.status == TaskStatus::Pending && self.dependencies_satisfied(t))
        })?;
        
        self.tasks.remove(ready_index)
    }

    fn dependencies_satisfied(&self, task: &Task) -> bool {
        task.dependencies.iter()
            .all(|dep| self.completed.contains(dep))
    }

    pub fn mark_task_complete(&mut self, task_id: &str) {
        self.completed.push(task_id.to_string());
    }

    pub fn mark_task_failed(&mut self, task_id: &str) {
        self.failed.push(task_id.to_string());
        // Mark dependent tasks as blocked
        for task in &mut self.tasks {
            if task.dependencies.contains(&task_id.to_string()) {
                task.status = TaskStatus::Blocked;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}
