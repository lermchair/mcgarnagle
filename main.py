import json
import pdb
import sys
import timeit
import traceback
from typing import Optional

from circuit import Circuit, Gate
from evaluator import Evaluator
from garbler import Garbler
from optim import Optimizer
from utils import parse_yosys_json


def wire_values(wire_name: str, value: int, bitsize: int):
    bits = bin(value)[2:].zfill(bitsize)
    return {f"{wire_name}_{i+2}": int(bit) for i, bit in enumerate(reversed(bits))}


def wire_values2(input_keys: list[str], value: int):
    bits = bin(value)[2:].zfill(len(input_keys))
    # map each input_key to the bit value
    return {input_key: int(bit) for input_key, bit in zip(input_keys, reversed(bits))}


def main():
    f = open("synthesized_multiplier.json", "r")
    data = json.load(f)

    circ, ins, outs = parse_yosys_json(data)
    # x = 1185372425
    # y = 1337

    x = 123
    y = 123

    garbler = Garbler(circ, ins, outs)

    alice_input_keys = [f"a_{i}" for i in ins["a"]]
    print(alice_input_keys)

    alice_input_values = {**wire_values2(alice_input_keys, x)}

    for wire_id, value in alice_input_values.items():
        alice_input_values[wire_id] = garbler.wire_to_keys[wire_id][value]

    print(alice_input_values)

    bob_input_keys = [f"b_{i}" for i in ins["b"]]
    bob_input_values = {**wire_values2(bob_input_keys, y)}
    for wire_id, value in bob_input_values.items():
        bob_input_values[wire_id] = garbler.wire_to_keys[wire_id][value]

    evaluator = Evaluator(circ, ins, outs, garbler.wire_to_keys, garbler.garbled_gates)

    output_wires = [f"result_{i}" for i in outs["result"]]
    print(output_wires)

    result = evaluator.evaluate(
        [alice_input_values, bob_input_values],
        [output_wires],
    )

    res = 0
    for i, wire in enumerate(output_wires):
        res |= result[wire] << i
    print(f"Result is: {res}")

    print(f"{len(garbler.garbled_gates)} total gates")


if __name__ == "__main__":
    # try:
    # duration = timeit.timeit(stmt=main, number=10)
    # print(f"Average duration over 10 runs: {duration} seconds")
    main()
# except Exception:
#     extype, value, tb = sys.exc_info()
#     traceback.print_exc()
#     pdb.post_mortem(tb)
