# Hadou 

Hadou is a TUI for `iverilog`, providing options to Create projects, edit projects and view waveforms


## Installation

### Dependencies

1. `just` = A command runner similar to `make`
2. `iverilog` = Icarus Verilog
3. `gtkwave` = Waveform viewing tool (Required for now)

### Installing

#### From crates.io

```bash
cargo install hadou
```

#### From source

```bash
git clone https://github.com/Vaishnav-Sabari-Girish/Hadou
cd Hadou/
cargo run --release
```

`crates.io` release expected soon.

## Features 

1. [x] Create New projects
    - [x] Write verilog code
    - [x] Compile it and generate `.vcd` file
2. [x] Edit projects
3. [ ] View waveform from `.vcd` files
