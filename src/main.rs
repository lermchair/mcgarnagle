use std::{collections::BTreeMap, fs::read_to_string};

use fernet::Fernet;
use rs::{
    evaluator::Evaluator, garbler::Garbler, optimizer::optimize, parser::parse_yosys_json,
    utils::wire_values,
};

fn main() {
    let delta = Fernet::generate_key();
    let file_path = "./circuits/synthesized.json".to_owned();
    let contents = read_to_string(file_path).expect("Couldn't find or load that file.");
    let (circuit, ins, outs) = parse_yosys_json(&contents);

    let out_keys = outs
        .iter()
        .map(|(_wire_name, wire_id)| {
            wire_id
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
        })
        .flatten()
        .collect::<Vec<String>>();

    let xor_optimized_circuit = optimize(circuit.clone(), out_keys.clone());

    let alice_input_keys = &ins["a"];

    let x = 12;
    let y = 12;

    let alice_input_values = wire_values(alice_input_keys, x);
    let mut alice_input_labels = BTreeMap::new();

    let bob_input_keys = &ins["b"];

    let bob_input_values = wire_values(bob_input_keys, y);
    let mut bob_input_labels = BTreeMap::new();

    let mut garbler = Garbler::new(
        delta,
        xor_optimized_circuit.clone(),
        ins.clone(),
        outs.clone(),
    );
    let (wire_to_keys, garbled_gates) = garbler.build();

    for (wire_id, value) in alice_input_values.iter() {
        let wire_key = wire_to_keys.get(wire_id).unwrap();
        let label = if value == &0 {
            &wire_key.0
        } else {
            &wire_key.1
        };
        alice_input_labels.insert(wire_id.clone(), label);
    }

    for (wire_id, value) in bob_input_values.iter() {
        let wire_key = wire_to_keys.get(wire_id).unwrap();
        let label = if value == &0 {
            &wire_key.0
        } else {
            &wire_key.1
        };
        bob_input_labels.insert(wire_id.clone(), label);
    }

    println!("Alice inputs: {:?}", alice_input_labels);
    println!("Bob inputs: {:?}", bob_input_labels);

    let mut evaluator = Evaluator::new(
        xor_optimized_circuit,
        out_keys.clone(),
        wire_to_keys.clone(),
        garbled_gates,
    );

    let result = evaluator.run(vec![alice_input_labels, bob_input_labels]);
    println!("Result: {:?}", result);

    let mut res: i64 = 0;
    for (i, wire) in out_keys.iter().enumerate() {
        let wire_value = *result.get(wire).unwrap() as i64;
        res |= wire_value << i;
    }
    println!("Result: {}", res);
}
