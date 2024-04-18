use std::collections::BTreeMap;
use std::option::Option;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};

use crate::utils::{
    bytes_xor, encrypt, generate_encryption_key, generate_keys, topo_sort_wires, Circuit, GateType,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
pub struct GarbledGate {
    pub operation: GateType,
    pub table: Vec<String>,
    pub input_wire_ids: Vec<String>,
    pub output_keys: (String, String),
}

#[derive(Debug)]
pub struct Garbler {
    delta: String,
    circuit: Circuit,
    wire_to_keys: BTreeMap<String, (String, String)>,
    garbled_gates: BTreeMap<String, GarbledGate>,
}

impl Garbler {
    pub fn new(
        delta: String,
        circuit: Circuit,
        ins: BTreeMap<String, Vec<String>>,
        outs: BTreeMap<String, Vec<String>>,
    ) -> Self {
        let mut wire_to_keys: BTreeMap<String, (String, String)> = BTreeMap::new();

        for (_wire_name, wire_id) in ins.iter() {
            for i in wire_id {
                wire_to_keys.insert(i.to_string(), generate_keys(&delta));
            }
        }

        for (_wire_name, wire_id) in outs.iter() {
            for i in wire_id {
                wire_to_keys.insert(i.to_string(), generate_keys(&delta));
            }
        }

        Self {
            delta,
            circuit,
            wire_to_keys,
            garbled_gates: BTreeMap::new(),
        }
    }

    pub fn build(
        &mut self,
    ) -> (
        BTreeMap<String, (String, String)>,
        BTreeMap<String, GarbledGate>,
    ) {
        let sorted_wires = topo_sort_wires(&self.circuit);
        for wire in sorted_wires {
            let (gate_type, gate_inputs) = self.circuit.get(&wire).unwrap();
            if gate_type == &GateType::INPUT {
                continue;
            }

            let gate_wire_to_keys_cloned: BTreeMap<String, (String, String)> = gate_inputs
                .iter()
                .map(|input_wire| {
                    let key_pair = self.wire_to_keys.get(input_wire).unwrap().clone();
                    (input_wire.clone(), key_pair)
                })
                .collect();

            let gate_wire_to_keys: BTreeMap<&String, &(String, String)> = gate_wire_to_keys_cloned
                .iter()
                .map(|(k, v)| (k, v))
                .collect();

            let output_wire_keys = self.wire_to_keys.get(&wire).cloned();
            let gg = self.garble_gate(
                *gate_type,
                gate_wire_to_keys,
                output_wire_keys,
                gate_inputs.to_vec(),
            );

            self.wire_to_keys
                .insert(wire.clone(), gg.output_keys.clone());
            self.garbled_gates.insert(wire, gg);
        }
        (self.wire_to_keys.clone(), self.garbled_gates.clone())
    }

    fn switch_gate(&self, gate_type: GateType, a_val: bool, b_val: bool) -> bool {
        match gate_type {
            GateType::AND => a_val & b_val,
            GateType::OR => a_val | b_val,
            GateType::NOR => !(a_val | b_val),
            GateType::ORNOT => a_val | !b_val,
            GateType::NAND => !(a_val & b_val),
            GateType::ANDNOT => a_val & !b_val,
            GateType::XNOR => a_val == b_val,
            GateType::XOR => a_val ^ b_val,
            GateType::NOT => !a_val,
            GateType::CONST_0 => false,
            GateType::CONST_1 => true,
            GateType::CONST => a_val,
            GateType::INPUT => a_val,
        }
    }

    pub fn garble_gate(
        &mut self,
        gate_op: GateType,
        gate_wire_to_keys: BTreeMap<&String, &(String, String)>,
        output: Option<(String, String)>,
        gate_input_names: Vec<String>,
    ) -> GarbledGate {
        assert!(
            (gate_op == GateType::NOT
                || gate_op == GateType::CONST_0
                || gate_op == GateType::CONST_1
                || gate_op == GateType::CONST)
                && gate_wire_to_keys.len() == 1
                || gate_wire_to_keys.len() == 2,
            "Invalid number of inputs for gate type"
        );

        let (in_keys_a, in_keys_b) = (
            &gate_wire_to_keys[&gate_input_names[0]],
            if gate_input_names.len() > 1 {
                gate_wire_to_keys.get(&gate_input_names[1])
            } else {
                None
            },
        );

        if gate_op == GateType::XOR {
            let safe_in_keys_a = URL_SAFE.decode(&in_keys_a.0).unwrap();
            let safe_in_keys_b = URL_SAFE.decode(&in_keys_b.unwrap().0).unwrap();
            let output_0_bytes = bytes_xor(&safe_in_keys_a, &&safe_in_keys_b);

            let safe_delta = URL_SAFE.decode(&self.delta).unwrap();
            let output_1_bytes = bytes_xor(&output_0_bytes, &safe_delta);

            let base64_output_0 = URL_SAFE.encode(&output_0_bytes);
            let base64_output_1 = URL_SAFE.encode(&output_1_bytes);

            return GarbledGate {
                operation: gate_op,
                table: vec![], // free xor gate, no table
                input_wire_ids: gate_input_names,
                output_keys: (base64_output_0, base64_output_1),
            };
        }

        let output_labels = output.unwrap_or_else(|| generate_keys(&self.delta));
        let mut garbled_table: Vec<String> = vec![];

        for (a_val, a_key) in [(0, &in_keys_a.0), (1, &in_keys_a.1)] {
            if let Some(in_keys_b) = in_keys_b {
                for (b_val, b_key) in [(0, &in_keys_b.0), (1, &in_keys_b.1)] {
                    let out_val = self.switch_gate(gate_op, a_val == 1, b_val == 1);
                    let out_bytes_val = if out_val {
                        &output_labels.1
                    } else {
                        &output_labels.0
                    };

                    let key = generate_encryption_key(&[a_key.as_bytes(), b_key.as_bytes()]);
                    let encoded_key = URL_SAFE.encode(&key);

                    garbled_table.push(encrypt(&encoded_key, out_bytes_val.as_bytes().to_vec()));
                }
            } else {
                let out_val = if a_val == 1 { 0 } else { 1 };
                let out_bytes_val = if out_val == 1 {
                    &output_labels.1
                } else {
                    &output_labels.0
                };

                let key = generate_encryption_key(&[a_key.as_bytes()]);
                let encoded_key = URL_SAFE.encode(&key);

                garbled_table.push(encrypt(&encoded_key, out_bytes_val.as_bytes().to_vec()));
            }
        }

        garbled_table.shuffle(&mut thread_rng());

        GarbledGate {
            operation: gate_op,
            table: garbled_table,
            input_wire_ids: gate_input_names,
            output_keys: (output_labels.0, output_labels.1),
        }
    }
}
