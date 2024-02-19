import unittest

from circuit import Circuit
from evaluator import Evaluator
from garbler import Garbler
from utils import GateType


class TestGarbledCircuitNOR(unittest.TestCase):
    def setUp(self):
        self.circuit = {
            "a": None,
            "b": None,
            "a_nor_b": (GateType.NOR, "a", "b"),
        }

        self.garbler = Garbler(self.circuit, ["a", "b"], ["a_nor_b"])
        self.evaluator = Evaluator(
            self.circuit,
            ["a", "b"],
            ["a_nor_b"],
            self.garbler.wire_to_keys,
            self.garbler.garbled_gates,
        )

    def test_and_0_0(self):
        alice_input_a = self.garbler.wire_to_keys["a"][0]
        bob_input_b = self.garbler.wire_to_keys["b"][0]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_nor_b"],
        )
        # Check result
        print(result)
        self.assertEqual(result["a_nor_b"], 1, "NOR operation failed for inputs 0, 0")

    def test_nor_0_1(self):
        alice_input_a = self.garbler.wire_to_keys["a"][0]
        bob_input_b = self.garbler.wire_to_keys["b"][1]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_nor_b"],
        )
        self.assertEqual(result["a_nor_b"], 0, "NOR operation failed for inputs 0, 1")

    def test_nor_1_0(self):
        alice_input_a = self.garbler.wire_to_keys["a"][1]
        bob_input_b = self.garbler.wire_to_keys["b"][0]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_nor_b"],
        )
        self.assertEqual(result["a_nor_b"], 0, "NOR operation failed for inputs 1, 0")

    def test_nor_1_1(self):
        alice_input_a = self.garbler.wire_to_keys["a"][1]
        bob_input_b = self.garbler.wire_to_keys["b"][1]
        result = self.evaluator.evaluate(
            [{"a": alice_input_a}, {"b": bob_input_b}],
            ["a_nor_b"],
        )
        self.assertEqual(result["a_nor_b"], 0, "NOR operation failed for inputs 1, 1")


if __name__ == "__main__":
    unittest.main()
