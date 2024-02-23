use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Display, Formatter},
    str::FromStr,
};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use fernet::Fernet;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub enum GateType {
    AND,
    NOT,
    OR,
    XOR,
    ORNOT,
    NOR,
    NAND,
    ANDNOT,
    XNOR,
    CONST_1,
    CONST_0,
    CONST,
    INPUT,
}

impl Display for GateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GateType::AND => write!(f, "AND"),
            GateType::NOT => write!(f, "NOT"),
            GateType::OR => write!(f, "OR"),
            GateType::XOR => write!(f, "XOR"),
            GateType::ORNOT => write!(f, "ORNOT"),
            GateType::NOR => write!(f, "NOR"),
            GateType::NAND => write!(f, "NAND"),
            GateType::ANDNOT => write!(f, "ANDNOT"),
            GateType::XNOR => write!(f, "XNOR"),
            GateType::CONST_1 => write!(f, "CONST_1"),
            GateType::CONST_0 => write!(f, "CONST_0"),
            GateType::CONST => write!(f, "CONST"),
            GateType::INPUT => write!(f, "INPUT"),
        }
    }
}

pub type Circuit = BTreeMap<String, (GateType, Vec<String>)>;
type HmacSha256 = Hmac<Sha256>;

impl FromStr for GateType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "$_AND_" => Ok(GateType::AND),
            "$_OR_" => Ok(GateType::OR),
            "$_NOT_" => Ok(GateType::NOT),
            "$_XOR_" => Ok(GateType::XOR),
            "$_NOR_" => Ok(GateType::NOR),
            "$_NAND_" => Ok(GateType::NAND),
            "$_ANDNOT_" => Ok(GateType::ANDNOT),
            "$_ORNOT_" => Ok(GateType::ORNOT),
            "$_XNOR_" => Ok(GateType::XNOR),
            _ => Err(()),
        }
    }
}

pub fn generate_keys(delta: &String) -> (String, String) {
    let zero = Fernet::generate_key();
    let y = URL_SAFE.decode(zero.clone()).unwrap();
    let z = URL_SAFE.decode(delta).unwrap();
    let xored = bytes_xor(&y, &z);
    let xored_str = URL_SAFE.encode(&xored);
    (zero, xored_str)
}

pub fn generate_encryption_key(keys: &[&[u8]]) -> Vec<u8> {
    if keys.is_empty() {
        panic!("At least one key must be provided");
    }

    let mut hmac = HmacSha256::new_from_slice(keys[0]).expect("HMAC can take key of any size");
    for &key in &keys[1..] {
        hmac.update(key);
    }
    hmac.finalize().into_bytes().to_vec()
}

// Expects a base-64 encoded string for the key
// Returns another base-64 encoded string
pub fn encrypt(key: &String, data: Vec<u8>) -> String {
    let fernet = Fernet::new(key);
    match fernet {
        Some(f) => f.encrypt(&data),
        None => panic!("Error creating fernet, check key: {}", key),
    }
}

// expects a base-64 encoded string for the key and data
pub fn decrypt(key: String, data: String) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let fernet = Fernet::new(&key).ok_or("Invalid Fernet key")?;
    fernet.decrypt(&data).map_err(Into::into)
}

pub fn bytes_xor(a: &Vec<u8>, b: &Vec<u8>) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

pub fn topo_sort_wires(circuit: &Circuit) -> Vec<String> {
    let mut sorted = Vec::new();
    let mut visited = HashSet::new();
    let mut temp_mark = HashSet::new();

    fn visit(
        node: &str,
        circuit: &Circuit,
        visited: &mut HashSet<String>,
        temp_mark: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> Result<(), &'static str> {
        if temp_mark.contains(node) {
            return Err("Circuit has a cycle");
        }
        if !visited.contains(node) {
            temp_mark.insert(node.to_string());
            if let Some((_, inputs)) = circuit.get(node) {
                for input in inputs {
                    visit(input, circuit, visited, temp_mark, sorted)?;
                }
            }
            temp_mark.remove(node);
            visited.insert(node.to_string());
            sorted.push(node.to_string());
        }
        Ok(())
    }

    for node in circuit.keys() {
        if let Err(e) = visit(node, circuit, &mut visited, &mut temp_mark, &mut sorted) {
            panic!("{}", e);
        }
    }
    sorted
}

pub fn wire_values(input_keys: &Vec<String>, value: i32) -> BTreeMap<String, i32> {
    let bits = format!("{:b}", value).chars().rev().collect::<String>();
    let bits_padded = format!("{:0<width$}", bits, width = input_keys.len());
    input_keys
        .into_iter()
        .zip(bits_padded.chars())
        .map(|(key, bit)| (key.to_string(), bit.to_digit(10).unwrap() as i32))
        .collect()
}
