module adder_64bit(
    input [63:0] a,  // 64-bit input operand a
    input [63:0] b,  // 64-bit input operand b
    output [63:0] sum  // 64-bit output sum
);

// Perform addition without carry-out
assign sum = a + b;

endmodule
