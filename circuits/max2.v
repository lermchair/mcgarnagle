module ManhattanDistanceCalculator(
    input [7:0] x1,
    input [7:0] y1,
    input [7:0] x2,
    input [7:0] y2,
    output [8:0] distance // Output width increased to accommodate max possible value
);

// Intermediate signals to hold absolute differences
wire [7:0] deltaX;
wire [7:0] deltaY;

// Calculating absolute difference for X and Y
assign deltaX = (x1 > x2) ? (x1 - x2) : (x2 - x1);
assign deltaY = (y1 > y2) ? (y1 - y2) : (y2 - y1);

// Summing the absolute differences to get Manhattan distance
assign distance = deltaX + deltaY;

endmodule
