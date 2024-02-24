use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    rc::Rc,
    sync::Mutex,
};

use egg::{Extractor, Id, RecExpr, Runner};

use crate::{
    optimizer::CircuitLang,
    utils::{Circuit, GateType},
};

use super::{circuit_rules, GarbleCost};

#[derive(Debug)]
enum GateOutputStructure {
    Gate(GateType, Vec<Rc<GateMapping>>),
}

#[derive(Debug)]
struct GateMapping {
    mapping: BTreeMap<String, Option<Rc<Box<GateOutputStructure>>>>,
}

pub fn optimize(circuit: Circuit, outputs: Vec<String>) -> Circuit {
    let mut output_to_expr: HashMap<String, RecExpr<CircuitLang>> = HashMap::new();

    println!("Traversing outputs...");

    let memo = Mutex::new(HashMap::new());

    for wire_name in outputs.iter() {
        println!("Traversing wire: {}", wire_name);
        let circuit_structure = traverse_wire(wire_name, &circuit, &memo);
        println!("Generating expression...");
        // println!("Building for: {} -> {:?}", wire_name, circuit_structure);
        let expr = gates_to_expr(&circuit_structure);
        println!("{} -> {}", wire_name, expr.to_string());
        output_to_expr.insert(wire_name.clone(), expr);
    }

    if output_to_expr.len() != outputs.len() {
        panic!("Mismatch between number of outputs and number of gates");
    }

    // println!("Output to expr: {:?}", output_to_expr.keys());

    let mut new_circuit: Circuit = Circuit::new();
    let mut wire_counter: usize = 0;

    let mut existing_gates: HashMap<String, String> = HashMap::new();

    println!("Simplifying expressions...");
    for (output_name, expr) in output_to_expr.iter() {
        let simplified = simplify(expr);
        build_circuit(
            simplified,
            &mut new_circuit,
            &mut existing_gates,
            &mut wire_counter,
            output_name.to_string(),
        );
    }
    println!("Orig circuit len: {}", circuit.keys().len());
    println!("New circuit len: {}", new_circuit.keys().len());
    new_circuit

    // TODO: if no optimizations found, we just return the original circuit
}

fn add_or_reuse_gate(
    circuit: &mut Circuit,
    gate_type: GateType,
    inputs: &Vec<String>,
    existing_gates: &mut HashMap<String, String>,
    wire_counter: &mut usize,
    output_wire: &Option<String>,
) -> String {
    if let Some(output_wire) = output_wire {
        circuit.insert(output_wire.clone(), (gate_type, inputs.to_vec()));
        return output_wire.to_string();
    }

    let gate_key = format!("{}_{:?}", gate_type, inputs);

    if let Some(output_wire) = existing_gates.get(&gate_key) {
        return output_wire.clone();
    }
    let output_wire = format!("i_{}", wire_counter.to_string());
    *wire_counter += 1;
    circuit.insert(output_wire.clone(), (gate_type, inputs.to_vec()));
    existing_gates.insert(gate_key, output_wire.clone());
    output_wire
}

fn build_circuit(
    expr: RecExpr<CircuitLang>,
    global_circuit: &mut Circuit,
    existing_gates: &mut HashMap<String, String>,
    wire_counter: &mut usize,
    output_name: String,
) {
    let rexpr = expr.as_ref();
    let mut stack: Vec<String> = vec![];
    for e in rexpr.iter() {
        let is_last = rexpr.last() == Some(e);
        let output_wire = if is_last {
            Some(output_name.clone())
        } else {
            None
        };
        match e {
            CircuitLang::Const0 => {
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::CONST_0,
                    &vec![],
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Const1 => {
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::CONST_1,
                    &vec![],
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::And(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::AND,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Or(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::OR,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Not(id) => {
                let mut inputs: Vec<String> = vec![];
                let wire = &stack[usize::from(*id)];
                inputs.push(wire.to_string());
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::NOT,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Xor(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::XOR,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::OrNot(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::ORNOT,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Nor(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::NOR,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Nand(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::NAND,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::AndNot(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::ANDNOT,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }
            CircuitLang::Xnor(ids) => {
                let mut inputs: Vec<String> = vec![];
                for id in ids.iter() {
                    let wire = &stack[usize::from(*id)];
                    inputs.push(wire.to_string());
                }
                let output_wire = add_or_reuse_gate(
                    global_circuit,
                    GateType::XNOR,
                    &inputs,
                    existing_gates,
                    wire_counter,
                    &output_wire,
                );
                stack.push(output_wire);
            }

            CircuitLang::Wire(wire) => {
                if let Some(output_wire) = output_wire {
                    global_circuit.insert(
                        output_wire.clone(),
                        (GateType::CONST, vec![wire.to_string()]),
                    );
                    return;
                }
                global_circuit
                    .entry(wire.to_string())
                    .or_insert((GateType::INPUT, Vec::new()));
                stack.push(wire.to_string());
            }
        }
    }
}

fn traverse_wire(
    output_wire: &str,
    circuit: &Circuit,
    memo: &Mutex<HashMap<String, Rc<GateMapping>>>,
) -> Rc<GateMapping> {
    let memo_lock = memo.lock().unwrap();
    if let Some(result) = memo_lock.get(output_wire) {
        return result.clone();
    }
    drop(memo_lock);

    let gate = circuit.get(output_wire);
    let result = match gate {
        None => {
            let mut mapping = BTreeMap::new();
            mapping.insert(output_wire.to_string(), None);
            Rc::new(GateMapping { mapping })
        }
        Some((gate_type, input_wires)) => {
            let mut input_structures: Vec<Rc<GateMapping>> = Vec::new();
            for input_wire in input_wires {
                let input_structure = traverse_wire(input_wire, circuit, memo);
                input_structures.push(input_structure);
            }
            let mut mapping = BTreeMap::new();
            mapping.insert(
                output_wire.to_string(),
                Some(Rc::new(Box::new(GateOutputStructure::Gate(
                    gate_type.clone(),
                    input_structures,
                )))),
            );
            Rc::new(GateMapping { mapping })
        }
    };

    let mut memo_lock = memo.lock().unwrap();
    memo_lock.insert(output_wire.to_string(), result.clone());
    result
}

fn gates_to_expr(gate: &GateMapping) -> RecExpr<CircuitLang> {
    let mut expr = RecExpr::default();
    let mut cache = HashMap::new(); // Cache for memoization
    build_expr(&gate, &mut expr, &mut cache);
    expr
}

fn build_expr(
    gate_mapping: &GateMapping,
    expr: &mut RecExpr<CircuitLang>,
    cache: &mut HashMap<String, Id>,
) -> Id {
    gate_mapping
        .mapping
        .iter()
        .map(|(gate_id, gate_info)| {
            if let Some(&cached_id) = cache.get(gate_id) {
                return cached_id;
            }
            let result_id = match gate_info {
                None => expr.add(CircuitLang::Wire(gate_id.clone())),
                Some(gate_structure) => {
                    let gos = gate_structure.as_ref();
                    let GateOutputStructure::Gate(gate_type, inputs) = gos.as_ref();
                    let input_exprs: Vec<Id> = inputs
                        .iter()
                        .map(|input_gate| build_expr(input_gate, expr, cache))
                        .collect();

                    match gate_type {
                        GateType::INPUT => expr.add(CircuitLang::Wire(gate_id.clone())),
                        GateType::CONST => expr.add(CircuitLang::Wire(gate_id.clone())),
                        GateType::CONST_0 => expr.add(CircuitLang::Const0),
                        GateType::CONST_1 => expr.add(CircuitLang::Const1),
                        GateType::AND
                        | GateType::OR
                        | GateType::XOR
                        | GateType::NOR
                        | GateType::ORNOT
                        | GateType::NAND
                        | GateType::ANDNOT
                        | GateType::XNOR => {
                            let ids: [Id; 2] = input_exprs
                                .try_into()
                                .expect("Expected two inputs for binary gate");
                            match gate_type {
                                GateType::AND => expr.add(CircuitLang::And(ids)),
                                GateType::OR => expr.add(CircuitLang::Or(ids)),
                                GateType::XOR => expr.add(CircuitLang::Xor(ids)),
                                GateType::NOR => expr.add(CircuitLang::Nor(ids)),
                                GateType::ORNOT => expr.add(CircuitLang::OrNot(ids)),
                                GateType::NAND => expr.add(CircuitLang::Nand(ids)),
                                GateType::ANDNOT => expr.add(CircuitLang::AndNot(ids)),
                                GateType::XNOR => expr.add(CircuitLang::Xnor(ids)),
                                _ => unreachable!("Handled all binary gate types"),
                            }
                        }
                        GateType::NOT => {
                            let id = *input_exprs
                                .first()
                                .expect("Expected one input for unary gate");
                            expr.add(CircuitLang::Not(id))
                        }
                        _ => unreachable!("All gate types handled"),
                    }
                }
            };
            cache.insert(gate_id.clone(), result_id);
            result_id
        })
        .next()
        .unwrap_or_else(|| panic!("Invalid gate mapping encountered."))
}

fn simplify(expr: &RecExpr<CircuitLang>) -> RecExpr<CircuitLang> {
    let runner = Runner::default().with_expr(&expr).run(&circuit_rules());
    let root = runner.roots[0];
    let extractor = Extractor::new(&runner.egraph, GarbleCost);
    let (best_cost, best) = extractor.find_best(root);
    if best.to_string() != *expr.to_string() {
        println!("Simplified to cost {}", best_cost);
    }
    best
}
