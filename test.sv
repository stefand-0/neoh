`timescale 1ns / 1ps

module my_logic();
  assign  x = a + b;
  assign  z = a - b;
  reg y;
endmodule
module tb_my_tb();
  my_logic dut();
  initial begin
    x 10 10;
  end
endmodule
