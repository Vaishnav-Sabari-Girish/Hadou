use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    pub name: String,
    pub identifier: String,
    pub width: usize,
    pub values: Vec<(u64, String)>, // (timestamp, value)
    pub chart_data: Vec<(f64, f64)>, // (time, numeric_value) for chart rendering
}

#[derive(Debug, Clone)]
pub struct VcdData {
    pub timescale: String,
    pub signals: Vec<Signal>,
    pub max_time: u64,
}

#[derive(Debug)]
pub struct WaveformViewer {
    pub vcd_files: Vec<PathBuf>,
    pub selected_file_index: usize,
    pub current_vcd: Option<VcdData>,
    pub selected_signal_index: usize,
    pub time_offset: u64,
    pub time_scale: f64,
    pub current_directory: PathBuf,
    pub visible_time_window: u64, // How many time units to show
}

impl WaveformViewer {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut viewer = Self {
            vcd_files: Vec::new(),
            selected_file_index: 0,
            current_vcd: None,
            selected_signal_index: 0,
            time_offset: 0,
            time_scale: 1.0,
            current_directory: current_dir,
            visible_time_window: 100,
        };
        
        viewer.scan_for_vcd_files();
        viewer
    }

    pub fn scan_for_vcd_files(&mut self) {
        self.vcd_files.clear();
        self.selected_file_index = 0;

        if let Ok(entries) = fs::read_dir(&self.current_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if extension == "vcd" {
                            self.vcd_files.push(path);
                        }
                    }
                }
            }
        }

        // Also check subdirectories for VCD files
        if let Ok(entries) = fs::read_dir(&self.current_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file() {
                                if let Some(extension) = sub_path.extension() {
                                    if extension == "vcd" {
                                        self.vcd_files.push(sub_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.vcd_files.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .cmp(b.file_name().unwrap_or_default())
        });
    }

    pub fn load_vcd_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.vcd_files.is_empty() {
            return Err("No VCD files found".into());
        }

        if self.selected_file_index >= self.vcd_files.len() {
            return Err("Invalid file selection".into());
        }

        let vcd_path = &self.vcd_files[self.selected_file_index];
        let vcd_data = self.parse_vcd_file(vcd_path)?;
        
        self.current_vcd = Some(vcd_data);
        self.selected_signal_index = 0;
        self.time_offset = 0;
        
        // Set initial visible window based on max time
        if let Some(vcd) = &self.current_vcd {
            self.visible_time_window = (vcd.max_time / 10).max(100);
        }
        
        Ok(())
    }

    fn parse_vcd_file(&self, path: &Path) -> Result<VcdData, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut timescale = String::from("1ns");
        let mut signals = Vec::new();
        let mut signal_map: HashMap<String, usize> = HashMap::new();
        let mut current_time = 0u64;
        let mut max_time = 0u64;
        let mut in_definitions = true;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("$timescale") {
                if let Some(next_line) = content.lines().skip_while(|l| !l.contains("$timescale")).nth(1) {
                    timescale = next_line.trim().to_string();
                }
            }

            if line.starts_with("$var") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    let width = parts[2].parse::<usize>().unwrap_or(1);
                    let identifier = parts[3].to_string();
                    let name = parts[4..].join(" ").trim_end_matches(" $end").to_string();

                    let signal = Signal {
                        name: name.clone(),
                        identifier: identifier.clone(),
                        width,
                        values: Vec::new(),
                        chart_data: Vec::new(),
                    };

                    signal_map.insert(identifier, signals.len());
                    signals.push(signal);
                }
            }

            if line.starts_with("$enddefinitions") {
                in_definitions = false;
            }

            if !in_definitions && !line.is_empty() && !line.starts_with("$") {
                if line.starts_with('#') {
                    if let Ok(time) = line[1..].parse::<u64>() {
                        current_time = time;
                        if time > max_time {
                            max_time = time;
                        }
                    }
                } else {
                    let (value, identifier) = if line.starts_with('b') {
                        let parts: Vec<&str> = line[1..].split_whitespace().collect();
                        if parts.len() >= 2 {
                            (parts[0].to_string(), parts[1].to_string())
                        } else {
                            continue;
                        }
                    } else if line.len() >= 2 {
                        (line[0..1].to_string(), line[1..].to_string())
                    } else {
                        continue;
                    };

                    if let Some(&signal_idx) = signal_map.get(&identifier) {
                        signals[signal_idx].values.push((current_time, value.clone()));
                    }
                }
            }
        }

        // Generate chart data for each signal
        for signal in &mut signals {
            self.generate_chart_data(signal, max_time);
        }

        Ok(VcdData {
            timescale,
            signals,
            max_time,
        })
    }

    fn generate_chart_data(&self, signal: &mut Signal, max_time: u64) {
        signal.chart_data.clear();
        
        if signal.values.is_empty() {
            return;
        }

        let mut current_value = 0.0;
        let mut value_index = 0;
        
        // Sample the signal at regular intervals
        let sample_interval = (max_time as f64 / 1000.0).max(1.0) as u64; // Sample at most 1000 points
        
        for time in (0..=max_time).step_by(sample_interval as usize) {
            // Find the current value at this time
            while value_index < signal.values.len() && signal.values[value_index].0 <= time {
                current_value = self.value_to_numeric(&signal.values[value_index].1, signal.width);
                value_index += 1;
            }
            
            // For multi-bit signals, normalize to 0-1 range based on signal width
            let normalized_value = if signal.width > 1 {
                current_value / ((1u64 << signal.width.min(32)) as f64 - 1.0)
            } else {
                current_value
            };
            
            signal.chart_data.push((time as f64, normalized_value));
        }
        
        // Reset value_index for next signal
    }

    fn value_to_numeric(&self, value: &str, width: usize) -> f64 {
        match value {
            "0" => 0.0,
            "1" => 1.0,
            "x" | "X" => 0.5, // Unknown state - middle value
            "z" | "Z" => 0.25, // High-Z state - quarter value
            _ => {
                // Multi-bit value - try to parse as binary or decimal
                if value.chars().all(|c| c == '0' || c == '1') {
                    // Binary string
                    u64::from_str_radix(value, 2).unwrap_or(0) as f64
                } else {
                    // Try decimal
                    value.parse::<u64>().unwrap_or(0) as f64
                }
            }
        }
    }

    pub fn get_visible_signals(&self) -> Vec<&Signal> {
        if let Some(vcd) = &self.current_vcd {
            // Return signals around the selected one for better visibility
            let start_idx = self.selected_signal_index.saturating_sub(2);
            let end_idx = (self.selected_signal_index + 3).min(vcd.signals.len());
            vcd.signals[start_idx..end_idx].iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_chart_bounds(&self) -> (f64, f64, f64, f64) {
        // x_min, x_max, y_min, y_max
        let x_min = self.time_offset as f64;
        let x_max = (self.time_offset + self.visible_time_window) as f64;
        let y_min = -0.5;
        let y_max = 1.5;
        
        (x_min, x_max, y_min, y_max)
    }

    pub fn move_file_selection_up(&mut self) {
        if !self.vcd_files.is_empty() {
            self.selected_file_index = if self.selected_file_index == 0 {
                self.vcd_files.len() - 1
            } else {
                self.selected_file_index - 1
            };
        }
    }

    pub fn move_file_selection_down(&mut self) {
        if !self.vcd_files.is_empty() {
            self.selected_file_index = (self.selected_file_index + 1) % self.vcd_files.len();
        }
    }

    pub fn move_signal_selection_up(&mut self) {
        if let Some(vcd) = &self.current_vcd {
            if !vcd.signals.is_empty() {
                self.selected_signal_index = if self.selected_signal_index == 0 {
                    vcd.signals.len() - 1
                } else {
                    self.selected_signal_index - 1
                };
            }
        }
    }

    pub fn move_signal_selection_down(&mut self) {
        if let Some(vcd) = &self.current_vcd {
            if !vcd.signals.is_empty() {
                self.selected_signal_index = (self.selected_signal_index + 1) % vcd.signals.len();
            }
        }
    }

    pub fn zoom_in(&mut self) {
        self.visible_time_window = (self.visible_time_window as f64 * 0.7) as u64;
        if self.visible_time_window < 10 {
            self.visible_time_window = 10;
        }
    }

    pub fn zoom_out(&mut self) {
        if let Some(vcd) = &self.current_vcd {
            self.visible_time_window = (self.visible_time_window as f64 * 1.4) as u64;
            if self.visible_time_window > vcd.max_time {
                self.visible_time_window = vcd.max_time;
            }
        }
    }

    pub fn scroll_left(&mut self) {
        let scroll_amount = self.visible_time_window / 10;
        if self.time_offset > scroll_amount {
            self.time_offset -= scroll_amount;
        } else {
            self.time_offset = 0;
        }
    }

    pub fn scroll_right(&mut self) {
        if let Some(vcd) = &self.current_vcd {
            let scroll_amount = self.visible_time_window / 10;
            self.time_offset += scroll_amount;
            if self.time_offset + self.visible_time_window > vcd.max_time {
                self.time_offset = vcd.max_time.saturating_sub(self.visible_time_window);
            }
        }
    }

    pub fn get_signal_value_at_time(&self, signal: &Signal, time: u64) -> String {
        let mut current_value = String::from("x");
        
        for (t, v) in &signal.values {
            if *t <= time {
                current_value = v.clone();
            } else {
                break;
            }
        }
        
        current_value
    }

    pub fn has_vcd_files(&self) -> bool {
        !self.vcd_files.is_empty()
    }

    pub fn vcd_file_count(&self) -> usize {
        self.vcd_files.len()
    }

    pub fn get_selected_file_name(&self) -> Option<String> {
        if self.selected_file_index < self.vcd_files.len() {
            self.vcd_files[self.selected_file_index]
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    pub fn get_selected_signal(&self) -> Option<&Signal> {
        if let Some(vcd) = &self.current_vcd {
            vcd.signals.get(self.selected_signal_index)
        } else {
            None
        }
    }

    pub fn refresh_vcd_files(&mut self) {
        self.scan_for_vcd_files();
    }
}

impl Default for WaveformViewer {
    fn default() -> Self {
        Self::new()
    }
}
