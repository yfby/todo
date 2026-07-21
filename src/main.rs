use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Position, Rect, Size},
    style::{Color, Modifier, Style},
    widgets::{Block, Clear, List, ListState, Paragraph, StatefulWidget, Widget},
};
use std::io;

mod task;

const SAVE_FILE: &str = "tasks.json";

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))
}

struct App {
    exit: bool,
    current_layout: CurrentLayout,
    current_interface: CurrentInterface,
    previous_layout: CurrentLayout,
    previous_interface: CurrentInterface,
    task_collection: task::TaskListCollection,
    menu_state: ListState,
    task_state: ListState,
    write_input: WriteInterface,
    cursor_position: Option<Position>,
}

#[derive(Clone, Copy)]
enum CurrentLayout {
    Task,
    Exit,
}

#[derive(PartialEq, Clone, Copy)]
enum CurrentInterface {
    TaskMenu,
    TaskBody,
    Write,
    Exit, // TODO: confirm exit
}

struct WriteInterface {
    input: String,
    character_index: usize,
    write_type: WriteType,
}

enum WriteType {
    Menu,
    Task,
    TaskDescription,
}

impl WriteInterface {
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn final_input(&self) -> &str {
        &self.input
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            exit: false,
            current_layout: CurrentLayout::Task,
            current_interface: CurrentInterface::TaskMenu,
            previous_layout: CurrentLayout::Task,
            previous_interface: CurrentInterface::TaskMenu,
            task_collection: task::load_or_default(SAVE_FILE),
            menu_state: ListState::default().with_selected(Some(0)),
            task_state: ListState::default().with_selected(None),
            write_input: WriteInterface {
                input: String::new(),
                character_index: 0,
                write_type: WriteType::Menu,
            },
            cursor_position: None,
        }
    }
}

/// Root Logic
impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                frame.render_widget(&mut self, frame.area());

                // set cursor position for writing
                if let Some(pos) = self.cursor_position {
                    frame.set_cursor_position(pos);
                }
            })?;

            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn selected_task(&mut self) -> Option<&mut task::Task> {
        let index = self.menu_state.selected()?;
        let list = self.task_collection.get_list(index)?;
        let task_index = self.task_state.selected()?;
        list.get_task(task_index)
    }
}

/// Event Logic
impl App {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.current_interface {
            CurrentInterface::TaskMenu => self.key_event_task_menu(key_event),
            CurrentInterface::TaskBody => self.key_event_task_body(key_event),
            CurrentInterface::Write => self.key_event_write(key_event),
            CurrentInterface::Exit => todo!(),
        }
    }

    fn key_event_task_menu(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => self.menu_state.select(None),
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
                self.menu_state.select_next();
                self.task_state.select(None);
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
                self.menu_state.select_previous();
                self.task_state.select(None);
            }
            (KeyCode::Char('l'), KeyModifiers::NONE)
            | (KeyCode::Right, KeyModifiers::NONE)
            | (KeyCode::Enter, KeyModifiers::NONE) => {
                self.current_interface = CurrentInterface::TaskBody;
                if self.task_state.selected().is_none() {
                    self.task_state.select(Some(0));
                }
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.enter_write(WriteType::Menu, None);
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) | (KeyCode::Delete, KeyModifiers::NONE) => {
                if let Some(index) = self.menu_state.selected() {
                    self.task_collection.remove_list(index);
                    self.task_state.select(None);
                    if self.task_collection.lists().is_empty() {
                        self.menu_state.select(None);
                    } else if index >= self.task_collection.lists().len() {
                        self.menu_state
                            .select(Some(self.task_collection.lists().len() - 1));
                    }
                }
            }

            // TODO: make universal
            (KeyCode::Char('q'), KeyModifiers::NONE) => self.exit(),
            (KeyCode::Char('w'), KeyModifiers::NONE) => self.save(),
            _ => {}
        }
    }

    fn key_event_task_body(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Char('h'), KeyModifiers::NONE)
            | (KeyCode::Left, KeyModifiers::NONE)
            | (KeyCode::Esc, KeyModifiers::NONE) => {
                self.current_interface = CurrentInterface::TaskMenu
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
                self.task_state.select_next()
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
                self.task_state.select_previous()
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) | (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.enter_write(WriteType::Task, None);
            }
            (KeyCode::Char('A'), KeyModifiers::SHIFT)
            | (KeyCode::Char('I'), KeyModifiers::SHIFT) => {
                if let Some(task_description) = self.selected_task()
                    && let Some(desc) = task_description.description()
                {
                    let desc_string = desc.to_string();
                    self.enter_write(WriteType::TaskDescription, Some(&desc_string));
                } else {
                    self.enter_write(WriteType::TaskDescription, None);
                }
            }
            (KeyCode::Char('d'), KeyModifiers::NONE) => {
                let Some(task_index) = self.task_state.selected() else {
                    return;
                };
                let Some(task_list) = self
                    .menu_state
                    .selected()
                    .and_then(|index| self.task_collection.get_list(index))
                else {
                    return;
                };
                task_list.remove_task(task_index);
                if task_list.tasks().is_empty() {
                    self.task_state.select(None);
                } else if task_index >= task_list.tasks().len() {
                    self.task_state.select(Some(task_list.tasks().len() - 1));
                }
            }
            (KeyCode::Char(' '), KeyModifiers::NONE) | (KeyCode::Enter, KeyModifiers::NONE) => {
                if let Some(task) = self.selected_task() {
                    task.toggle();
                }
            }
            _ => {}
        }
    }

    fn key_event_write(&mut self, key_event: KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Enter, KeyModifiers::NONE) => {
                match self.write_input.write_type {
                    WriteType::Menu => self
                        .task_collection
                        .add_list(task::TaskList::new(self.write_input.final_input())),
                    WriteType::Task => {
                        let Some(task_list) = self
                            .menu_state
                            .selected()
                            .and_then(|index| self.task_collection.get_list(index))
                        else {
                            return;
                        };

                        task_list.add_task(task::Task::new(self.write_input.final_input(), &None));
                    }
                    WriteType::TaskDescription => {
                        let new_desc = self.write_input.final_input().to_string();
                        if new_desc.is_empty() {
                            self.selected_task().unwrap().change_description(&None);
                        } else {
                            self.selected_task()
                                .unwrap()
                                .change_description(&Some(new_desc));
                        }
                    }
                }

                self.current_layout = self.previous_layout;
                self.current_interface = self.previous_interface;
            }
            (KeyCode::Char(to_insert), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.write_input.enter_char(to_insert)
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => self.write_input.delete_char(),
            (KeyCode::Left, KeyModifiers::NONE) => self.write_input.move_cursor_left(),
            (KeyCode::Right, KeyModifiers::NONE) => self.write_input.move_cursor_right(),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => self.write_input.character_index = 0,
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                self.write_input.character_index = self.write_input.input.chars().count();
            }
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => {
                let idx = self.write_input.character_index;
                if idx > 0 {
                    let chars: Vec<char> = self.write_input.input.chars().collect();
                    let mut new_chars = chars[..idx - 1].to_vec();
                    let after = &chars[idx..];
                    // skip trailing whitespace, then skip one word
                    let mut skip = 0;
                    for c in after.iter() {
                        if *c == ' ' {
                            skip += 1;
                        } else {
                            break;
                        }
                    }
                    // skip the word after spaces
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
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                self.write_input
                    .input
                    .truncate(self.write_input.byte_index());
                self.write_input.character_index = 0;
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                let idx = self.write_input.character_index;
                let chars: Vec<char> = self.write_input.input.chars().collect();
                self.write_input.input = chars[..idx].iter().collect();
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.current_layout = self.previous_layout;
                self.current_interface = self.previous_interface;
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn save(&self) {
        if let Err(error) = task::save_to_file(&self.task_collection, SAVE_FILE) {
            eprintln!("Problem opening the file: {:?}", error);
        }
    }
}

/// Rendering Logic
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.current_layout {
            CurrentLayout::Task => self.render_task_layout(area, buf),
            CurrentLayout::Exit => todo!(),
        }
    }
}

impl App {
    fn render_task_layout(&mut self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).split(area);

        let task_menu_area = chunks[0];
        let task_body_area = chunks[1];
        // if self.current_interface == CurrentInterface::Write {
        // }
        // Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(chunks[1]);

        self.render_task_menu(task_menu_area, buf);
        self.render_task_body(task_body_area, buf);

        self.write_widget(area, buf);
    }

    fn render_task_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<_> = self.task_collection.get_lists_name();

        let mut block = Block::bordered()
            .title("Task Menu")
            .title_alignment(Alignment::Left);

        if self.current_interface == CurrentInterface::TaskMenu {
            block = block.border_style(Style::new().light_blue());
        }

        if items.is_empty() {
            Paragraph::new("No Tasks Found")
                .block(block)
                .centered()
                .render(area, buf);
        } else {
            let list = List::new(items).block(block).highlight_style(
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            );

            StatefulWidget::render(list, area, buf, &mut self.menu_state);
        }
    }

    fn render_task_body(&mut self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::bordered()
            .title("Tasks")
            .title_alignment(Alignment::Center);

        if self.current_interface == CurrentInterface::TaskBody {
            block = block.border_style(Style::new().light_blue());
        }

        if let Some(tasks) = self
            .menu_state
            .selected()
            .and_then(|index| self.task_collection.get_list(index))
            .map(|list| list.tasks())
            .filter(|items| !items.is_empty())
        {
            let items = tasks.iter().map(|t| {
                if t.is_completed() {
                    format!("✓ {}", t.task())
                } else {
                    format!("☐ {}", t.task())
                }
            });
            let list = List::new(items).block(block).highlight_style(
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            );
            StatefulWidget::render(list, area, buf, &mut self.task_state);

            let selected_idx = self.task_state.selected().unwrap_or(0) as u16;

            if let Some(task_description) = self.selected_task()
                && task_description.description().is_some()
            {
                let description_block = Block::bordered()
                    .border_style(Style::new().light_green())
                    .title("Description")
                    .title_alignment(Alignment::Center);
                let mut descripton_area = area.resize(Size::new(50, 10));

                descripton_area.x += 1;
                descripton_area.y = descripton_area.y + 2 + selected_idx;

                Clear.render(descripton_area, buf);
                Paragraph::new(task_description.description().unwrap_or(""))
                    .block(description_block)
                    .render(descripton_area, buf);
            }
        } else {
            let msg_area = area.centered(Constraint::Length(40), Constraint::Length(1));
            let msg = if self.menu_state.selected().is_none() {
                "No Task Selected"
            } else {
                "No Tasks Available"
            };

            if self.menu_state.selected().is_some() {
                block.render(area, buf);
            }

            Paragraph::new(msg).centered().render(msg_area, buf);
        }
    }

    fn write_widget(&mut self, area: Rect, buf: &mut Buffer) {
        let write_area = area.centered(Constraint::Length(30), Constraint::Length(3));

        if self.current_interface == CurrentInterface::Write {
            Clear.render(write_area, buf);

            let mut write_block = Block::bordered();

            if self.current_interface == CurrentInterface::Write {
                write_block = write_block.border_style(Style::new().light_blue());
            }

            match self.write_input.write_type {
                WriteType::Menu => {
                    write_block = write_block.title("Menu").title_alignment(Alignment::Center);
                }
                WriteType::Task => {
                    write_block = write_block.title("Task").title_alignment(Alignment::Center);
                }
                WriteType::TaskDescription => {
                    write_block = write_block
                        .title("Description")
                        .title_alignment(Alignment::Center);
                }
            }

            Paragraph::new(self.write_input.final_input())
                .block(write_block)
                .render(write_area, buf);

            // cursor positioning
            self.cursor_position = Some(Position::new(
                write_area.x + self.write_input.character_index as u16 + 1,
                write_area.y + 1,
            ));
        } else {
            self.cursor_position = None;
        }
    }
    fn enter_write(&mut self, write_type: WriteType, set_input: Option<&String>) {
        self.previous_layout = self.current_layout;
        self.previous_interface = self.current_interface;
        self.write_input.reset_cursor();
        if let Some(input) = set_input {
            self.write_input.input = input.clone();
        } else {
            self.write_input.input = String::new();
        }
        self.write_input.write_type = write_type;

        self.current_interface = CurrentInterface::Write;
    }
}
