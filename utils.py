import base64
import enum
from typing import Optional, Set

from cryptography.fernet import Fernet


class GateType(enum.Enum):
    AND = "AND"
    NOT = "NOT"
    OR = "OR"
    XOR = "XOR"
    ORNOT = "ORNOT"
    NOR = "NOR"
    NAND = "NAND"
    ANDNOT = "ANDNOT"
    XNOR = "XNOR"


def generate_keys(delta: bytes) -> tuple[bytes, bytes]:
    zero = Fernet.generate_key()
    return zero, bytes_xor(zero, delta)


def encrypt(key: bytes, data: bytes) -> bytes:
    return Fernet(key).encrypt(data)


def decrypt(key: bytes, data: bytes) -> bytes:
    return Fernet(key).decrypt(data)


def bytes_xor(a: bytes, b: bytes) -> bytes:
    key1_bytes = base64.urlsafe_b64decode(a)
    key2_bytes = base64.urlsafe_b64decode(b)
    xor_result = bytes(a ^ b for a, b in zip(key1_bytes, key2_bytes))
    return base64.urlsafe_b64encode(xor_result)


def topo_sort_wires(
    circuit, inputs, outputs
) -> dict[str, Optional[tuple[GateType, str, Optional[str]]]]:
    sorted_circuit: list[str] = []
    visited: Set[str] = set()

    def visit(wire_name: str):
        if wire_name in visited:
            return
        visited.add(wire_name)
        if wire_name not in inputs:
            if circuit[wire_name] is None:
                return
            gate, *input_wire_names = circuit[wire_name]
            for input_wire in input_wire_names:
                visit(input_wire)
        sorted_circuit.append(wire_name)

    output_wires = [
        f"{output}_{bit}" for output, bits in outputs.items() for bit in bits
    ]
    for wire in output_wires:
        visit(wire)
    return {k: circuit[k] for k in sorted_circuit}


def parse_yosys_json(contents: dict):
    # parse the ports
    circuit: dict[str, Optional[tuple[GateType, str, Optional[str]]]] = {}
    inputs: dict[str, list[int]] = {}
    outputs: dict[str, list[int]] = {}

    gate_type_mapping = {
        "$_AND_": GateType.AND,
        "$_OR_": GateType.OR,
        "$_NOT_": GateType.NOT,
        "$_XOR_": GateType.XOR,
        "$_NOR_": GateType.NOR,
        "$_NAND_": GateType.NAND,
        "$_ANDNOT_": GateType.ANDNOT,
        "$_ORNOT_": GateType.ORNOT,
        "$_XNOR_": GateType.XNOR,
    }

    for module_name, modules in contents["modules"].items():
        for port_name, port in modules["ports"].items():
            if port["direction"] == "input":
                for bit in port["bits"]:
                    if port_name in inputs:
                        inputs[port_name].append(bit)
                    else:
                        inputs[port_name] = [bit]
                    circuit[f"{port_name}_{bit}"] = None
            elif port["direction"] == "output":
                for bit in port["bits"]:
                    # append to outputs[port_name]
                    if port_name in outputs:
                        outputs[port_name].append(bit)
                    else:
                        outputs[port_name] = [bit]
                    circuit[f"{port_name}_{bit}"] = None
            else:
                raise ValueError("unsupported port direction:", port["direction"])
        flattened_outputs = [item for sublist in outputs.values() for item in sublist]
        flattened_inputs = [item for sublist in inputs.values() for item in sublist]
        for cell_name, cell in modules["cells"].items():
            gate_type = cell["type"]
            gate_out: str = None
            gate_in: list[str] = []
            for port_dir, connection in zip(
                cell["port_directions"].values(), cell["connections"].values()
            ):
                if port_dir == "input":
                    is_external_input = connection[0] in flattened_inputs
                    if is_external_input:
                        input_name = list(
                            filter(lambda x: connection[0] in inputs[x], inputs.keys())
                        )[0]
                        gate_in.append(f"{input_name}_{connection[0]}")
                    else:
                        gate_in.extend(connection)
                elif port_dir == "output":
                    gate_out = connection[0]
                else:
                    raise ValueError("unsupported port direction:", port_dir)

                if gate_type in gate_type_mapping:
                    is_external_output = gate_out in flattened_outputs
                    gate_enum = gate_type_mapping[gate_type]

                    if is_external_output:
                        output_name = list(filter(lambda x: x != gate_out, outputs))[0]
                        circuit[f"{output_name}_{gate_out}"] = (gate_enum, *gate_in)
                    else:
                        circuit[gate_out] = (gate_enum, *gate_in)
                else:
                    raise ValueError("unsupported gate type:", gate_type)
    return circuit, inputs, outputs
