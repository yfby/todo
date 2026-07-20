use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskListCollection {
    task_lists: Vec<TaskList>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskList {
    name: String,
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    completed: bool,
    task: String,
    description: String,
}

impl Default for TaskListCollection {
    fn default() -> Self {
        TaskListCollection {
            task_lists: Vec::new(),
        }
    }
}

impl TaskListCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_list(&mut self, list: TaskList) {
        self.task_lists.push(list);
    }

    // FIX: Remove by index dont use name
    pub fn remove_list(&mut self, name: &str) {
        self.task_lists.retain(|l| l.name != name);
    }

    // TODO: get

    pub fn get_list(&self, name: &str) -> Option<&TaskList> {
        self.task_lists.iter().find(|l| l.name == name)
    }

    pub fn get_list_mut(&mut self, name: &str) -> Option<&mut TaskList> {
        self.task_lists.iter_mut().find(|l| l.name == name)
    }

    pub fn lists(&self) -> &[TaskList] {
        &self.task_lists
    }
}

impl TaskList {
    pub fn new(name: &str) -> Self {
        TaskList {
            name: name.to_string(),
            tasks: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, name: &str) {
        self.tasks.retain(|t| t.task != name);
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
}

impl Task {
    pub fn new(task: &str, description: &str) -> Self {
        Task {
            completed: false,
            task: task.to_string(),
            description: description.to_string(),
        }
    }

    pub fn toggle(&mut self) {
        self.completed = !self.completed;
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn task(&self) -> &str {
        &self.task
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

pub fn save_to_file(list: &TaskListCollection, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(list).expect("Failed to serialize TaskListCollection");
    fs::write(path, json)
}

pub fn load_from_file(path: &str) -> std::io::Result<TaskListCollection> {
    let data = fs::read_to_string(path)?;
    let task_lists = serde_json::from_str(&data).expect("Failed to deserialize TaskListCollection");
    Ok(task_lists)
}

pub fn load_or_default(path: &str) -> TaskListCollection {
    if Path::new(path).exists() {
        load_from_file(path).unwrap_or_default()
    } else {
        TaskListCollection::new()
    }
}
