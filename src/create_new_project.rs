use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct ProjectCreator {
    pub project_name: String,
}

impl ProjectCreator {
    pub fn new() -> Self {
        Self { 
            project_name: String::new() 
        }
    }

    pub fn reset(&mut self) {
        self.project_name.clear();
    }

    pub fn create_project(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        if self.project_name.is_empty() {
            return Err("Project name cannot be empty".into());
        }

        if !self.is_valid_project_name(&self.project_name) {
            return Err("Invalid Project name. Use only alphanumeric characters, underscores and hyphens".into());
        }

        let project_path = PathBuf::from(&self.project_name);

        if project_path.exists() {
            return Err(format!("Directory {} already exists", self.project_name).into());
        }

        fs::create_dir_all(&project_path)?;

        // Create main.v file
        let main_v_path = project_path.join("main.v");
        let main_v_content = self.generate_main_v_content();
        fs::write(&main_v_path, main_v_content)?;

        // Create main_test.v file
        let main_test_v_path = project_path.join("main_test.v");
        let main_test_v_content = self.generate_testbench_content();
        fs::write(&main_test_v_path, main_test_v_content)?;

        // Create a simple Justfile for easy compilation
        let justfilee_path = project_path.join("Justfile");
        let justfile_contents = self.generate_justfile();
        fs::write(&justfilee_path, justfile_contents)?;

        Ok(project_path.canonicalize()?)
    }

    fn is_valid_project_name(&self, name: &str) -> bool {
        !name.is_empty()
        && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        && !name.starts_with('-')
        && !name.starts_with('_')
    }

    fn generate_main_v_content(&self) -> String {
        format!(
r#"`timescale 1ns / 1ps

//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: {}
// Design Name: {}
// Module Name: {}
// Project Name: {}
// Target Devices: 
// Tool Versions: 
// Description: 
// 
// Dependencies: 
// 
// Revision:
// Revision 0.01 - File Created
// Additional Comments:
// 
//////////////////////////////////////////////////////////////////////////////////

module {} (
    input wire clk,
    input wire reset,
    output reg [7:0] data_out
);

    // Internal registers and wires
    reg [7:0] counter;
    
    // Main logic
    always @(posedge clk or posedge reset) begin
        if (reset) begin
            counter <= 8'b0;
            data_out <= 8'b0;
        end else begin
            counter <= counter + 1;
            data_out <= counter;
        end
    end

endmodule
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            self.project_name,
            self.project_name,
            self.project_name,
            self.project_name,
        )
    }

    fn generate_testbench_content(&self) -> String {
        format!(
r#"`timescale 1ns / 1ps

//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: {}
// Design Name: {}_testbench
// Module Name: {}_test
// Project Name: {}
// Target Devices: 
// Tool Versions: 
// Description: Testbench for {}
// 
// Dependencies: 
// 
// Revision:
// Revision 0.01 - File Created
// Additional Comments:
// 
//////////////////////////////////////////////////////////////////////////////////

module {}_test;

    // Inputs
    reg clk;
    reg reset;
    
    // Outputs
    wire [7:0] data_out;
    
    // Instantiate the Unit Under Test (UUT)
    {} uut (
        .clk(clk),
        .reset(reset),
        .data_out(data_out)
    );
    
    // Clock generation
    always #5 clk = ~clk; // 100MHz clock (10ns period)
    
    initial begin
        // Initialize inputs
        clk = 0;
        reset = 0;
        
        // Add stimulus here
        $display("Starting simulation...");
        
        // Apply reset
        reset = 1;
        #20;
        reset = 0;
        
        // Let it run for some cycles
        #200;
        
        $display("Simulation completed at time %t", $time);
        $finish;
    end
    
    // Monitor changes
    initial begin
        $monitor("Time=%t, Reset=%b, Data_out=%d", $time, reset, data_out);
    end
    
    // Generate VCD file for waveform viewing
    initial begin
        $dumpfile("{}.vcd");
        $dumpvars(0, {}_test);
    end

endmodule
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            self.project_name,
            self.project_name,
            self.project_name,
            self.project_name,
            self.project_name,
            self.project_name,
            self.project_name,
            self.project_name,
        )
    }

    fn generate_justfile(&self) -> String {
        format!(
r#"# justfile for {} Verilog project
# Generated by Hadou

# Project configuration
PROJECT_NAME := "{}"
SRC_FILE := "main.v"
TEST_FILE := "main_test.v"
VVP_FILE := PROJECT_NAME + ".vvp"
VCD_FILE := PROJECT_NAME + ".vcd"

# Default recipe - compile and simulate
default: compile simulate

# Compile the design and testbench
compile:
    @echo "Compiling Verilog files..."
    iverilog -o {{{{VVP_FILE}}}} {{{{SRC_FILE}}}} {{{{TEST_FILE}}}}
    @echo "Compilation completed: {{{{VVP_FILE}}}}"

# Run the simulation
simulate: compile
    @echo "Running simulation..."
    vvp {{{{VVP_FILE}}}}
    @echo "Simulation completed. VCD file: {{{{VCD_FILE}}}}"

# View waveform (requires GTKWave)
view: simulate
    @echo "Opening waveform viewer..."
    gtkwave {{{{VCD_FILE}}}} &

# Clean generated files
clean:
    @echo "Cleaning generated files..."
    -rm {{{{VVP_FILE}}}} {{{{VCD_FILE}}}}
    @echo "Clean completed."

# Show project info
info:
    @echo "Project: {{{{PROJECT_NAME}}}}"
    @echo "Source file: {{{{SRC_FILE}}}}"
    @echo "Test file: {{{{TEST_FILE}}}}"
    @echo "Output files: {{{{VVP_FILE}}}}, {{{{VCD_FILE}}}}"

# List all available recipes
list:
    @just --list

# Help - show available commands
help:
    @echo "Available commands:"
    @echo "  just           - Compile and simulate (default)"
    @echo "  just compile   - Compile Verilog files"
    @echo "  just simulate  - Run simulation (generates VCD)"
    @echo "  just view      - Open GTKWave to view waveform"
    @echo "  just clean     - Remove generated files"
    @echo "  just info      - Show project information"
    @echo "  just list      - List all available recipes"
    @echo "  just help      - Show this help message"
"#,
            self.project_name,
            self.project_name,
        )
    }
}

impl Default for ProjectCreator {
    fn default() -> Self {
        Self::new()
    }
}
