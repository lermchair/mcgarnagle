use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::utils::{Circuit, GateType};

use super::Gate;

#[derive(Debug, Serialize, Deserialize)]
struct YosysPort {
    direction: String,
    bits: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YosysCell {
    #[serde(rename = "type")]
    type_: String,
    port_directions: BTreeMap<String, String>,
    connections: BTreeMap<String, Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YosysModule {
    ports: BTreeMap<String, YosysPort>,
    cells: BTreeMap<String, YosysCell>,
}

#[derive(Debug, Serialize, Deserialize)]
struct YosysJson {
    creator: String,
    modules: BTreeMap<String, YosysModule>,
}

fn handle_port_direction(port_name: &String, bit: &i32, map: &mut BTreeMap<String, Vec<String>>) {
    if map.contains_key(port_name) {
        map.get_mut(port_name).unwrap().push(format!("w_{}", *bit));
    } else {
        map.insert(port_name.clone(), vec![format!("w_{}", *bit)]);
    }
}

fn build_gate(
    cell: &YosysCell,
    inputs: &BTreeMap<String, Vec<String>>,
    outputs: &BTreeMap<String, Vec<String>>,
) -> Gate {
    let mut gate_out = String::new();
    let mut gate_in: Vec<String> = Vec::new();
    let mut merged: BTreeMap<&str, (Option<i32>, Option<&String>)> = BTreeMap::new();
    for (key, value) in cell.connections.iter() {
        merged.entry(key).or_insert((None, None)).0 = Some(value[0]);
    }
    for (key, value) in cell.port_directions.iter() {
        merged.entry(key).or_insert((None, None)).1 = Some(value);
    }

    for (_, (connection, port_dir)) in merged {
        match (connection, port_dir) {
            (Some(connection_value), Some(dir)) if dir == "input" => {
                gate_in.push(
                    inputs
                        .iter()
                        .find(|(_name, bits)| bits.contains(&connection_value.to_string()))
                        .map_or_else(
                            || format!("w_{}", connection_value.to_string()),
                            // |(input_name, _)| format!("{}_{}", input_name, connection_value),
                            |(input_name, _)| format!("w_{}", connection_value),
                            // |(_input_name, _)| connection_value.to_string(),
                        ),
                );
            }
            (Some(connection_value), Some(dir)) if dir == "output" => {
                gate_out = outputs
                    .iter()
                    .find(|(_name, bits)| bits.contains(&connection_value.to_string()))
                    .map_or_else(
                        || format!("w_{}", connection_value.to_string()),
                        // |(output_name, _)| format!("{}_{}", output_name, connection_value),
                        |(output_name, _)| format!("w_{}", connection_value),
                        // |(_, _)| connection_value.to_string(),
                    );
            }
            (None, _) | (_, None) => panic!("Connection or port_dir is None"),
            (_, Some(dir)) => panic!("Unknown port direction: {}", dir),
        }
    }

    Gate {
        inputs: gate_in,
        output: gate_out,
        type_: cell.type_.parse().unwrap(),
    }
}

pub fn parse_yosys_json(
    json_content: &str,
) -> (
    Circuit,
    BTreeMap<String, Vec<String>>,
    BTreeMap<String, Vec<String>>,
) {
    let mut inputs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut circuit: Circuit = BTreeMap::new();
    let mut outputs: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let parsed_circuit: YosysJson = serde_json::from_str(json_content).unwrap();

    for (_module_name, modules) in parsed_circuit.modules.iter() {
        for (port_name, port) in modules.ports.iter() {
            for bit in port.bits.iter() {
                match port.direction.as_str() {
                    "input" => handle_port_direction(port_name, bit, &mut inputs),
                    "output" => handle_port_direction(port_name, bit, &mut outputs),
                    _ => panic!("Unknown port direction: {}", port.direction),
                }
                circuit.insert(
                    // format!("{}_{}", port_name.clone(), bit),
                    format!("w_{}", bit),
                    // bit.to_string(),
                    (GateType::INPUT, vec![]),
                );
            }
        }

        for (_cell_name, cell) in modules.cells.iter() {
            let gate = build_gate(cell, &inputs, &outputs);
            circuit.insert(gate.output, (gate.type_, gate.inputs));
        }
    }
    (circuit, inputs, outputs)
}
