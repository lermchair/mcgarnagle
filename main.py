import pdb
import sys
import traceback
from circuit import Circuit
from evaluator import Evaluator
from garbler import Garbler
from optim import Optimizer


from utils import GateType


def main():
    # 1-bit full adder circuit
    c = Circuit()
    a = c.add_wire()  # A input
    b = c.add_wire()  # B input
    cin = c.add_wire()  # Carry-in input

    # Sum
    a_xor_b = c.add_gate(GateType.XOR, [a, b])  # Intermediate sum (A XOR B)
    # Final sum (A XOR B XOR Cin)
    sum_ = c.add_gate(GateType.XOR, [a_xor_b, cin])

    # Carry-out
    a_and_b = c.add_gate(GateType.AND, [a, b])  # A AND B
    b_and_cin = c.add_gate(GateType.AND, [b, cin])  # B AND Cin
    a_and_cin = c.add_gate(GateType.AND, [a, cin])  # A AND Cin

    intermediate_carry1 = c.add_gate(
        GateType.OR, [a_and_b, b_and_cin]
    )  # (A AND B) OR (B AND Cin)
    carry_out = c.add_gate(
        GateType.OR, [intermediate_carry1, a_and_cin]
    )  # (A AND B) OR (B AND Cin) OR (A AND Cin)

    # c.pretty_print_circuit()
    optim = Optimizer()
    parsed = optim.parse_circuit(c)
    optim.run(parsed, 100)

    garbler = Garbler(c)
    evaluator = Evaluator(garbler.wire_to_keys, garbler.garbled_gates)

    alice_input_a = garbler.wire_to_keys[a.id][0]
    bob_input_b = garbler.wire_to_keys[b.id][1]
    cin_input = garbler.wire_to_keys[cin.id][0]

    result = evaluator.evaluate(
        [(a.id, alice_input_a), (b.id, bob_input_b), (cin.id, cin_input)],
        [sum_.id, carry_out.id],
    )

    print(result)


if __name__ == "__main__":
    # try:
    main()
# except Exception:
# extype, value, tb = sys.exc_info()
# traceback.print_exc()
# pdb.post_mortem(tb)
