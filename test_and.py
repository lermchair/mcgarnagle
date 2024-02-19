import unittest

from circuit import Circuit
from evaluator import Evaluator
from garbler import Garbler
from utils import GateType


class TestGarbledCircuitAND(unittest.TestCase):
    def setUp(self):
        # Setup common to all tests
        # self.circuit = Circuit()
        # self.a = self.circuit.add_wire()
        # self.b = self.circuit.add_wire()
        # self.a_xor_b = self.circuit.add_gate(GateType.XOR, [self.a, self.b])

        self.circuit = {
            "a": None,
            "b": None,
            "a_and_b": (GateType.AND, "a", "b"),
        }

        self.garbler = Garbler(self.circuit, ["a", "b"], ["a_and_b"])
        self.evaluator = Evaluator(
            self.circuit,
            ["a", "b"],
            ["a_and_b"],
            self.garbler.wire_to_keys,
            self.garbler.garbled_gates,
        )

    def test_and_0_0(self):
        alice_input_a = self.garbler.wire_to_keys["a"][0]
        bob_input_b = self.garbler.wire_to_keys["b"][0]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_and_b"],
        )
        # Check result
        print(result)
        self.assertEqual(result["a_and_b"], 0, "AND operation failed for inputs 0, 0")

    def test_and_0_1(self):
        alice_input_a = self.garbler.wire_to_keys["a"][0]
        bob_input_b = self.garbler.wire_to_keys["b"][1]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_and_b"],
        )
        self.assertEqual(result["a_and_b"], 0, "AND operation failed for inputs 0, 1")

    def test_and_1_0(self):
        alice_input_a = self.garbler.wire_to_keys["a"][1]
        bob_input_b = self.garbler.wire_to_keys["b"][0]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_and_b"],
        )
        self.assertEqual(result["a_and_b"], 0, "AND operation failed for inputs 1, 0")

    def test_and_1_1(self):
        alice_input_a = self.garbler.wire_to_keys["a"][1]
        bob_input_b = self.garbler.wire_to_keys["b"][1]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_and_b"],
        )
        self.assertEqual(result["a_and_b"], 1, "AND operation failed for inputs 1, 1")


if __name__ == "__main__":
    unittest.main()
