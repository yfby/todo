use dirs::data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct TaskListCollection {
    task_lists: Vec<TaskList>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskList {
    name: String,
    tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    completed: bool,
    task: String,
    description: Option<String>,
}

impl TaskListCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_list(&mut self, list: TaskList) {
        self.task_lists.push(list);
    }

    pub fn remove_list(&mut self, index: usize) -> bool {
        if index < self.task_lists.len() {
            self.task_lists.remove(index);
            true
        } else {
            false
        }
    }

    pub fn get_list_names(&self) -> Vec<String> {
        self.task_lists
            .iter()
            .map(|tl| tl.name().to_string())
            .collect()
    }

    pub fn get_list(&self, index: usize) -> Option<&TaskList> {
        self.task_lists.get(index)
    }

    pub fn get_list_mut(&mut self, index: usize) -> Option<&mut TaskList> {
        self.task_lists.get_mut(index)
    }

    pub fn lists(&self) -> &[TaskList] {
        &self.task_lists
    }

    pub fn lists_mut(&mut self) -> &mut [TaskList] {
        &mut self.task_lists
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

    pub fn rename(&mut self, new_name: &str) {
        self.name = new_name.to_owned();
    }

    pub fn get_task(&self, index: usize) -> Option<&Task> {
        self.tasks.get(index)
    }

    pub fn get_task_mut(&mut self, index: usize) -> Option<&mut Task> {
        self.tasks.get_mut(index)
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, index: usize) -> bool {
        if index < self.tasks.len() {
            self.tasks.remove(index);
            true
        } else {
            false
        }
    }

    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn tasks_mut(&mut self) -> &mut [Task] {
        &mut self.tasks
    }
}

impl Task {
    pub fn new(task: &str, description: Option<&str>) -> Self {
        Task {
            completed: false,
            task: task.to_string(),
            description: description.map(String::from),
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

    pub fn rename(&mut self, new_task: &str) {
        self.task = new_task.to_owned();
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn change_description(&mut self, description: Option<&str>) {
        self.description = description.map(String::from);
    }
}

pub fn save_file_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("todo").join("tasks.json"))
}

pub fn save_to_file(list: &TaskListCollection, path: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(list).map_err(std::io::Error::other)?;
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)
}

pub fn load_from_file(path: &str) -> std::io::Result<TaskListCollection> {
    let data = fs::read_to_string(path)?;
    let task_lists = serde_json::from_str(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(task_lists)
}

pub fn load_or_default(path: &str) -> TaskListCollection {
    if Path::new(path).exists() {
        load_from_file(path).unwrap_or_default()
    } else {
        TaskListCollection::new()
    }
}
