use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal
};

use catppuccin::PALETTE;

use std::{default, io};
use std::path::PathBuf;

mod create_new_project;

use create_new_project::ProjectCreator;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    MainMenu,
    CreateProject,
    CompileProject,
    EditProject,
    ViewWaveform,
    InputDialog,
    MessageDialog
}

#[derive(Debug)]
pub struct  App {
    pub mode: AppMode,
    pub selected_index: usize,
    pub project_creator: ProjectCreator,
    pub input_buffer: String,
    pub message: String,
    pub should_quit: bool
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::MainMenu,
            selected_index: 0,
            project_creator: ProjectCreator::new(),
            input_buffer: String::new(),
            message: String::new(),
            should_quit: false
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::MainMenu => self.handle_main_menu_key(key),
            AppMode::CreateProject => self.handle_create_project_key(key),
            AppMode::InputDialog => self.handle_input_dialog_key(key),
            AppMode::MessageDialog => self.handle_message_dialog_key(key),
            _ => {
                if key == KeyCode::Esc {
                    self.mode = AppMode::MainMenu;
                }
            }
        }
    }

    fn handle_main_menu_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Down => {
                self.selected_index = (self.selected_index + 1) % 4;
            },
            KeyCode::Up => {
                self.selected_index = if self.selected_index == 0 {
                    3
                } else {
                        self.selected_index - 1
                };
            },
            KeyCode::Enter => {
                match self.selected_index {
                    0 => self.mode = AppMode::CreateProject,
                    1 => {
                        self.message = "Compile project feature on da wae".to_string();
                        self.mode = AppMode::MessageDialog;
                    }
                    2 => {
                        self.message = "Edit project feature on da wae".to_string();
                        self.mode = AppMode::MessageDialog;
                    }
                    3 => {
                        self.message = "View  Waveform feature on da wae".to_string();
                        self.mode = AppMode::MessageDialog;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_create_project_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.mode = AppMode::MainMenu,
            KeyCode::Enter => {
                if !self.project_creator.project_name.is_empty() {
                    match self.project_creator.create_project() {
                        Ok(path) => {
                            self.message = format!("Project Created successfully at : {}", path.display());
                            self.project_creator.reset();
                            self.mode = AppMode::MessageDialog;
                        }
                        Err(e) => {
                            self.message = format!("Error creating project : {:?}", e);
                            self.mode = AppMode::MessageDialog;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                self.project_creator.project_name.pop();
            }
            KeyCode::Char(c) => {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    self.project_creator.project_name.push(c);
                }
            }
            _ => {}
        }
    }

    fn handle_message_dialog_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter | KeyCode::Esc => {
                self.message.clear();
                self.mode = AppMode::MainMenu;
            }
            _ => {}
        }
    }
}


fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(f.area());

    match app.mode {
        AppMode::MainMenu => render_main_menu(f, app, chunks[0]),
        AppMode::CreateProject => render_create_project(f, app, chunks[0]),
        AppMode::MessageDialog => {
            render_main_menu(f, app, chunks[0]);
            render_message_dialog(f, app);
        }
        _ => render_main_menu(f, app, chunks[0]),
    }
}

fn render_main_menu<B: Backend>(f: &mut Frame<B>, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("🌊 Hadou - Verilog Project Manager")
        .style(Style::default().fg(PALETTE.macchiato.colors.teal.into()).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    let menu_items = vec![
        "📁 Create New Project",
        "⚙️  Compile Project", 
        "✏️  Edit Project",
        "📊 View Waveform",
    ];

    let items: Vec<ListItem> = menu_items
    .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_index {
                Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black)
            } else {
                Style::default()
            };
            ListItem::new(*item).style(style)
        })
            .collect();

    let menu = List::new(items)
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .highlight_style(Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black));

    let help = Paragraph::new("Use ↑/↓ to navigate, Enter to select, 'q' or Esc to quit")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
            .split(area);

    f.render_widget(title, layout[0]);
    f.render_widget(menu, layout[1]);
    f.render_widget(help, layout[2]);
}

fn render_create_project<B: Backend>(f: &mut Frame<B>, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("📁 Create New Verilog Project")
        .style(Style::default().fg(PALETTE.macchiato.colors.green.into()).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
}

fn main() {
    println!("Hello, world!");
}
