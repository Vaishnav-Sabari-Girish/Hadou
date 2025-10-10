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
use std::path::PathBuf;

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
    pub vcd_files: Vec<PathBuf>,
    pub selected_vcd_index: usize,
    pub input_buffer: String,
    pub message: String,
    pub should_quit: bool
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            mode: AppMode::MainMenu,
            selected_index: 0,
            project_creator: ProjectCreator::new(),
            project_editor: ProjectEditor::new(),
            project_compiler: ProjectCompiler::new(),
            vcd_files: Vec::new(),
            selected_vcd_index: 0,
            input_buffer: String::new(),
            message: String::new(),
            should_quit: false
        };
        app.scan_vcd_files();
        app
    }

    fn scan_vcd_files(&mut self) {
        self.vcd_files.clear();
        self.selected_vcd_index = 0;

        // Scan current directory for VCD files
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("vcd") {
                    self.vcd_files.push(path);
                }
            }
        }

        // Also scan subdirectories
        if let Ok(entries) = std::fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub_entries) = std::fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() && sub_path.extension().and_then(|s| s.to_str()) == Some("vcd") {
                                self.vcd_files.push(sub_path);
                            }
                        }
                    }
                }
            }
        }

        // Sort VCD files alphabetically
        self.vcd_files.sort();
    }

    fn launch_waveform_viewer(&mut self) {
        if self.vcd_files.is_empty() {
            self.message = "No VCD files found. Run a simulation first!".to_string();
            self.mode = AppMode::MessageDialog;
            return;
        }

        let vcd_file = &self.vcd_files[self.selected_vcd_index];

        // Try different waveform viewers in order of preference
        let viewers = [
            ("dwfv", vec![vcd_file.to_string_lossy().to_string()]),
            ("digisurf", vec!["-f".to_string(), vcd_file.to_string_lossy().to_string()]),
            ("gtkwave", vec![vcd_file.to_string_lossy().to_string()]),
        ];

        for (viewer, args) in &viewers {
            match std::process::Command::new(viewer).args(args).spawn() {
                Ok(mut child) => {
                    // Show a message that the viewer is launching
                    self.message = format!("Launching {} with {}\n\nHadou will exit when you close the waveform viewer.", viewer, vcd_file.display());
                    self.mode = AppMode::MessageDialog;

                    // Wait for the external viewer to exit, then quit Hadou
                    std::thread::spawn(move || {
                        let _ = child.wait();
                        std::process::exit(0);
                    });

                    return;
                }
                Err(_) => continue,
            }
        }

        self.message = "No waveform viewers found!\n\nInstall options:\n• cargo install dwfv (recommended)\n• cargo install digisurf\n• sudo apt install gtkwave".to_string();
        self.mode = AppMode::MessageDialog;
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match self.mode {
            AppMode::MainMenu => self.handle_main_menu_key(key),
            AppMode::CreateProject => self.handle_create_project_key(key),
            AppMode::CompileProject => self.handle_compile_project_key(key),
            AppMode::EditProject => self.handle_edit_project_key(key),
            AppMode::ViewWaveform => self.handle_view_waveform_key(key),
            AppMode::InputDialog => self.handle_input_dialog_key(key),
            AppMode::MessageDialog => self.handle_message_dialog_key(key),
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
                        // Refresh VCD files and enter waveform mode
                        self.scan_vcd_files();
                        self.mode = AppMode::ViewWaveform;
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
                            // Refresh VCD files since compilation might have generated new ones
                            self.scan_vcd_files();
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

    fn handle_view_waveform_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => self.mode = AppMode::MainMenu,
            KeyCode::Up => {
                if !self.vcd_files.is_empty() {
                    self.selected_vcd_index = if self.selected_vcd_index == 0 {
                        self.vcd_files.len() - 1
                    } else {
                            self.selected_vcd_index - 1
                        };
                }
            }
            KeyCode::Down => {
                if !self.vcd_files.is_empty() {
                    self.selected_vcd_index = (self.selected_vcd_index + 1) % self.vcd_files.len();
                }
            }
            KeyCode::Enter => {
                self.launch_waveform_viewer();
            }
            KeyCode::Char('r') => {
                // Refresh VCD files
                self.scan_vcd_files();
                self.message = format!("Refreshed VCD files. Found {} files", self.vcd_files.len());
                self.mode = AppMode::MessageDialog;
            }
            KeyCode::Char('i') => {
                // Show install instructions
                self.message = "Waveform Viewer Installation:\n\n• DWFV (recommended): cargo install dwfv\n• DigiSurf: cargo install digisurf\n• GTKWave: sudo apt install gtkwave\n\nDWFV provides the best terminal experience with vi-like keybindings!".to_string();
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
        AppMode::ViewWaveform => render_view_waveform(f, app, chunks[0]),
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

fn render_view_waveform(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = Paragraph::new("📊 View Waveform with External Viewer")
        .style(Style::default().fg(PALETTE.macchiato.colors.mauve.into()).add_modifier(Modifier::BOLD))
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
        Line::from(format!("Found {} VCD file(s):", app.vcd_files.len())),
    ];

    let info = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("VCD Info"));

    // VCD files list
    let vcd_widget = if !app.vcd_files.is_empty() {
        let vcd_items: Vec<ListItem> = app.vcd_files
            .iter()
            .enumerate()
            .map(|(i, vcd_path)| {
                let file_name = vcd_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let parent_dir = vcd_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or(".");

                let style = if i == app.selected_vcd_index {
                    Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black)
                } else {
                    Style::default()
                };

                let display_text = if parent_dir == "." {
                    format!("📄 {}", file_name)
                } else {
                    format!("📄 {}/{}", parent_dir, file_name)
                };

                ListItem::new(display_text).style(style)
            })
            .collect();

        List::new(vcd_items)
            .block(Block::default().title("VCD Files").borders(Borders::ALL))
            .highlight_style(Style::default().bg(PALETTE.macchiato.colors.yellow.into()).fg(Color::Black))
    } else {
        List::new(vec![ListItem::new("No VCD files found. Run a simulation first!")])
            .block(Block::default().title("VCD Files").borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
    };

    // Viewer info
    let viewer_info = vec![
        Line::from("Supported Waveform Viewers:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("📊 DWFV", Style::default().fg(PALETTE.macchiato.colors.green.into()).add_modifier(Modifier::BOLD)),
            Span::raw(" - Vi-like TUI waveform viewer (Recommended)"),
        ]),
        Line::from(vec![
            Span::styled("⚡ DigiSurf", Style::default().fg(PALETTE.macchiato.colors.blue.into()).add_modifier(Modifier::BOLD)),
            Span::raw(" - Modern TUI with command interface"),
        ]),
        Line::from(vec![
            Span::styled("🖥️  GTKWave", Style::default().fg(PALETTE.macchiato.colors.yellow.into()).add_modifier(Modifier::BOLD)),
            Span::raw(" - Traditional GUI waveform viewer"),
        ]),
    ];

    let viewer_widget = Paragraph::new(viewer_info)
        .block(Block::default().borders(Borders::ALL).title("Viewer Options"));

    let help_text = if !app.vcd_files.is_empty() {
        "↑/↓: Select VCD file | Enter: Launch viewer | 'r': Refresh | 'i': Install info | Esc: Return"
    } else {
        "'r': Refresh files | 'i': Install viewer info | Esc: Return to main menu"
    };

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).title("Controls"));

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(6),  // Info
            Constraint::Min(8),     // VCD files
            Constraint::Length(8),  // Viewer options
            Constraint::Length(3),  // Help
        ])
        .split(area);

    f.render_widget(title, layout[0]);
    f.render_widget(info, layout[1]);
    f.render_widget(vcd_widget, layout[2]);
    f.render_widget(viewer_widget, layout[3]);
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
