use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal
};

use catppuccin::PALETTE;

use std::io;

mod create_new_project;
mod edit_project;
mod compile_project;

use create_new_project::ProjectCreator;
use edit_project::ProjectEditor;

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
pub struct App {
    pub mode: AppMode,
    pub selected_index: usize,
    pub project_creator: ProjectCreator,
    pub project_editor: ProjectEditor,
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
            project_editor: ProjectEditor::new(),
            input_buffer: String::new(),
            message: String::new(),
            should_quit: false
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::MainMenu => self.handle_main_menu_key(key),
            AppMode::CreateProject => self.handle_create_project_key(key),
            AppMode::EditProject => self.handle_edit_project_key(key),
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
                        // Refresh project list when entering edit mode
                        self.project_editor.refresh_projects();
                        self.mode = AppMode::EditProject;
                    }
                    3 => {
                        self.message = "View Waveform feature on da wae".to_string();
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
                            self.message = format!("Project Created successfully at: {}", path.display());
                            self.project_creator.reset();
                            // Refresh project editor list since we created a new project
                            self.project_editor.refresh_projects();
                            self.mode = AppMode::MessageDialog;
                        }
                        Err(e) => {
                            self.message = format!("Error creating project: {:?}", e);
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

    fn handle_edit_project_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.mode = AppMode::MainMenu,
            KeyCode::Up => {
                self.project_editor.move_selection_up();
            }
            KeyCode::Down => {
                self.project_editor.move_selection_down();
            }
            KeyCode::Enter => {
                if self.project_editor.has_projects() {
                    match self.project_editor.open_project_in_editor() {
                        Ok(()) => {
                            if let Some(project_name) = self.project_editor.get_selected_project_name() {
                                self.message = format!("Opened project '{}' in editor", project_name);
                            } else {
                                self.message = "Project opened in editor".to_string();
                            }
                            self.mode = AppMode::MessageDialog;
                        }
                        Err(e) => {
                            self.message = format!("Error opening project in editor: {}", e);
                            self.mode = AppMode::MessageDialog;
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh project list
                self.project_editor.refresh_projects();
                self.message = format!("Refreshed project list. Found {} projects", 
                    self.project_editor.project_count());
                self.mode = AppMode::MessageDialog;
            }
            _ => {}
        }
    }

    fn handle_input_dialog_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_buffer.clear();
                self.mode = AppMode::MainMenu;
            }
            KeyCode::Enter => {
                // Handle input submission
                self.input_buffer.clear();
                self.mode = AppMode::MainMenu;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
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

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)].as_ref())
        .split(f.area());

    match app.mode {
        AppMode::MainMenu => render_main_menu(f, app, chunks[0]),
        AppMode::CreateProject => render_create_project(f, app, chunks[0]),
        AppMode::EditProject => render_edit_project(f, app, chunks[0]),
        AppMode::MessageDialog => {
            render_main_menu(f, app, chunks[0]);
            render_message_dialog(f, app);
        }
        _ => render_main_menu(f, app, chunks[0]),
    }
}

fn render_main_menu(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
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

fn render_create_project(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("📁 Create New Verilog Project")
        .style(Style::default().fg(PALETTE.macchiato.colors.green.into()).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    let current_dir = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let info_text = vec![
        Line::from(vec![
            Span::raw("Current Directory: "),
            Span::styled(current_dir, Style::default().fg(PALETTE.macchiato.colors.yellow.into())),
        ]),
        Line::from(""),
        Line::from("Enter Project name (alphanumeric, _ and - allowed):"),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Project Info"));

    let input = Paragraph::new(app.project_creator.project_name.as_str())
        .style(Style::default().fg(PALETTE.macchiato.colors.yellow.into()))
        .block(Block::default().borders(Borders::ALL).title("Project Name"));

    let preview_text = if app.project_creator.project_name.is_empty() {
        "Enter a Project Name to see preview".to_string()
    } else {
        format!(
            "Will Create:\n📁 {}/\n 📄 main.v (main module)\n 🧪 main_test.v (testbench)\n ⚡ justfile (build automation)",
            app.project_creator.project_name
        )
    };

    let preview = Paragraph::new(preview_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Preview"));

    let help = Paragraph::new("Enter to create a new project, Esc to return to main menu")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(3),
        ])
        .split(area);

    f.render_widget(title, layout[0]);
    f.render_widget(info, layout[1]);
    f.render_widget(input, layout[2]);
    f.render_widget(preview, layout[3]);
    f.render_widget(help, layout[4]);
}

fn render_edit_project(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("✏️  Edit Verilog Project")
        .style(Style::default().fg(PALETTE.macchiato.colors.blue.into()).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));

    let current_dir = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let info_text = vec![
        Line::from(vec![
            Span::raw("Current Directory: "),
            Span::styled(current_dir, Style::default().fg(PALETTE.macchiato.colors.yellow.into())),
        ]),
        Line::from(""),
        Line::from(format!("Found {} Verilog project(s):", app.project_editor.project_count())),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Project Info"));

    // Project list or empty message
    let projects_widget = if app.project_editor.has_projects() {
        let project_items: Vec<ListItem> = app.project_editor.projects
            .iter()
            .enumerate()
            .map(|(i, project_path)| {
                let project_name = project_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let style = if i == app.project_editor.selected_project_index {
                    Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black)
                } else {
                    Style::default()
                };

                // Show project name with file count
                let files = app.project_editor.get_project_files(project_path);
                let display_text = format!("📁 {} ({} files)", project_name, files.len());
                ListItem::new(display_text).style(style)
            })
            .collect();

        List::new(project_items)
            .block(Block::default().title("Projects").borders(Borders::ALL))
            .highlight_style(Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black))
    } else {
        List::new(vec![ListItem::new("No Verilog projects found in current directory")])
            .block(Block::default().title("Projects").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
    };

    // Preview of selected project files
    let preview_text = if let Some(selected_path) = app.project_editor.get_selected_project_path() {
        let files = app.project_editor.get_project_files(selected_path);
        if !files.is_empty() {
            let mut preview = format!("Will open in editor:\n📁 {}\n", 
                selected_path.file_name().unwrap().to_string_lossy());
            
            for file in files.iter().take(8) { // Show max 8 files to avoid overflow
                if let Some(file_name) = file.file_name() {
                    let icon = match file.extension().and_then(|ext| ext.to_str()) {
                        Some("v") => "📄",
                        Some(_) => "📄",
                        None => "⚡", // justfile has no extension
                    };
                    preview.push_str(&format!(" {} {}\n", icon, file_name.to_string_lossy()));
                }
            }
            if files.len() > 8 {
                preview.push_str(&format!(" ... and {} more files", files.len() - 8));
            }
            preview
        } else {
            "No editable files found in selected project".to_string()
        }
    } else {
        "Select a project to see preview".to_string()
    };

    let preview = Paragraph::new(preview_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Preview"));

    let help_text = if app.project_editor.has_projects() {
        "Use ↑/↓ to navigate, Enter to edit project, 'r' to refresh, Esc to return to main menu"
    } else {
        "No projects found. Press 'r' to refresh, Esc to return to main menu"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(6),
            Constraint::Length(5),
            Constraint::Length(3),
        ])
        .split(area);

    f.render_widget(title, layout[0]);
    f.render_widget(info, layout[1]);
    f.render_widget(projects_widget, layout[2]);
    f.render_widget(preview, layout[3]);
    f.render_widget(help, layout[4]);
}

fn render_message_dialog(f: &mut Frame, app: &App) {
    let area = f.area();
    let popup_area = ratatui::layout::Rect {
        x: area.width / 4,
        y: area.height / 3,
        width: area.width / 2,
        height: area.height / 3,
    };

    f.render_widget(Clear, popup_area);

    let message = Paragraph::new(app.message.as_str())
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title("Message")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(message, popup_area);

    let help_area = ratatui::layout::Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height - 1,
        width: popup_area.width,
        height: 1,
    };

    let help = Paragraph::new("Press Enter or Esc to continue")
        .style(Style::default().fg(Color::Gray))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(help, help_area);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.on_key(key.code);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
