module multiplier(
    input [7:0] a, // 8-bit input operand a
    input [7:0] b, // 8-bit input operand b
    output [7:0] result // 16-bit output result
);

// Perform multiplication
assign result = a * b;

endmodule
