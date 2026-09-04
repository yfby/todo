use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Position, Rect, Size},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Clear, List, ListItem, Paragraph, StatefulWidget, Widget, Wrap},
};

use super::{App, CurrentInterface, CurrentLayout, WriteType};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.current_layout {
            CurrentLayout::Task => self.render_task_layout(area, buf),
            CurrentLayout::Help => todo!("Implement help layout rendering"),
            CurrentLayout::Exit => {} // TODO: confirm exit
        }
    }
}

impl App {
    fn render_task_layout(&mut self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::horizontal([Constraint::Length(20), Constraint::Fill(1)]).split(area);

        let task_menu_area = chunks[0];
        let task_body_area = chunks[1];

        self.render_task_menu(task_menu_area, buf);
        self.render_task_body(task_body_area, buf);

        self.write_widget(area, buf);
        self.render_error_message(area, buf);

        if self.task_collection != self.original_task_collection {
            let unsaved_area = Rect {
                x: area.x + 1,
                y: area.y + area.height.saturating_sub(1),
                width: area.width,
                height: 1,
            };
            Paragraph::new(" Unsaved Changes* ")
                .style(Style::new().add_modifier(Modifier::BOLD))
                .render(unsaved_area, buf);
        }
    }

    fn render_task_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let items: Vec<_> = self.task_collection.get_list_names();

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
            let mut list_items: Vec<ListItem> = vec![];
            for item in tasks {
                if item.is_completed() {
                    list_items.push(
                        ListItem::new(
                            format!("\u{2611} {}", item.task()).add_modifier(Modifier::CROSSED_OUT),
                        )
                        .gray(),
                    );
                } else {
                    list_items.push(
                        ListItem::new(format!("\u{2610} {}", item.task()).bold().white()),
                    );
                }
            }

            let list = List::new(list_items).block(block).highlight_style(
                Style::default()
                    .fg(Color::LightYellow)
                    .add_modifier(Modifier::BOLD),
            );
            StatefulWidget::render(list, area, buf, &mut self.task_state);

            // description
            let selected_idx = self.task_state.selected().unwrap_or(0) as u16;

            if let Some((list_index, task_index)) =
                self.menu_state.selected().zip(self.task_state.selected())
                && let Some(task) = self
                    .task_collection
                    .get_list(list_index)
                    .and_then(|l| l.get_task(task_index))
                && task.description().is_some()
            {
                let mut description_area = area.resize(Size::new(50, 10));
                description_area.x += 1;
                description_area.y = description_area.y + 2 + selected_idx;

                Clear.render(description_area, buf);
                let description = task.description().unwrap_or("").to_string();

                if self.current_interface == CurrentInterface::Write
                    && self.write_input.write_type == WriteType::TaskDescription
                {
                    self.write_widget(description_area, buf);
                } else {
                    self.description_widget(&description, description_area, buf);
                }
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

    fn description_widget(&mut self, description: &str, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .border_style(Style::new().light_green())
            .title("Description")
            .title_alignment(Alignment::Center);

        Paragraph::new(description)
            .wrap(Wrap { trim: false })
            .block(block)
            .render(area, buf);
    }

    pub(super) fn write_widget(&mut self, area: Rect, buf: &mut Buffer) {
        let area = area.centered(Constraint::Length(30), Constraint::Length(3));
        if self.current_interface == CurrentInterface::Write {
            Clear.render(area, buf);

            let mut write_block = Block::bordered().border_style(Style::new().light_blue());

            match self.write_input.write_type {
                WriteType::Menu => {
                    write_block = write_block
                        .title("New Menu")
                        .title_alignment(Alignment::Center);
                }
                WriteType::Task => {
                    write_block = write_block
                        .title("New Task")
                        .title_alignment(Alignment::Center);
                }
                WriteType::RenameMenu => {
                    write_block = write_block
                        .title("New Menu Name")
                        .title_alignment(Alignment::Center);
                }
                WriteType::RenameTask => {
                    write_block = write_block
                        .title("New Task Name")
                        .title_alignment(Alignment::Center);
                }
                WriteType::TaskDescription => {
                    write_block = write_block
                        .title("Description")
                        .title_alignment(Alignment::Center);
                }
            }

            Paragraph::new(self.write_input.final_input())
                .wrap(Wrap { trim: false })
                .block(write_block)
                .render(area, buf);

            self.cursor_position = Some(Position::new(
                area.x + self.write_input.character_index as u16 + 1,
                area.y + 1,
            ));
        } else {
            self.cursor_position = None;
        }
    }

    fn render_error_message(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(ref msg) = self.error_message {
            let error_area = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1),
                width: area.width.min(msg.len() as u16 + 4),
                height: 1,
            };
            Paragraph::new(msg.as_str())
                .style(Style::new().fg(Color::Red).add_modifier(Modifier::BOLD))
                .render(error_area, buf);
        }
    }

    pub(super) fn enter_write(&mut self, write_type: WriteType, set_input: Option<&str>) {
        self.previous_layout = self.current_layout;
        self.previous_interface = self.current_interface;
        self.write_input.reset_cursor();
        self.write_input.input = set_input.unwrap_or_default().to_string();
        self.write_input.write_type = write_type;
        self.current_interface = CurrentInterface::Write;
    }
}
