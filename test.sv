module Adder(
);

  logic res;
  assign res = 1;
  assign c = a + b;
endmodule

module TestAdder_tb;
  Adder uut();
  initial begin
    #10 $display("Simulation Started");
  end
endmodule

