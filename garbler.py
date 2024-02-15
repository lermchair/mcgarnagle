from dataclasses import dataclass
from random import shuffle
from typing import Optional

from cryptography.fernet import Fernet

from circuit import Circuit
from utils import GateType, bytes_xor, encrypt, generate_keys


@dataclass
class GarbledGate:
    operation: GateType
    table: dict[tuple[int, Optional[int]], bytes]  # lhs, rhs -> output
    input_keys: dict[int, tuple[bytes, bytes]]  # wire_id -> (0_label, 1_label)
    output_keys: tuple[bytes, bytes]


class Garbler:
    def __init__(self, circuit: Circuit):
        self.__delta = Fernet.generate_key()
        self.circuit = circuit
        self.wire_to_keys: dict[int, tuple[bytes, bytes]] = {}
        # gate_id -> garbled_gate
        self.garbled_gates: dict[int, GarbledGate] = {}

        for id, wire in self.circuit.wires.items():
            self.wire_to_keys[id] = generate_keys(self.__delta)

        for gate in self.circuit.gates:
            print(f"Garbling gate {gate.gate_type} with inputs {gate.inputs}")
            garbled = self.garble_gate(
                gate.gate_type,
                {wire.id: self.wire_to_keys[wire.id] for wire in gate.inputs},
                self.wire_to_keys[gate.output.id],
            )
            self.wire_to_keys[gate.output.id] = (
                garbled.output_keys[0],
                garbled.output_keys[1],
            )
            self.garbled_gates[gate.output.id] = garbled

    def garble_gate(
        self,
        gate_op: GateType,
        wire_to_labels: dict[int, tuple[bytes, bytes]],
        output: Optional[tuple[bytes, bytes]],
    ) -> GarbledGate:
        if gate_op == GateType.NOT:
            assert len(wire_to_labels) == 1, "NOT gate only has one input"
        else:
            assert len(wire_to_labels) == 2, "Gate expects have 2 inputs"

        # a_val, b_val -> output
        garbled_table: dict[tuple[int, Optional[int]], bytes] = {}

        wire_ids = list(wire_to_labels.keys())
        in_key_a_0, in_key_a_1 = wire_to_labels[wire_ids[0]]
        # TODO: handle case where there is only one input
        in_key_b_0, in_key_b_1 = wire_to_labels[wire_ids[1]]

        if gate_op == GateType.XOR:
            output_0_label = bytes_xor(in_key_a_0, in_key_b_0)
            output_1_label = bytes_xor(output_0_label, self.__delta)

            return GarbledGate(
                gate_op, {}, wire_to_labels, (output_0_label, output_1_label)
            )

        if output is None:
            output_0_label, output_1_label = generate_keys(self.__delta)
        else:
            output_0_label, output_1_label = output

        for a_val in (0, 1):
            for b_val in (0, 1) if gate_op != GateType.NOT else [None]:
                if gate_op == GateType.NOT:
                    in_key_a = wire_to_labels[0][a_val]
                    out_val = not a_val
                    out_bytes_val = output_1_label if out_val else output_0_label
                    garbled_table[(a_val, None)] = encrypt(in_key_a, out_bytes_val)
                else:
                    in_key_a = in_key_a_0 if a_val == 0 else in_key_a_1
                    in_key_b = in_key_b_0 if b_val == 0 else in_key_b_1

                    switch = {
                        GateType.AND: lambda a_val, b_val: a_val and b_val,
                        GateType.OR: lambda a_val, b_val: a_val or b_val,
                    }
                    out_op = switch[gate_op]
                    out_val = out_op(a_val, b_val)
                    out_bytes_val = output_1_label if out_val else output_0_label
                    garbled_table[(a_val, b_val)] = encrypt(
                        in_key_a, encrypt(in_key_b, out_bytes_val)
                    )

        items = list(garbled_table.items())
        shuffle(items)
        shuffled_table = dict(items)
        return GarbledGate(
            gate_op, shuffled_table, wire_to_labels, (output_0_label, output_1_label)
        )
