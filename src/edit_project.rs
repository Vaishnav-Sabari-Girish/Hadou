use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct ProjectEditor {
    pub projects: Vec<PathBuf>,
    pub selected_project_index: usize,
    pub current_directory: PathBuf
}

impl ProjectEditor {
    pub fn new() -> Self {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut editor = Self {
            projects: Vec::new(),
            selected_project_index: 0,
            current_directory: current_dir,
        };

        editor.scan_for_projects();

        editor
    }

    pub fn scan_for_projects(&mut self) {
        self.projects.clear();
        self.selected_project_index = 0;

        if let Ok(entries) = fs::read_dir(&self.current_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && self.is_valid_project(&path) {
                    self.projects.push(path);
                }
            }
        }

        self.projects.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(b.file_name().unwrap_or_default())
        });
    }

    pub fn is_valid_project(&self, dir_path: &Path) -> bool {
        let main_v_path = dir_path.join("main.v");
        main_v_path.exists() && main_v_path.is_file()
    }

    pub fn get_project_files(&self, project_path: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();

        let essential_files = ["main.v", "main_test.v", "Justfile", "justfile"];

        for file_name in &essential_files {
            let file_path = project_path.join(file_name);
            if file_path.exists() {
                files.push(file_path);
            }
        }

        // Add any other .v files in the directory
        if let Ok(entries) = fs::read_dir(project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "v" {
                            let file_name = path.file_name().unwrap().to_string_lossy();
                            if file_name != "main.v" && file_name != "main_test.v" {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }

        files
    }

    pub fn open_project_in_editor(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.projects.is_empty() {
            return Err("No Verilog projects found in current directory".into());
        }

        if self.selected_project_index >= self.projects.len() {
            return Err("Invalid Project Selection".into());
        }

        let project_path = &self.projects[self.selected_project_index];
        let files_to_edit = self.get_project_files(project_path);

        if files_to_edit.is_empty() {
            return Err("No Editable files found".into());
        } 

        self.launch_editor(&files_to_edit, project_path)?;
        Ok(())
    }

    fn launch_editor(&self, files: &[PathBuf], project_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let editor = self.detect_editor()?;

        let mut command = Command::new(&editor);

        command.current_dir(project_dir);   // Change to project directory

        // Add files to command
        for file in files {
            if let Ok(relative_path) = file.strip_prefix(project_dir) {
                command.arg(relative_path);
            } else {
                command.arg(file);
            }
        }

        match editor.to_lowercase().as_str() {
            editor_name if editor_name.contains("code") => {
                // Clear previous args and set new ones for VS Code
                command = Command::new(&editor);
                command.current_dir(project_dir);
                command.args(&[".", "--goto", "main.v:1:1"]);
            }
            editor_name if editor_name.contains("nvim") || editor_name.contains("vim") => {
                command.arg("-p"); // Open in tabs
            }
            editor_name if editor_name.contains("emacs") => {
                command.arg("--no-wait");
            }
            editor_name if editor_name.contains("codium") => {
                // Clear previous args and set new ones for VSCodium
                command = Command::new(&editor);
                command.current_dir(project_dir);
                command.args(&[".", "--goto", "main.v:1:1"]);
            }
            editor_name if editor_name.contains("edit") => {
                // For editors that can only edit one file at a time
                command = Command::new(&editor);
                command.current_dir(project_dir);
                command.arg(files.iter().find(|f| f.file_name().unwrap() == "main.v")
                    .unwrap_or(&files[0]));
            }
            _ => {
                // Default: keep all files as arguments
            }
        }

        let status = command.status()?;

        if !status.success() {
            return Err(format!("Editor {} exited with error code: {}", editor, status.code().unwrap_or(-1)).into());
        }

        Ok(())
    }

    fn detect_editor(&self) -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(editor) = env::var("EDITOR") {
             if !editor.is_empty() {
                return  Ok(editor);
            }
         } 

        if cfg!(target_os = "windows") {
            let windows_editors = [
                "code",
                "codium",
                "notepad++",
                "notepad",
            ];

            for editor in &windows_editors {
                if self.command_exists(editor) {
                    return  Ok(editor.to_string());
                }
            }

            return  Ok("notepad".to_string());   // Default
        } else {
            let unix_editors = [
                "nvim",
                "vim",
                "emacs",
                "code",
                "codium",
                "nano",
                "gedit",
                "kate"
            ];

            for editor in &unix_editors {
                if self.command_exists(editor) {
                    return Ok(editor.to_string());
                }
            }

            return Err("No suitable editor found. Please set the EDITOR environment variable".into());
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

    pub fn move_selection_up(&mut self) {
        if !self.projects.is_empty() {
            self.selected_project_index = if self.selected_project_index == 0 {
                self.projects.len() - 1
            } else {
                    self.selected_project_index - 1
            };
        }
    }

    // Fixed typo: move_sleection_down -> move_selection_down
    pub fn move_selection_down(&mut self) {
        if !self.projects.is_empty() {
            self.selected_project_index = (self.selected_project_index + 1) % self.projects.len();
        }
    }

    pub fn refresh_projects(&mut self) {
        self.scan_for_projects();
    }

    // Fixed method name: selected_project_name -> get_selected_project_name
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

    // Fixed method name: get_selected_project_files -> get_selected_project_path
    pub fn get_selected_project_path(&self) -> Option<&PathBuf> {
        if self.selected_project_index < self.projects.len() {
            Some(&self.projects[self.selected_project_index])
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
}

impl Default for ProjectEditor {
    fn default() -> Self {
        Self::new()
    }
}
