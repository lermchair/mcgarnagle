# McGarnagle

An optimizer for Yao's garbled circuits.

> [!WARNING]  
> This is a proof of concept (for learning) and should not be taken seriously.

## What is this?

[Garbled circuits](https://en.wikipedia.org/wiki/Garbled_circuit) have an optimization that lets XOR gates be computed for free. We can rewrite boolean circuits to maximize the number of XOR gates in the circuit, with [e-graphs](https://en.wikipedia.org/wiki/E-graph) and equality saturation. This specific proof of concept uses the [egg](https://egraphs-good.github.io/) crate to handle e-graphs.

## Benchmarks

All benchmarks are run with `cargo bench` on an M2 Macbook Pro.

### 64-bit adder circuit generated by Yosys

| Avg. Time (ms) | Unoptimized | Optimized |
| -------------- | ----------- | --------- |
| Lower          | 5.5505      | 4.9284    |
| Estimate       | 5.5709      | 4.9427    |
| Upper          | 5.5926      | 4.9571    |

### 64-bit adder circuit in Bristol format (already optimized)

| Avg. Time (ms) | Unoptimized | Optimized |
| -------------- | ----------- | --------- |
| Lower          | 2.1816      | 2.0040    |
| Estimate       | 2.1935      | 2.0064    |
| Upper          | 2.2153      | 2.0092    |

For large circuits (e.g. SHA-256), the actual optimization time will be much slower than simply running the circuit unoptimized. However, we technically only need to do the optimization once, so the cost can be spread out over many uses of the circuit.

## How is this better than normal logic synthesis tools?

- If you have a boolean circuit, it's possible to automatically rewrite it to maximize the number of XOR gates it uses. One way to do this is by applying rules that match patterns in the circuit and replace them with equivalent patterns that use more XOR gates.
- Most optimizers, like the one in [Yosys](https://github.com/YosysHQ/yosys), apply rewrite rules in ordered passes. However the order in which you apply these rules is important, because applying one rule might cut off the possibility of applying another rule that would have been better.
- We use a technique called equality saturation to search for all possible orders of applying the rewrite rules, and pick the best one.
- The cost function we use to select the "optimal" circuit is the AST size, weighted by the gate operation (i.e. XORs cost 1, ANDs cost 4, etc). You can see the cost function in `src/optimizer/mod.rs`
- The actual garbled circuit implementation is not important and is interchangeable (as long as it implements the Free XOR optimization).

### What does the code do?

1. Parse a Verilog file or Bristol formatted boolean circuit.
2. Convert the boolean circuit into an e-graph.
3. Saturate the e-graph and extract the best circuit.
4. Demonstrate a basic Garbled Circuit computation with the optimized circuit.

TODO:

- [ ] Make slightly more configurable
