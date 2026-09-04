use crossterm::event::{KeyEvent, KeyModifiers};

use super::{App, CurrentInterface, WriteType};

impl App {
    pub(super) fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.current_interface {
            CurrentInterface::TaskMenu => self.key_event_task_menu(key_event),
            CurrentInterface::TaskBody => self.key_event_task_body(key_event),
            CurrentInterface::Write => self.key_event_write(key_event),
            CurrentInterface::Help => self.key_event_help(key_event),
            CurrentInterface::Exit => {} // TODO: confirm exit
        }
    }

    fn key_event_task_menu(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            // navigation
            (crossterm::event::KeyCode::Esc, KeyModifiers::NONE) => self.menu_state.select(None),
            (crossterm::event::KeyCode::Char('j'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Down, KeyModifiers::NONE) => {
                self.menu_state.select_next();
                self.task_state.select(None);
            }
            (crossterm::event::KeyCode::Char('k'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Up, KeyModifiers::NONE) => {
                self.menu_state.select_previous();
                self.task_state.select(None);
            }
            (crossterm::event::KeyCode::Char('l'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Right, KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Enter, KeyModifiers::NONE) => {
                self.current_interface = CurrentInterface::TaskBody;
                if self.task_state.selected().is_none() {
                    self.task_state.select(Some(0));
                }
            }

            // action
            (crossterm::event::KeyCode::Char('a'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.enter_write(WriteType::Menu, None);
            }
            (crossterm::event::KeyCode::Char('d'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Delete, KeyModifiers::NONE) => {
                if let Some(index) = self.menu_state.selected()
                    && self.task_collection.remove_list(index)
                {
                    self.task_state.select(None);
                    if self.task_collection.lists().is_empty() {
                        self.menu_state.select(None);
                    } else if index >= self.task_collection.lists().len() {
                        self.menu_state
                            .select(Some(self.task_collection.lists().len() - 1));
                    }
                }
            }
            (crossterm::event::KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                if let Some(index) = self.menu_state.selected()
                    && let Some(list) = self.task_collection.get_list(index)
                {
                    let name = list.name().to_string();
                    self.enter_write(WriteType::RenameMenu, Some(&name));
                }
            }

            // reorder
            (crossterm::event::KeyCode::Char('K'), KeyModifiers::SHIFT)
            | (crossterm::event::KeyCode::Down, KeyModifiers::SHIFT) => {
                if let Some(task_index) = self.menu_state.selected()
                    && (task_index + 1) < self.task_collection.lists().len()
                {
                    self.task_collection
                        .lists_mut()
                        .swap(task_index, task_index + 1);
                    self.menu_state.select_next();
                }
            }
            (crossterm::event::KeyCode::Char('J'), KeyModifiers::SHIFT)
            | (crossterm::event::KeyCode::Up, KeyModifiers::SHIFT) => {
                if let Some(task_index) = self.menu_state.selected()
                    && task_index > 0
                {
                    self.task_collection
                        .lists_mut()
                        .swap(task_index, task_index - 1);
                    self.menu_state.select_previous();
                }
            }

            (crossterm::event::KeyCode::Char('q'), KeyModifiers::NONE) => self.exit(),
            (crossterm::event::KeyCode::Char('w'), KeyModifiers::NONE) => self.save(),
            _ => {}
        }
    }

    fn key_event_task_body(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            // navigation
            (crossterm::event::KeyCode::Char('h'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Left, KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Esc, KeyModifiers::NONE) => {
                self.current_interface = CurrentInterface::TaskMenu
            }
            (crossterm::event::KeyCode::Char('j'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Down, KeyModifiers::NONE) => {
                self.task_state.select_next()
            }
            (crossterm::event::KeyCode::Char('k'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Up, KeyModifiers::NONE) => {
                self.task_state.select_previous()
            }

            // action
            (crossterm::event::KeyCode::Char('a'), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.enter_write(WriteType::Task, None);
            }
            (crossterm::event::KeyCode::Char('d'), KeyModifiers::NONE) => {
                let Some(task_index) = self.task_state.selected() else {
                    return;
                };
                let Some(task_list) = self
                    .menu_state
                    .selected()
                    .and_then(|index| self.task_collection.get_list_mut(index))
                else {
                    return;
                };
                if task_list.remove_task(task_index) {
                    if task_list.tasks().is_empty() {
                        self.task_state.select(None);
                    } else if task_index >= task_list.tasks().len() {
                        self.task_state.select(Some(task_list.tasks().len() - 1));
                    }
                }
            }
            (crossterm::event::KeyCode::Char(' '), KeyModifiers::NONE)
            | (crossterm::event::KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some((list_index, task_index)) =
                    self.menu_state.selected().zip(self.task_state.selected())
                    && let Some(task) = self
                        .task_collection
                        .get_list_mut(list_index)
                        .and_then(|l| l.get_task_mut(task_index))
                {
                    task.toggle();
                }
            }

            // reorder
            (crossterm::event::KeyCode::Char('K'), KeyModifiers::SHIFT)
            | (crossterm::event::KeyCode::Down, KeyModifiers::SHIFT) => {
                let Some(task_index) = self.task_state.selected() else {
                    return;
                };
                let Some(task_list) = self
                    .menu_state
                    .selected()
                    .and_then(|index| self.task_collection.get_list_mut(index))
                else {
                    return;
                };

                if (task_index + 1) < task_list.tasks().len() {
                    task_list.tasks_mut().swap(task_index, task_index + 1);
                    self.task_state.select_next()
                }
            }
            (crossterm::event::KeyCode::Char('J'), KeyModifiers::SHIFT)
            | (crossterm::event::KeyCode::Up, KeyModifiers::SHIFT) => {
                let Some(task_index) = self.task_state.selected() else {
                    return;
                };
                let Some(task_list) = self
                    .menu_state
                    .selected()
                    .and_then(|index| self.task_collection.get_list_mut(index))
                else {
                    return;
                };

                if task_index > 0 {
                    task_list.tasks_mut().swap(task_index, task_index - 1);
                    self.task_state.select_previous()
                }
            }
            (crossterm::event::KeyCode::Char('A'), KeyModifiers::SHIFT) => {
                let Some((list_index, task_index)) =
                    self.menu_state.selected().zip(self.task_state.selected())
                else {
                    return;
                };
                let desc = self
                    .task_collection
                    .get_list(list_index)
                    .and_then(|l| l.get_task(task_index))
                    .and_then(|t| t.description())
                    .map(String::from);
                self.enter_write(WriteType::TaskDescription, desc.as_deref());
            }
            (crossterm::event::KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                let Some((list_index, task_index)) =
                    self.menu_state.selected().zip(self.task_state.selected())
                else {
                    return;
                };
                let Some(task_name) = self
                    .task_collection
                    .get_list(list_index)
                    .and_then(|l| l.get_task(task_index))
                    .map(|t| t.task().to_string())
                else {
                    return;
                };
                self.enter_write(WriteType::RenameTask, Some(&task_name));
            }
            _ => {}
        }
    }

    fn key_event_help(&mut self, _key_event: KeyEvent) {
        // TODO: help key inputs
        todo!("Implement help key events");
    }

    pub(super) fn key_event_write(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (crossterm::event::KeyCode::Enter, KeyModifiers::NONE) => {
                if self.write_input.final_input().is_empty() {
                    self.current_layout = self.previous_layout;
                    self.current_interface = self.previous_interface;
                    return;
                }
                match self.write_input.write_type {
                    WriteType::Menu => {
                        self.task_collection
                            .add_list(crate::task::TaskList::new(self.write_input.final_input()));
                        // select the newly added menu
                        self.menu_state
                            .select(Some(self.task_collection.lists().len() - 1));
                    }
                    WriteType::Task => {
                        if let Some(task_list) = self
                            .menu_state
                            .selected()
                            .and_then(|index| self.task_collection.get_list_mut(index))
                        {
                            task_list.add_task(crate::task::Task::new(
                                self.write_input.final_input(),
                                None,
                            ));
                        }
                    }
                    WriteType::RenameMenu => {
                        if let Some(task_list) = self
                            .menu_state
                            .selected()
                            .and_then(|index| self.task_collection.get_list_mut(index))
                        {
                            task_list.rename(self.write_input.final_input());
                        }
                    }
                    WriteType::RenameTask => {
                        if let Some((list_index, task_index)) =
                            self.menu_state.selected().zip(self.task_state.selected())
                        {
                            let new_name = self.write_input.final_input().to_owned();
                            if let Some(task) = self
                                .task_collection
                                .get_list_mut(list_index)
                                .and_then(|l| l.get_task_mut(task_index))
                            {
                                task.rename(&new_name);
                            }
                        }
                    }
                    WriteType::TaskDescription => {
                        if let Some((list_index, task_index)) =
                            self.menu_state.selected().zip(self.task_state.selected())
                        {
                            let new_desc = self.write_input.final_input().to_string();
                            let desc = if new_desc.is_empty() {
                                None
                            } else {
                                Some(new_desc.as_str())
                            };
                            if let Some(task) = self
                                .task_collection
                                .get_list_mut(list_index)
                                .and_then(|l| l.get_task_mut(task_index))
                            {
                                task.change_description(desc);
                            }
                        }
                    }
                }

                self.current_layout = self.previous_layout;
                self.current_interface = self.previous_interface;
            }
            (crossterm::event::KeyCode::Char(to_insert), KeyModifiers::NONE | KeyModifiers::SHIFT) =>
            {
                self.write_input.enter_char(to_insert)
            }
            (crossterm::event::KeyCode::Backspace, KeyModifiers::NONE) => self.write_input.delete_char(),
            (crossterm::event::KeyCode::Left, KeyModifiers::NONE) => self.write_input.move_cursor_left(),
            (crossterm::event::KeyCode::Right, KeyModifiers::NONE) => self.write_input.move_cursor_right(),
            (crossterm::event::KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                self.write_input.character_index = 0
            }
            (crossterm::event::KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.write_input.character_index = self.write_input.input.chars().count();
            }
            (crossterm::event::KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                let idx = self.write_input.character_index;
                if idx > 0 {
                    let chars: Vec<char> = self.write_input.input.chars().collect();
                    let mut new_chars = chars[..idx - 1].to_vec();
                    let after = &chars[idx..];
                    let mut skip = 0;
                    for c in after.iter() {
                        if *c == ' ' {
                            skip += 1;
                        } else {
                            break;
                        }
                    }
                    let mut word_skipped = false;
                    for c in after[skip..].iter() {
                        if *c == ' ' && word_skipped {
                            break;
                        }
                        skip += 1;
                        word_skipped = true;
                    }
                    new_chars.extend_from_slice(&after[skip..]);
                    self.write_input.input = new_chars.into_iter().collect();
                    self.write_input.character_index = idx - 1 - skip;
                }
            }
            (crossterm::event::KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.write_input
                    .input
                    .truncate(self.write_input.byte_index());
                self.write_input.character_index = 0;
            }
            (crossterm::event::KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                let idx = self.write_input.character_index;
                let chars: Vec<char> = self.write_input.input.chars().collect();
                self.write_input.input = chars[..idx].iter().collect();
            }
            (crossterm::event::KeyCode::Esc, KeyModifiers::NONE) => {
                self.current_layout = self.previous_layout;
                self.current_interface = self.previous_interface;
            }
            _ => {}
        }
    }

    pub(super) fn exit(&mut self) {
        self.exit = true;
    }

    pub(super) fn save(&mut self) {
        let Some(ref path) = self.save_path else {
            self.error_message = Some("Could not determine data directory".to_string());
            return;
        };
        let Some(path_str) = path.to_str() else {
            self.error_message = Some("Invalid save path".to_string());
            return;
        };
        if let Err(error) = crate::task::save_to_file(&self.task_collection, path_str) {
            self.error_message = Some(format!("Failed to save: {error}"));
        } else {
            self.original_task_collection = self.task_collection.clone();
            self.error_message = None;
        }
    }
}
