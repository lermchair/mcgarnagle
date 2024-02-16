from egglog import (
    EGraph,
    Expr,
    check,
    eq,
    i64,
    i64Like,
    method,
    rewrite,
    ruleset,
    vars_,
    StringLike,
)

from circuit import Circuit, Gate, Wire
from utils import GateType


class Node(Expr):
    def __init__(self, value: i64Like | StringLike) -> None:
        ...

    # python has overloads for all these operators except for AND
    # so to keep things consistent, we'll just not overload any of them

    @method(cost=2)
    def and_(self, other: "Node") -> "Node":  # type: ignore[empty-body]
        ...

    @method(cost=2)
    def or_(self, other: "Node") -> "Node":  # type: ignore[empty-body]
        ...

    @method(cost=0)
    def xor_(self, other: "Node") -> "Node":  # type: ignore[empty-body]
        ...

    @method(cost=1)
    def not_(self) -> "Node":  # type: ignore[empty-body]
        ...


class Optimizer:
    def __init__(self):
        self.egraph = EGraph()

        a, b = vars_("a b", Node)
        # TODO: actually make good rules
        self.egraph.register(
            rewrite((a.or_(b)).and_((a.not_().and_(b)))).to(
                a.xor_(b)
            ),  # (a OR b) AND (NOT a AND b) -> a XOR b
            rewrite(a.not_().not_()).to(a),  # NOT NOT a -> a
            rewrite(a.and_(a.not_())).to(a.xor_(a)),  # a AND NOT a -> a XOR a
            rewrite((a.not_().and_(b)).or_(a.and_(b.not_()))).to(
                a.xor_(b)
            ),  # (NOT a AND b) OR (A and NOT b) -> a XOR b
            # (a AND NOT a) -> a XOR a
            rewrite(a.and_(a.not_())).to(a.xor_(a)),
            rewrite(a.not_().xor_(b.not_())).to(
                a.xor_(b)
            ),  # (NOT a) XOR (NOT b) -> a XOR b
            # rewrite(not_(a).or_(not_(b))).to(not_(a.and_(b))).to((a.xor(1)).and_(b.xor(1))),
        )

    def run(self, exprs: list[Node], iterations: int):
        self.egraph.run(iterations)

        for expr in exprs:
            x = self.egraph.extract(expr)
            print("simplified")
            print(x)

    def parse_circuit(self, circuit: Circuit) -> list[Node]:
        output_wires = circuit.get_output_wires()
        visited_gates: set[int] = set()
        gates: list[list[Gate]] = []
        exprs: list[Node] = []

        for wire in output_wires:
            circuit_structure: list[Gate] = []
            self._traverse_wire(wire, visited_gates, circuit_structure, circuit)
            gates.append(circuit_structure)

        for gate_list in gates:
            expr = self._gates_to_expr(gate_list)
            print(expr)
            exprs.append(expr)
            self.egraph.let(f"expr_{gate_list[0].output.id}", expr)

        return exprs

    def _gates_to_expr(self, gates: list[Gate]):
        node_dict: dict[int, Node] = {}

        def get_node(node_id):
            if node_id in node_dict:
                return node_dict[node_id]
            else:
                # Assuming non-existent nodes are created with default values
                node = Node(node_id)
                node_dict[node_id] = node
                return node

        for gate in gates:
            inputs = gate.inputs
            output_id = gate.output
            operation = gate.gate_type

            if operation == GateType.AND:
                result = get_node(inputs[0].id).and_(get_node(inputs[1].id))
            elif operation == GateType.OR:
                result = get_node(inputs[0].id).or_(get_node(inputs[1].id))
            elif operation == GateType.XOR:
                result = get_node(inputs[0].id).xor_(get_node(inputs[1].id))
            elif operation == GateType.NOT:
                result = get_node(inputs[0].id).not_()
            else:
                raise ValueError(f"Unknown operation: {operation}")

            node_dict[output_id.id] = result

        # Assuming the last gate's output is the final expression's output
        return node_dict[gates[-1].output.id]

    def _traverse_wire(
        self,
        wire: Wire,
        visited_gates: set,
        circuit_structure: list[Gate],
        circuit: Circuit,
    ):
        # Check if the wire is an output of any gate using the wire_to_gate dictionary
        gate = circuit.wire_to_gate.get(wire.id)
        if gate and gate.output.id not in visited_gates:
            visited_gates.add(gate.output.id)
            # Recursively traverse the input wires of the gate
            for input_wire in gate.inputs:
                self._traverse_wire(
                    input_wire, visited_gates, circuit_structure, circuit
                )
            # Add the gate to the circuit structure
            circuit_structure.append(gate)
