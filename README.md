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
