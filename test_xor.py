import unittest

from circuit import Circuit
from evaluator import Evaluator
from garbler import Garbler
from utils import GateType


class TestGarbledCircuitXOR(unittest.TestCase):
    def setUp(self):
        # Setup common to all tests
        self.circuit = Circuit()
        self.a = self.circuit.add_wire()
        self.b = self.circuit.add_wire()
        self.a_xor_b = self.circuit.add_gate(GateType.XOR, [self.a, self.b])
        self.garbler = Garbler(self.circuit)
        self.evaluator = Evaluator(
            self.garbler.wire_to_keys, self.garbler.garbled_gates
        )

    def test_xor_0_0(self):
        # Inputs
        alice_input_a = self.garbler.wire_to_keys[self.a.id][0]  # Alice's A input is 0
        bob_input_b = self.garbler.wire_to_keys[self.b.id][0]  # Bob's B input is 0
        # Evaluation
        result = self.evaluator.evaluate(
            [(self.a.id, alice_input_a), (self.b.id, bob_input_b)], [self.a_xor_b.id]
        )
        # Check result
        self.assertEqual(
            result[self.a_xor_b.id], 0, "XOR operation failed for inputs 0, 0"
        )

    def test_xor_0_1(self):
        alice_input_a = self.garbler.wire_to_keys[self.a.id][0]
        bob_input_b = self.garbler.wire_to_keys[self.b.id][1]
        result = self.evaluator.evaluate(
            [(self.a.id, alice_input_a), (self.b.id, bob_input_b)], [self.a_xor_b.id]
        )
        self.assertEqual(
            result[self.a_xor_b.id], 1, "XOR operation failed for inputs 0, 1"
        )

    def test_xor_1_0(self):
        alice_input_a = self.garbler.wire_to_keys[self.a.id][1]
        bob_input_b = self.garbler.wire_to_keys[self.b.id][0]
        result = self.evaluator.evaluate(
            [(self.a.id, alice_input_a), (self.b.id, bob_input_b)], [self.a_xor_b.id]
        )
        self.assertEqual(
            result[self.a_xor_b.id], 1, "XOR operation failed for inputs 1, 0"
        )

    def test_xor_1_1(self):
        alice_input_a = self.garbler.wire_to_keys[self.a.id][1]
        bob_input_b = self.garbler.wire_to_keys[self.b.id][1]
        result = self.evaluator.evaluate(
            [(self.a.id, alice_input_a), (self.b.id, bob_input_b)], [self.a_xor_b.id]
        )
        self.assertEqual(
            result[self.a_xor_b.id], 0, "XOR operation failed for inputs 1, 1"
        )


if __name__ == "__main__":
    unittest.main()
