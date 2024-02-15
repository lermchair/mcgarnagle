from garbler import GarbledGate
from utils import GateType, bytes_xor, decrypt


class Evaluator:
    def __init__(
        self,
        wire_to_keys: dict[int, tuple[bytes, bytes]],
        gates: dict[int, GarbledGate],
    ):
        self.wire_to_keys = wire_to_keys  # wire_id -> (0_label, 1_label)
        self.gates = gates
        self.computed: dict[int, bytes] = {}  # wire_id -> value

    # inputs = [(wire_id, value)]
    def evaluate(
        self, inputs: list[tuple[int, bytes]], output_wires: list[int]
    ) -> dict[int, int]:
        wire_to_value: dict[int, int] = {}
        for wire_id, value in inputs:
            assert (
                self.wire_to_keys[wire_id][0] == value
                or self.wire_to_keys[wire_id][1] == value
            ), f"Input key {value.decode()} does not match with wire {wire_id} keys"
            self.computed[wire_id] = value

        sorted_tables = sorted(self.gates.items(), key=lambda table: table[0])

        for gate in sorted_tables:
            gate_id, garbled_gate = gate

            result = self.__evaluate_garbled_gate(
                garbled_gate,
                [self.computed[wire_id] for wire_id in garbled_gate.input_keys.keys()],
            )
            if result in garbled_gate.output_keys:
                self.computed[gate_id] = result
                wire_to_value[gate_id] = garbled_gate.output_keys.index(result)
            else:
                raise ValueError("No match found for output labels")

        return {output_wire: wire_to_value[output_wire] for output_wire in output_wires}

    def __evaluate_garbled_gate(
        self,
        garbled_gate: GarbledGate,
        inputs: list[bytes],
    ) -> bytes | None:
        if garbled_gate.operation == GateType.XOR:
            return bytes_xor(inputs[0], inputs[1])

        for _input_values, garbled_output in garbled_gate.table.items():
            decrypted_output = self.__try_decrypt(inputs, garbled_output)
            if decrypted_output is not None:
                return decrypted_output
        raise ValueError("No match found for output labels")

    def __try_decrypt(self, inputs, garbled_output):
        try:
            if len(inputs) == 1:
                return decrypt(inputs[0], garbled_output)
            else:
                return decrypt(inputs[1], decrypt(inputs[0], garbled_output))
        except Exception as e:
            # print(f"Type of exception: {type(e)}")
            return None
