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
    ) -> Self {
        Self {
            circuit,
            outputs,
            wire_to_keys,
            gates,
            computed: BTreeMap::new(),
        }
    }

    pub fn run(&mut self, inputs: Vec<BTreeMap<String, &String>>) -> BTreeMap<String, i32> {
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

        let mut wire_to_value = BTreeMap::new();
        for wire in topo_sort_wires(&self.circuit) {
            if let Some(garbled_gate) = self.gates.get(&wire) {
                let gate_inputs: Vec<String> = garbled_gate
                    .input_wire_ids
                    .iter()
                    .map(|wire_id| self.computed[wire_id].clone())
                    .collect();

                let result = self
                    .evaluate_garbled_gate(garbled_gate, gate_inputs)
                    .unwrap_or_else(|e| panic!("Error: {:?}", e));

                assert!(
                    garbled_gate.output_keys.0 == result || garbled_gate.output_keys.1 == result,
                    "Output value does not match the keys"
                );

                self.computed.insert(wire.clone(), result.clone());
                wire_to_value.insert(
                    wire.clone(),
                    if result == garbled_gate.output_keys.0 {
                        0
                    } else {
                        1
                    },
                );
            }
        }
        self.outputs
            .iter()
            .map(|output_wire| (output_wire.to_string(), wire_to_value[output_wire]))
            .collect()
    }

    fn evaluate_garbled_gate(
        &self,
        garbled_gate: &GarbledGate,
        inputs: Vec<String>,
    ) -> Result<String, Box<dyn Error>> {
        if garbled_gate.operation == GateType::XOR {
            let result_bytes = bytes_xor(
                &URL_SAFE.decode(&inputs[0]).unwrap(),
                &URL_SAFE.decode(&inputs[1]).unwrap(),
            );
            return Ok(URL_SAFE.encode(&result_bytes));
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

    fn try_decrypt(&self, inputs: &[String], garbled_output: &str) -> Option<String> {
        let key = generate_encryption_key(&inputs.iter().map(|i| i.as_bytes()).collect::<Vec<_>>());
        decrypt(URL_SAFE.encode(&key), garbled_output.to_string())
            .ok()
            .and_then(|decrypted| from_utf8(&decrypted).map(|s| s.to_string()).ok())
    }
}
