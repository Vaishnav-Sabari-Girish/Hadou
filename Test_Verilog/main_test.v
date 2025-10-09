//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: 2025-10-09 17:32:23 UTC
// Design Name: Test_Verilog_testbench
// Module Name: Test_Verilog_test
// Project Name: Test_Verilog
// Target Devices: 
// Tool Versions: 
// Description: Testbench for Test_Verilog
// 
// Dependencies: 
// 
// Revision:
// Revision 0.01 - File Created
// Additional Comments:
// 
//////////////////////////////////////////////////////////////////////////////////

module Test_Verilog_test;

    reg a, b;
    wire c;
    // Instantiate the Unit Under Test (UUT)
    Test_Verilog uut (
      a, 
      b, 
      c
    );
    
    initial begin
        // Add stimulus here
        $display("Starting simulation...");
    
    // Generate VCD file for waveform viewing
        $dumpfile("Test_Verilog.vcd");
        $dumpvars(0, Test_Verilog_test);

        a = 0; b = 0; #100;
        a = 0; b = 1; #100;
        a = 1; b = 0; #100;
        a = 1; b = 1; #100;
    end

endmodule
