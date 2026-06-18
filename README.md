# neoh
NeoH (or NeoHDL) is a declarative HDL, proficient in creating extremely fast and secure testbenches, with complete support for SystemVerilog, as it transpiles into said HDL.
## Features
• Modular "blocks", improves on "modules" in SystemVerilog:
```SystemVerilog
block Example(in a, in b, out c) logic {
    ret tempassign c [always 7:0] <= a + b;
}
```
• Improves on "interfaces":
```SystemVerilog
piece ExampleBus {
    in addr, in data, out ready
}
pieced block Example(ExampleBus <= bus){...}
```
• Improved hierarchy support:
```SystemVerilog
block top(in x, in y, out z) logic {
    n1 passparams example(x,y,z);
    ret n1;
}
```
• Easier macros with "known":
```SystemVerilog
known macro1 <= clk, rst;
// pass into block
block macroeater(macro1) {//} 
```
• Testbenches are easier now:
```SystemVerilog
testbench random(RandomBlock){
    getvars(signal1, signal2, !signal3);
// "!" before a var (in getvars) indicates it should never be included, if, for example, "*" is passed as an argument (which means "all")
    when(BEGIN){
        put signal1 <= 1;
        /10 expect(signal2 == 1);
        pulse clk(10), rst(2);
        /50 watchfor req <= ack & /100 out(status);
       writefile(mode vcd, file output.vcd); 
}
}
```