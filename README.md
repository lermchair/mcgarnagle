# McGarnagle

- A fun way to optimize garbled circuits with e-graphs.
- It works with any boolean circuit, but we care most about garbled circuits because we rewrite them to maximize the number of XOR gates and take advantage of the Free XOR optimization.
- The actual garbled circuit implementation is not important and can be switched out, as long as it can be represented as an e-graph.
- Approach:
  1.  We have a bunch of rewrite rules, and we explore every possible alternative order of applying the rewrite rules (i.e. equality saturation).
  2.  Then, from the saturated e-graph, we pick the circuit with the lowest "cost" (e.g. XOR gates have "0 cost", AND gates have "1 cost").

TODO:

- [ ] Oblivious transfer
- [ ] Parse Verilog
- [x] Convert Circuit to e-graph
- [ ] Convert e-graph to Circuit
