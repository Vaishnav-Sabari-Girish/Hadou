//////////////////////////////////////////////////////////////////////////////////
// Company: 
// Engineer: 
// 
// Create Date: 2025-10-10 04:39:29 UTC
// Design Name: NOT_GATE_testbench
// Module Name: NOT_GATE_test
// Project Name: NOT_GATE
// Target Devices: 
// Tool Versions: 
// Description: Testbench for NOT_GATE
// 
// Dependencies: 
// 
// Revision:
// Revision 0.01 - File Created
// Additional Comments:
// 
//////////////////////////////////////////////////////////////////////////////////

module NOT_GATE_test;
    
    reg a, b;
    wire c;

    NOT_GATE uut (
      a, b, c
    );
    initial begin
        $display("Starting simulation...");
        $dumpfile("NOT_GATE.vcd");
        $dumpvars(0, NOT_GATE_test);
        a = 0; b = 0; #10;
        a = 0;b = 1; #10;
        a = 1; b = 0; #10;
        a = 1; b = 1; #10;


    end

endmodule
