use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskLists {
    task_lists: Vec<TaskList>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskList {
    name: String,
    tasks: Vec<Task>,
}

impl TaskList {
    fn new(name: &str) -> Self {
        TaskList {
            name: name.to_string(),
            tasks: Vec::new(),
        }
    }

    fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    fn remove_task(&mut self, name: &str) {
        self.tasks.retain(|t| t.task != name);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    completed: bool,
    task: String,
    description: String,
}

pub fn save_to_file(list: &TaskLists, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(list).expect("Failed to serialize TaskList");
    fs::write(path, json)
}
