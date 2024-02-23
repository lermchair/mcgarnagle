// a verilog module that takes 2 2-bit inputs and returns the sum of the inputs

module test3 (input [1:0] a, input [1:0] b, output [2:0] sum);
	assign sum = a + b;
endmodule