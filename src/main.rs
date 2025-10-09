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
use compile_project::ProjectCompiler;

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
    pub project_compiler: ProjectCompiler,
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
            project_compiler: ProjectCompiler::new(),
            input_buffer: String::new(),
            message: String::new(),
            should_quit: false
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::MainMenu => self.handle_main_menu_key(key),
            AppMode::CreateProject => self.handle_create_project_key(key),
            AppMode::CompileProject => self.handle_compile_project_key(key),
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
                        // Refresh project list when entering edit mode
                        self.project_editor.refresh_projects();
                        self.mode = AppMode::EditProject;
                    }
                    2 => {
                        // Refresh project list when entering compile mode
                        self.project_compiler.refresh_projects();
                        self.mode = AppMode::CompileProject;
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
                            // Refresh both editor and compiler lists since we created a new project
                            self.project_editor.refresh_projects();
                            self.project_compiler.refresh_projects();
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

    fn handle_compile_project_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.mode = AppMode::MainMenu,
            KeyCode::Up => {
                self.project_compiler.move_project_selection_up();
            }
            KeyCode::Down => {
                self.project_compiler.move_project_selection_down();
            }
            KeyCode::Left => {
                self.project_compiler.move_action_selection_up();
            }
            KeyCode::Right => {
                self.project_compiler.move_action_selection_down();
            }
            KeyCode::Enter => {
                if self.project_compiler.has_projects() && !self.project_compiler.is_compiling {
                    match self.project_compiler.execute_compilation() {
                        Ok(success_msg) => {
                            self.message = success_msg;
                            self.mode = AppMode::MessageDialog;
                        }
                        Err(e) => {
                            self.message = format!("Compilation failed: {}", e);
                            self.mode = AppMode::MessageDialog;
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                // Refresh project list
                self.project_compiler.refresh_projects();
                self.message = format!("Refreshed project list. Found {} projects", 
                    self.project_compiler.project_count());
                self.mode = AppMode::MessageDialog;
            }
            KeyCode::Char('c') => {
                // Clear compilation output
                self.project_compiler.clear_compilation_output();
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
        AppMode::CompileProject => render_compile_project(f, app, chunks[0]),
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
        "✏️  Edit Project",
        "⚙️  Compile Project", 
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

fn render_compile_project(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("⚙️  Compile Verilog Project")
        .style(Style::default().fg(PALETTE.macchiato.colors.red.into()).add_modifier(Modifier::BOLD))
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
        Line::from(format!("Found {} Verilog project(s):", app.project_compiler.project_count())),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Project Info"));

    // Create horizontal layout for projects and actions
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(main_layout[0]);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(6),
        ])
        .split(main_layout[1]);

    // Projects list
    let projects_widget = if app.project_compiler.has_projects() {
        let project_items: Vec<ListItem> = app.project_compiler.projects
            .iter()
            .enumerate()
            .map(|(i, project_path)| {
                let project_name = project_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let style = if i == app.project_compiler.selected_project_index {
                    Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black)
                } else {
                    Style::default()
                };

                // Show project name with verilog file count and justfile status
                let verilog_files = app.project_compiler.get_verilog_files(project_path);
                let has_justfile = app.project_compiler.has_justfile(project_path);
                let justfile_indicator = if has_justfile { "⚡" } else { "❌" };
                
                let display_text = format!("📁 {} ({} .v files) {}", 
                    project_name, verilog_files.len(), justfile_indicator);
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

    // Actions list
    let action_items: Vec<ListItem> = app.project_compiler.available_actions
        .iter()
        .enumerate()
        .map(|(i, action)| {
            let style = if i == app.project_compiler.selected_action_index {
                Style::default().bg(PALETTE.macchiato.colors.blue.into()).fg(Color::White)
            } else {
                Style::default()
            };

            let display_text = format!("{} {}", action.icon(), action.description());
            ListItem::new(display_text).style(style)
        })
        .collect();

    let actions_widget = List::new(action_items)
        .block(Block::default().title("Actions").borders(Borders::ALL))
        .highlight_style(Style::default().bg(PALETTE.macchiato.colors.blue.into()).fg(Color::White));

    // Preview of selected project
    let preview_text = if let Some(selected_path) = app.project_compiler.get_selected_project_path() {
        let verilog_files = app.project_compiler.get_verilog_files(selected_path);
        let has_justfile = app.project_compiler.has_justfile(selected_path);
        
        if !verilog_files.is_empty() {
            let mut preview = format!("Selected Project:\n📁 {}\n", 
                selected_path.file_name().unwrap().to_string_lossy());
            
            preview.push_str(&format!("\nJustfile: {}\n", if has_justfile { "✅ Found" } else { "❌ Missing" }));
            
            preview.push_str("\nVerilog files:\n");
            for file in verilog_files.iter().take(6) {
                if let Some(file_name) = file.file_name() {
                    preview.push_str(&format!(" 📄 {}\n", file_name.to_string_lossy()));
                }
            }
            if verilog_files.len() > 6 {
                preview.push_str(&format!(" ... and {} more files", verilog_files.len() - 6));
            }
            
            if let Some(action) = app.project_compiler.get_selected_action() {
                preview.push_str(&format!("\nWill execute: just {}", action.as_just_recipe()));
            }
            
            preview
        } else {
            "No Verilog files found in selected project".to_string()
        }
    } else {
        "Select a project to see preview".to_string()
    };

    let preview = Paragraph::new(preview_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Preview"));

    let help_text = if app.project_compiler.has_projects() {
        "↑/↓ select project, ←/→ select action, Enter to execute, 'r' refresh, 'c' clear output, Esc to return"
    } else {
        "No projects found. Press 'r' to refresh, Esc to return to main menu"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Help"));

    // Render left side (title, info, projects, help)
    f.render_widget(title, left_layout[0]);
    f.render_widget(info, left_layout[1]);
    f.render_widget(projects_widget, left_layout[2]);
    f.render_widget(help, left_layout[3]);

    // Render right side (actions, preview)
    f.render_widget(actions_widget, right_layout[1]);
    f.render_widget(preview, right_layout[2]);
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
