from dataclasses import dataclass

from graphviz import Digraph

from utils import GateType


@dataclass
class Wire:
    id: int


@dataclass
class Gate:
    gate_type: GateType
    inputs: list[Wire]
    output: Wire


class Circuit:
    def __init__(self) -> None:
        self.gates: list[Gate] = []
        self.wires: dict[int, Wire] = {}
        self.wire_nonce: int = 0
        self.wire_to_gate: dict[int, Gate] = {}

    def add_wire(self) -> Wire:
        wire = Wire(self.wire_nonce)
        self.wires[self.wire_nonce] = wire
        self.wire_nonce += 1
        return wire

    def add_gate(self, gate: GateType, inputs: list[Wire]) -> Wire:
        for wire in inputs:
            assert wire.id in self.wires, f"Input wire {wire} does not exist"
        output_wire = Wire(self.wire_nonce)
        self.wires[self.wire_nonce] = output_wire
        new_gate = Gate(gate, inputs, self.wires[self.wire_nonce])
        self.gates.append(new_gate)
        self.wire_to_gate[output_wire.id] = new_gate
        self.wire_nonce += 1
        return output_wire

    def get_output_wires(self) -> list[Wire]:
        input_wire_ids = {wire.id for gate in self.gates for wire in gate.inputs}
        return [wire for wire in self.wires.values() if wire.id not in input_wire_ids]

    def pretty_print_circuit(self, view: bool = True) -> None:
        dot = Digraph(comment="Garbled Circuit", format="png")
        dot.attr(rankdir="LR")
        dot.attr("edge", arrowhead="vee", arrowsize="0.5")

        # Styling for wire nodes
        wire_style = {"shape": "plaintext", "fontcolor": "blue"}

        # Styling for gate nodes
        gate_style = {
            "shape": "box",
            "style": "filled",
            "color": "lightgrey",
            "fontcolor": "black",
        }

        for wire_id, wire in self.wires.items():
            dot.node(f"wire{wire_id}", f"Wire {wire_id}", **wire_style)

        for gate in self.gates:
            gate_label = f"{gate.gate_type.value} (out {gate.output.id})"
            gate_name = f"gate_{gate.gate_type.value}_out_{gate.output.id}".replace(
                " ", "_"
            )
            dot.node(gate_name, gate_label, **gate_style)
            for wire in gate.inputs:
                dot.edge(f"wire{wire.id}", gate_name, color="darkgreen")
            dot.edge(gate_name, f"wire{gate.output.id}", color="red")

        dot.render("garbled_circuit.gv", view=view)
