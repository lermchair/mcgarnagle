use criterion::{criterion_group, criterion_main, Criterion};
use fernet::Fernet;
use rs::evaluator::Evaluator;
use rs::garbler::{GarbledGate, Garbler};
use rs::optimizer::optimize;
use rs::parser;
use rs::utils::{topo_sort_wires, wire_values, Circuit, GateType};
use std::collections::BTreeMap;
use std::fs::read_to_string;

fn garble_and_evaluate_prepared(
    circuit: &Circuit,
    ins: &BTreeMap<String, Vec<String>>,
    wire_to_keys: &BTreeMap<String, (String, String)>,
    garbled_gates: &BTreeMap<String, GarbledGate>,
) {
    let alice_input_keys = &ins["a"];
    let bob_input_keys = &ins["b"];

    // Assuming wire_values and other necessary functions are accessible
    let alice_input_values = wire_values(alice_input_keys, 3); // Example values
    let bob_input_values = wire_values(bob_input_keys, 3);

    let mut alice_input_labels = BTreeMap::new();
    let mut bob_input_labels = BTreeMap::new();

    for (wire_id, value) in alice_input_values.iter() {
        let wire_key = wire_to_keys.get(wire_id).unwrap();
        let label = if value == &0 {
            &wire_key.0
        } else {
            &wire_key.1
        };
        alice_input_labels.insert(wire_id.clone(), label.to_string());
    }

    for (wire_id, value) in bob_input_values.iter() {
        let wire_key = wire_to_keys.get(wire_id).unwrap();
        let label = if value == &0 {
            &wire_key.0
        } else {
            &wire_key.1
        };
        bob_input_labels.insert(wire_id.clone(), label.to_string());
    }

    // Assuming the setup for evaluator is similar to the one in your main function
    let sorted_wires = topo_sort_wires(&circuit);

    let mut output_wires = Vec::new();
    for wire in sorted_wires.iter() {
        if !garbled_gates
            .values()
            .any(|gate| gate.input_wire_ids.contains(&wire))
        {
            output_wires.push(wire.clone());
        }
    }
    let mut evaluator = Evaluator::new(
        circuit.clone(),
        output_wires,
        wire_to_keys.clone(),
        garbled_gates.clone(),
    );

    // Run the evaluator and return the result
    evaluator.run(vec![alice_input_labels, bob_input_labels]);
}

fn criterion_benchmark(c: &mut Criterion) {
    let file_path = "./circuits/adder64.txt".to_owned();
    let contents = read_to_string(file_path).expect("Couldn't find or load that file.");

    // Move circuit creation outside of the benchmark loop
    let (circuit, ins, outs) = parser::parse_bristol_fashion(&contents);
    let normal_circuit = circuit.clone();

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

    let xor_optimized_circuit = optimize(circuit.clone(), out_keys);

    let number_of_xor_gates = normal_circuit
        .values()
        .map(|(gate, _)| if gate == &GateType::XOR { 1 } else { 0 })
        .sum::<i32>();

    let optim_number_of_xor_gates = xor_optimized_circuit
        .values()
        .map(|(gate, _)| if gate == &GateType::XOR { 1 } else { 0 })
        .sum::<i32>();

    println!("Normal Number of XOR gates: {}", number_of_xor_gates);
    println!("Optim Number of XOR gates: {}", optim_number_of_xor_gates);

    let delta = Fernet::generate_key();
    let delta2 = Fernet::generate_key();

    let mut garbler = Garbler::new(delta, circuit.clone(), ins.clone(), outs.clone());
    let (wire_to_keys, garbled_gates) = garbler.build();

    let mut xor_garbler = Garbler::new(
        delta2,
        xor_optimized_circuit.clone(),
        ins.clone(),
        outs.clone(),
    );
    let (xor_wire_to_keys, xor_garbled_gates) = xor_garbler.build();

    let mut group = c.benchmark_group("CompareAdder64");

    group.bench_function("normal_circuit", |b| {
        b.iter(|| {
            garble_and_evaluate_prepared(&normal_circuit, &ins, &wire_to_keys, &garbled_gates)
        })
    });
    group.bench_function("xor_optimized_circuit", |b| {
        b.iter(|| {
            garble_and_evaluate_prepared(
                &xor_optimized_circuit,
                &ins,
                &xor_wire_to_keys,
                &xor_garbled_gates,
            )
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
