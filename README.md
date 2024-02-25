# McGarnagle

> [!WARNING]  
> This is a proof of concept and should not be taken seriously.

## What is this?

[Garbled circuits](https://en.wikipedia.org/wiki/Garbled_circuit) have an optimization that lets XOR gates be computed for free. We can rewrite boolean circuits to maximize the number of XOR gates in the circuit, with [e-graphs](https://en.wikipedia.org/wiki/E-graph) and equality saturation. This specific proof of concept uses the [egg](https://egraphs-good.github.io/) crate to handle e-graphs.

## Does it actually work?

Kind of. Here is the result from running `cargo bench`, which compares an optimized and unoptimized 64-bit adder circuit.

![Benchmark](./benchmark.png?raw=true "Benchmark")

For large circuits (e.g. SHA-256), the actual optimization time will be much slower than simply running the circuit unoptimized. However, we technically only need to do the optimization once, so the cost can be spread out over many uses of the circuit.

## How is this better than normal logic synthesis tools?

- If you have a boolean circuit, it's possible to automatically rewrite it to maximize the number of XOR gates it uses. One way to do this is by applying rules that match patterns in the circuit and replace them with equivalent patterns that use more XOR gates.
- Most optimizers, like the one in [Yosys](https://github.com/YosysHQ/yosys), apply rewrite rules in ordered passes. However the order in which you apply these rules is important, because applying one rule might cut off the possibility of applying another rule that would have been better.
- We use a technique called equality saturation to search for all possible orders of applying the rewrite rules, and pick the best one.
- The cost function we use to select the "optimal" circuit is the AST size, weighted by the gate operation (i.e. XORs cost 0, ANDs cost 4, etc).
- The actual garbled circuit implementation is not important and is interchangeable (as long as it implements the Free XOR optimization).

### What does the code do?

1. Parse a verilog file into a boolean circuit via Yosys (and apply its optimizations).
2. Convert the boolean circuit into an e-graph.
3. Saturate the e-graph and extract the best circuit.
4. Demonstrate a basic Garbled Circuit computation with the optimized circuit.

TODO:

- [ ] Actually implement oblivious transfer
- [ ] Parse multi-file verilog modules
- [ ] Write better rules
