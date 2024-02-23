# McGarnagle

> [!WARNING]  
> This is a proof of concept and should not be taken seriously.

## What is this?

- A simple way to optimize [garbled circuits](https://en.wikipedia.org/wiki/Garbled_circuit) with [e-graphs](https://en.wikipedia.org/wiki/E-graph) and equality saturation. It uses the [egg](https://egraphs-good.github.io/) crate to handle e-graphs.
- Garbled circuits have an optimization you can do that allows XOR gates to be free to compute. This rewrite boolean circuits to maximize the number of XOR gates.

## Does it actually work?

Kind of. Here is the result from running `cargo bench`, which compares an optimized and unoptimized 64-bit adder circuit. Bigger circuits will probably show more improvement.

![Benchmark](./benchmark.png?raw=true "Benchmark")

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
- [ ] Parse multi-file verilog modules (e.g. AES)
- [ ] Add parsers for more circuit formats, like Bristol Fashion
- [ ] Write better rules
