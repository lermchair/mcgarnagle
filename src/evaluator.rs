use std::{collections::BTreeMap, error::Error, str::from_utf8};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};

use crate::{
    garbler::GarbledGate,
    utils::{bytes_xor, decrypt, generate_encryption_key, topo_sort_wires, Circuit, GateType},
};

pub struct Evaluator {
    circuit: Circuit,
    outputs: Vec<String>,
    wire_to_keys: BTreeMap<String, (String, String)>,
    gates: BTreeMap<String, GarbledGate>,
    computed: BTreeMap<String, String>,
}

impl Evaluator {
    pub fn new(
        circuit: Circuit,
        outputs: Vec<String>,
        wire_to_keys: BTreeMap<String, (String, String)>,
        gates: BTreeMap<String, GarbledGate>,
    ) -> Evaluator {
        Evaluator {
            circuit,
            outputs,
            wire_to_keys,
            gates,
            computed: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, inputs: Vec<BTreeMap<String, &String>>) -> BTreeMap<String, i32> {
        let mut wire_to_value: BTreeMap<String, i32> = BTreeMap::new();

        for party in inputs.iter() {
            for (wire_id, value) in party.iter() {
                assert!(
                    &self.wire_to_keys[wire_id].0 == *value
                        || &self.wire_to_keys[wire_id].1 == *value,
                    "Input value does not match keys"
                );
                self.computed.insert(wire_id.to_string(), value.to_string());
            }
        }

        let sorted_wires = topo_sort_wires(&self.circuit);
        for wire in &sorted_wires {
            if !self.gates.contains_key(wire) {
                continue;
            }
            let garbled_gate = self.gates.get(wire).unwrap();

            let gate_inputs: Vec<String> = garbled_gate
                .input_wire_ids
                .iter()
                .map(|wire_id| self.computed.get(wire_id).unwrap().clone())
                .collect();

            // println!(
            //     "Evaluating gate {:?} with inputs {:?}",
            //     garbled_gate.operation, garbled_gate.input_wire_ids
            // );

            let result = self.evaluate_garbled_gate(garbled_gate, gate_inputs);

            match result {
                Ok(value) => {
                    if garbled_gate.output_keys.0 == value || garbled_gate.output_keys.1 == value {
                        self.computed.insert(wire.clone(), value.clone());
                        if value == garbled_gate.output_keys.0 {
                            wire_to_value.insert(wire.clone(), 0);
                        } else if value == garbled_gate.output_keys.1 {
                            wire_to_value.insert(wire.clone(), 1);
                        }
                    } else {
                        panic!("Output value does not match the keys");
                    }
                }
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            }
        }

        let mut result = BTreeMap::new();
        for output_wire in self.outputs.iter() {
            result.insert(
                output_wire.to_string(),
                *wire_to_value.get(output_wire).unwrap(),
            );
        }
        result
    }

    fn evaluate_garbled_gate(
        &self,
        garbled_gate: &GarbledGate,
        inputs: Vec<String>,
    ) -> Result<String, Box<dyn Error>> {
        if garbled_gate.operation == GateType::XOR {
            let in_a = URL_SAFE.decode(&inputs[0]).unwrap();
            let in_b = URL_SAFE.decode(&inputs[1]).unwrap();
            let result_bytes = bytes_xor(&in_a, &in_b);
            let result = URL_SAFE.encode(&result_bytes);
            return Ok(result);
        }

        for garbled_output in &garbled_gate.table {
            if let Some(decrypted_output) = self.try_decrypt(&inputs, garbled_output) {
                return Ok(decrypted_output);
            }
        }

        Err(format!(
            "No match found for output labels, {:?}",
            garbled_gate.output_keys
        )
        .into())
    }

    fn try_decrypt(&self, inputs: &Vec<String>, garbled_output: &String) -> Option<String> {
        let bytes_inputs = inputs.iter().map(|i| i.as_bytes()).collect::<Vec<&[u8]>>();
        let key = generate_encryption_key(&bytes_inputs);
        let encoded_key = URL_SAFE.encode(&key);
        let decrypted = decrypt(encoded_key, garbled_output.to_string());
        match decrypted {
            Ok(decrypted) => {
                return Some(from_utf8(&decrypted).unwrap().to_string());
            }
            Err(_) => None,
        }
    }
}
