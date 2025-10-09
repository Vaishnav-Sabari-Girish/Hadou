use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileAction {
    CompileOnly,
    CompileAndSimulate,
    CompileSimulateAndView,
    Clean,
    Info,
}

impl CompileAction {
    pub fn as_just_recipe(&self) -> &'static str {
        match self {
            CompileAction::CompileOnly => "compile",
            CompileAction::CompileAndSimulate => "simulate", // simulate depends on compile
            CompileAction::CompileSimulateAndView => "view", // view depends on simulate
            CompileAction::Clean => "clean",
            CompileAction::Info => "info",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CompileAction::CompileOnly => "Compile Verilog files only",
            CompileAction::CompileAndSimulate => "Compile and run simulation",
            CompileAction::CompileSimulateAndView => "Compile, simulate, and open waveform",
            CompileAction::Clean => "Clean generated files",
            CompileAction::Info => "Show project information",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CompileAction::CompileOnly => "⚙️ ",
            CompileAction::CompileAndSimulate => "🚀",
            CompileAction::CompileSimulateAndView => "📊",
            CompileAction::Clean => "🧹",
            CompileAction::Info => "ℹ️ ",
        }
    }
}

#[derive(Debug)]
pub struct ProjectCompiler {
    pub projects: Vec<PathBuf>,
    pub selected_project_index: usize,
    pub selected_action_index: usize,
    pub current_directory: PathBuf,
    pub available_actions: Vec<CompileAction>,
    pub compilation_output: Vec<String>,
    pub is_compiling: bool,
}

impl ProjectCompiler {
    pub fn new() -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut compiler = Self {
            projects: Vec::new(),
            selected_project_index: 0,
            selected_action_index: 0,
            current_directory: current_dir,
            available_actions: vec![
                CompileAction::CompileOnly,
                CompileAction::CompileAndSimulate,
                CompileAction::CompileSimulateAndView,
                CompileAction::Clean,
                CompileAction::Info,
            ],
            compilation_output: Vec::new(),
            is_compiling: false,
        };

        compiler.scan_for_projects();
        compiler
    }

    pub fn scan_for_projects(&mut self) {
        self.projects.clear();
        self.selected_project_index = 0;

        if let Ok(entries) = fs::read_dir(&self.current_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && self.has_verilog_files(&path) {
                    self.projects.push(path);
                }
            }
        }

        // Sort projects alphabetically
        self.projects.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(b.file_name().unwrap_or_default())
        });
    }

    pub fn has_verilog_files(&self, dir_path: &Path) -> bool {
        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "v" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn has_justfile(&self, dir_path: &Path) -> bool {
        let justfile_path = dir_path.join("justfile");
        let justfile_alt_path = dir_path.join("Justfile");
        justfile_path.exists() || justfile_alt_path.exists()
    }

    pub fn get_verilog_files(&self, project_path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "v" {
                            files.push(path);
                        }
                    }
                }
            }
        }

        // Sort files alphabetically
        files.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(b.file_name().unwrap_or_default())
        });

        files
    }

    pub fn execute_compilation(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.projects.is_empty() {
            return Err("No Verilog projects found in current directory".into());
        }

        if self.selected_project_index >= self.projects.len() {
            return Err("Invalid project selection".into());
        }

        if self.selected_action_index >= self.available_actions.len() {
            return Err("Invalid action selection".into());
        }

        // Clone the values we need to avoid borrowing conflicts
        let project_path = self.projects[self.selected_project_index].clone();
        let action = self.available_actions[self.selected_action_index].clone();

        // Check if justfile exists
        if !self.has_justfile(&project_path) {
            return Err("No justfile found in project directory. Please create the project using Hadou first.".into());
        }

        self.is_compiling = true;
        self.compilation_output.clear();

        let result = self.run_just_command(&project_path, &action);
        
        self.is_compiling = false;
        result
    }

    fn run_just_command(&mut self, project_dir: &Path, action: &CompileAction) -> Result<String, Box<dyn std::error::Error>> {
        // Check if just command exists
        if !self.command_exists("just") {
            return Err("'just' command not found. Please install 'just' command runner.".into());
        }

        let mut command = Command::new("just");
        command.current_dir(project_dir);
        command.arg(action.as_just_recipe());

        // Capture both stdout and stderr
        let output = command.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Store output for display
        if !stdout.is_empty() {
            self.compilation_output.extend(stdout.lines().map(String::from));
        }
        if !stderr.is_empty() {
            self.compilation_output.extend(stderr.lines().map(String::from));
        }

        if output.status.success() {
            let project_name = project_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            Ok(format!(
                "{} completed successfully for project '{}'",
                action.description(),
                project_name
            ))
        } else {
            Err(format!(
                "{} failed with exit code: {}\nOutput: {}{}",
                action.description(),
                output.status.code().unwrap_or(-1),
                stdout,
                if !stderr.is_empty() { format!("\nErrors: {}", stderr) } else { String::new() }
            ).into())
        }
    }

    fn command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or_else(|_| {
                if cfg!(target_os = "windows") {
                    Command::new("where")
                        .arg(command)
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false)
                } else {
                    false
                }
            })
    }

    pub fn move_project_selection_up(&mut self) {
        if !self.projects.is_empty() {
            self.selected_project_index = if self.selected_project_index == 0 {
                self.projects.len() - 1
            } else {
                self.selected_project_index - 1
            };
        }
    }

    pub fn move_project_selection_down(&mut self) {
        if !self.projects.is_empty() {
            self.selected_project_index = (self.selected_project_index + 1) % self.projects.len();
        }
    }

    pub fn move_action_selection_up(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_action_index = if self.selected_action_index == 0 {
                self.available_actions.len() - 1
            } else {
                self.selected_action_index - 1
            };
        }
    }

    pub fn move_action_selection_down(&mut self) {
        if !self.available_actions.is_empty() {
            self.selected_action_index = (self.selected_action_index + 1) % self.available_actions.len();
        }
    }

    pub fn refresh_projects(&mut self) {
        self.scan_for_projects();
    }

    pub fn get_selected_project_name(&self) -> Option<String> {
        if self.selected_project_index < self.projects.len() {
            self.projects[self.selected_project_index]
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn get_selected_project_path(&self) -> Option<&PathBuf> {
        if self.selected_project_index < self.projects.len() {
            Some(&self.projects[self.selected_project_index])
        } else {
            None
        }
    }

    pub fn get_selected_action(&self) -> Option<&CompileAction> {
        if self.selected_action_index < self.available_actions.len() {
            Some(&self.available_actions[self.selected_action_index])
        } else {
            None
        }
    }

    pub fn has_projects(&self) -> bool {
        !self.projects.is_empty()
    }

    pub fn project_count(&self) -> usize {
        self.projects.len()
    }

    pub fn get_compilation_output(&self) -> &[String] {
        &self.compilation_output
    }

    pub fn clear_compilation_output(&mut self) {
        self.compilation_output.clear();
    }
}

impl Default for ProjectCompiler {
    fn default() -> Self {
        Self::new()
    }
}
