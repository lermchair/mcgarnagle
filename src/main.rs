use std::{collections::BTreeMap, fs::read_to_string};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use fernet::Fernet;
use rs::{
    evaluator::Evaluator,
    garbler::{GarbledGate, Garbler},
    optimizer::optimize,
    ot::{Receiver, Sender},
    parser::parse_bristol_fashion,
    utils::{wire_values, GateType},
};

fn alice_setup(
    input: i32,
    circuit: BTreeMap<String, (GateType, Vec<String>)>,
    ins: BTreeMap<String, Vec<String>>,
    outs: BTreeMap<String, Vec<String>>,
    delta: String,
) -> (
    Sender,
    Vec<String>,
    BTreeMap<String, (String, String)>,
    BTreeMap<String, GarbledGate>,
    BTreeMap<String, String>,
) {
    let alice = Sender::new();

    let alice_input_keys = &ins["a"];

    let alice_input_values = wire_values(alice_input_keys, input);
    let mut alice_input_labels = BTreeMap::new();

    let mut garbler = Garbler::new(delta, circuit, ins.clone(), outs);

    let (wire_to_keys, garbled_gates) = garbler.build();
    for (wire_id, value) in alice_input_values.iter() {
        let wire_key = wire_to_keys.get(wire_id).unwrap();
        let label = if value == &0 {
            &wire_key.0
        } else {
            &wire_key.1
        };
        alice_input_labels.insert(wire_id.to_string(), label.to_string());
    }

    let bob_input_keys = &ins["b"];
    (
        alice,
        bob_input_keys.clone(),
        wire_to_keys,
        garbled_gates,
        alice_input_labels,
    )
}

fn main() {
    let delta = Fernet::generate_key();
    let file_path = "./circuits/add64.txt".to_owned();
    let contents = read_to_string(file_path).expect("Couldn't find or load file.");
    let (circuit, ins, outs) = parse_bristol_fashion(&contents);

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

    let alice_input_raw = 999;
    let bob_input_raw = 77;

    let (alice, bob_input_keys, wire_to_keys, garbled_gates, alice_input_labels) = alice_setup(
        alice_input_raw,
        xor_optimized_circuit.clone(),
        ins,
        outs,
        delta,
    );
    let mut bob = Receiver::new();

    let bob_input_values = wire_values(&bob_input_keys, bob_input_raw);
    let mut bob_input_labels = BTreeMap::new();
    for (wire_id, value) in bob_input_values.iter() {
        let choice = bob.choose(&alice.s, *value as u8);
        let wire_key = wire_to_keys.get(wire_id).unwrap();
        let alice_keys = alice.derive_keys(choice);

        // wire_keys are base64 encoded
        let m1 = URL_SAFE.decode(wire_key.0.clone()).unwrap();
        let m2 = URL_SAFE.decode(wire_key.1.clone()).unwrap();

        let (e0, e1) = alice.encrypt(alice_keys.0.to_vec(), alice_keys.1.to_vec(), &m1, &m2);

        let bob_key = bob.derive_key();
        let decrypted = bob.decrypt_ciphertexts(bob_key, (e0, e1)).unwrap();
        bob_input_labels.insert(wire_id.clone(), URL_SAFE.encode(&decrypted));
    }

    let mut evaluator = Evaluator::new(
        xor_optimized_circuit,
        out_keys.clone(),
        wire_to_keys.clone(),
        garbled_gates,
    );

    let result = evaluator.run(vec![alice_input_labels, bob_input_labels]);
    let mut res: i64 = 0;
    for (i, wire) in out_keys.iter().enumerate() {
        let wire_value = *result.get(wire).unwrap() as i64;
        res |= wire_value << i;
    }
    println!("RESULT: {}", res);
}
