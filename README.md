# neoh
NeoH (or NeoHDL) is a declarative HDL, proficient in creating extremely fast and secure testbenches, with complete support for SystemVerilog, as it transpiles into said HDL.
## Features
• Modular "blocks", improves on "modules" in SystemVerilog:
```SystemVerilog
block Example(in a, in b, out c) logic {
    ret tempassign c [always 7:0] <= a + b;
}
// NOTE tempassign assign a value to a variable for only that block. The [always MSB:LSB] identifier can be assigned to either the result, or expression.
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
    getvars(signal1, signal2, !signal3, clk, rst);
// "!" before a var (in getvars) indicates it should never be included, if, for example, "*" is passed as an argument (which means "all")
    when(BEGIN){
        put signal1 <= 1;
        /10 expect(signal2 == 1);
        pulse len(clk), gap(rst);
        /50 watchfor req <= ack & /100 out(status);
       writefile(mode vcd, file output.vcd); 
    }
}
```
• Along with "testgroups":
```SystemVerilog
testgroup ExampleGroup {
    do tb_random;
    same {
        run tb1;
        run tb2;
    } // "same" = synchronous running;
}
```
• Typing "always_ff" is easier now 🙃:
```SystemVerilog
aff posedge(clk) or negedge(rst) {}
```

# In Conclusion...
• It provides a more declarative way to write testbenches.

• No more boilerplate code with complex hierarchies.

• And more modular syntax.

## Installation
Ensure you have [Rust/Cargo](https://rustup.rs/) installed.
```bash
git clone [https://github.com/yourusername/neoh](https://github.com/yourusername/neoh)
cd neoh
cargo build --release

## Quick Start
Create a file called main.neoh:
```SystemVerilog
block Example(in a, in b, out c) logic {
    out tempassign c <= a + b;
    ret c;
}
```