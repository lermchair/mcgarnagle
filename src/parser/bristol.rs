use std::collections::BTreeMap;

use crate::{
    parser::Gate,
    utils::{Circuit, GateType},
};

#[derive(Debug)]
struct BristolHeader {
    total_gates: usize,
    total_wires: usize,
}

pub fn parse_bristol_fashion(
    input: &str,
) -> (
    Circuit,
    BTreeMap<String, Vec<String>>,
    BTreeMap<String, Vec<String>>,
) {
    let mut circuit_inputs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut circuit: Circuit = BTreeMap::new();
    let mut circuit_outputs: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let mut lines = input.lines();

    // Parse header
    let header_line = lines.next().unwrap();
    println!("Header line: {:?}", header_line);
    let _header_parts: Vec<&str> = header_line.split_whitespace().collect();
    let header = BristolHeader {
        total_gates: _header_parts[0].parse::<usize>().unwrap(),
        total_wires: _header_parts[1].parse::<usize>().unwrap(),
    };

    println!("Header: {:?}", header);

    // Parse inputs
    let inputs_line = lines.next().unwrap();
    let inputs: Vec<usize> = inputs_line
        .split_whitespace()
        .skip(1) // The first number is the number of parties
        .map(|n| n.parse::<usize>().unwrap())
        .collect();

    let max_input_id = inputs[0] + inputs[1] - 1;
    let min_input_id = 0;
    println!("Input ID range: {} - {}", min_input_id, max_input_id);

    let a_range = min_input_id..inputs[0];
    let b_range = inputs[0]..max_input_id + 1;

    let a_inputs: Vec<String> = (a_range).map(|n| format!("a_{}", n)).collect();

    let b_inputs: Vec<String> = (b_range).map(|n| format!("b_{}", n)).collect();

    println!(
        "Parsed inputs lengths: {}, {}",
        a_inputs.len(),
        b_inputs.len()
    );

    for input in a_inputs.iter() {
        circuit.insert(input.to_string(), (GateType::INPUT, vec![]));
    }
    for input in b_inputs.iter() {
        circuit.insert(input.to_string(), (GateType::INPUT, vec![]));
    }

    circuit_inputs.insert("a".to_string(), a_inputs);
    circuit_inputs.insert("b".to_string(), b_inputs);

    println!("Circuit inputs: {:?}", circuit_inputs);

    let outputs_line = lines.next().unwrap();
    let outputs: Vec<usize> = outputs_line
        .split_whitespace()
        .skip(1)
        .map(|n| n.parse::<usize>().unwrap())
        .collect();

    let max_output_id = header.total_wires - 1;
    let min_output_id = header.total_wires - outputs[0] - 1;
    println!("Output ID range: {} - {}", min_output_id, max_output_id);
    assert!(max_output_id - min_output_id == outputs[0]);

    let output_wires: Vec<String> = (min_output_id + 1..max_output_id + 1)
        .map(|n| format!("out_{}", n))
        .collect();

    println!("Circuit outputs: {:?}", output_wires);
    println!("Number of outputs: {:?} -> {}", outputs, output_wires.len());
    assert!(output_wires.len() == outputs[0]);

    circuit_outputs.insert("out".to_string(), output_wires);

    lines.next().unwrap();
    let mut gates: Vec<Gate> = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let gate_type = match parts.last().unwrap() {
            &"AND" => GateType::AND,
            &"OR" => GateType::OR,
            &"XOR" => GateType::XOR,
            &"INV" => GateType::NOT,
            _ => panic!("Unknown gate type"),
        };

        let input_wires: Vec<String> = parts[2..parts.len() - 2]
            .iter()
            .map(|&n| n.parse::<usize>().unwrap().to_string())
            .collect();

        let mut new_input_wires: Vec<String> = Vec::new();
        for input_wire in &input_wires {
            let input_parsed = input_wire.parse::<usize>().unwrap();
            let in_input_range = input_parsed >= min_input_id && input_parsed <= max_input_id;
            let in_output_range = input_parsed >= min_output_id && input_parsed <= max_output_id;
            if in_input_range {
                let in_b_range = input_parsed >= inputs[0] && input_parsed <= max_input_id + 1;
                let in_a_range = input_parsed >= min_input_id && input_parsed < inputs[0];
                if in_b_range {
                    let new_input_wire = format!("b_{}", input_wire);
                    new_input_wires.push(new_input_wire);
                } else if in_a_range {
                    let new_input_wire = format!("a_{}", input_wire);
                    new_input_wires.push(new_input_wire);
                } else {
                    panic!("Input wire not in a or b range: {}", input_wire);
                }
            } else {
                if in_output_range {
                    let new_input_wire = format!("out_{}", input_wire);
                    new_input_wires.push(new_input_wire);
                } else {
                    new_input_wires.push(input_wire.to_string());
                }
            }
        }

        let output_wire = parts[parts.len() - 2].parse::<usize>().unwrap();
        let in_output_range = output_wire >= min_output_id && output_wire <= max_output_id;
        if in_output_range {
            let new_output_wire = format!("out_{}", output_wire);
            gates.push(Gate {
                inputs: new_input_wires,
                output: new_output_wire,
                type_: gate_type,
            });
        } else {
            gates.push(Gate {
                inputs: new_input_wires,
                output: output_wire.to_string(),
                type_: gate_type,
            });
        }
    }

    for gate in gates {
        if gate.inputs.len() > 2 {
            panic!("Gate has more than 2 inputs {:?}", gate);
        }
        circuit.insert(gate.output, (gate.type_, gate.inputs));
    }

    (circuit, circuit_inputs, circuit_outputs)
}
