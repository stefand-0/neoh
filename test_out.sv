`timescale 1ns / 1ps

module EmptyLogic();
endmodule
module tb_TestOut();
  EmptyLogic dut();
  initial begin
    #100 $display("Hello, NeoH!");
  end
endmodule
module tg_MyGroup();
endmodule
