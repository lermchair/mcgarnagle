mod optim;

pub use self::optim::*;
use egg::{define_language, rewrite, CostFunction, Id, Language, Rewrite};

define_language! {
    enum CircuitLang {
        "and" = And([Id; 2]),
        "not" = Not(Id),
        "or" = Or([Id; 2]),
        "xor" = Xor([Id; 2]),
        "ornot" = OrNot([Id; 2]),
        "nor" = Nor([Id; 2]),
        "nand" = Nand([Id; 2]),
        "andnot" = AndNot([Id; 2]),
        "xnor" = Xnor([Id; 2]),
        Wire(String),
    }
}

fn circuit_rules() -> Vec<Rewrite<CircuitLang, ()>> {
    vec![
        rewrite!("simplify-and"; "(and ?a ?a)" => "?a"),
        rewrite!("simplify-or"; "(or (and ?a (not ?b)) (and ?b (not ?a)))" => "(xor ?a ?b)"),
        rewrite!("double-negation-xor"; "(xor (not (not ?a)) ?b)" => "(xor ?a ?b)"),
        rewrite!("optimize-xor-combinations"; "(xor (xor ?a ?b) ?b)" => "?a"),
        rewrite!("eliminate-redundant-xor"; "(xor ?a (xor ?a ?b))" => "?b"),
        rewrite!("nand-to-not"; "(nand ?a ?a)" => "(not ?a)"),
        rewrite!("nand-to-and"; "(nand (nand ?a ?b) (nand ?a ?b))" => "(and ?a ?b)"),
        rewrite!("nand-to-or"; "(nand (nand ?a ?a) (nand ?b ?b))" => "(or ?a ?b)"),
        rewrite!("de-morgan-nand-to-or"; "(nand ?a ?b)" => "(not (and ?a ?b))"),
        //
        rewrite! {"double_negative";     "(not (not ?a))"      => "?a"},
        //
        rewrite! {"idempotent_and";      "(and ?a ?a)"         => "?a"},
        rewrite! {"idempotent_or";       "(or ?a ?a)"          => "?a"},
        //
        rewrite! {"absorption_and";      "(and ?a (or ?a ?b))" => "?a"},
        rewrite! {"absorption_or";       "(or ?a (and ?a ?b))" => "?a"},
        rewrite!("or-xor-1"; "(or ?a (xor ?a ?b))" => "(or ?a ?b)"),
        rewrite!("xor_conv"; "(and (or ?a ?b) (not (and ?a ?b)))" => "(xor ?a ?b)"),
        rewrite!("xor_conv2"; "(or (and (not ?a) ?b) (and ?a (not ?b)))" => "(xor ?a ?b)"),
        rewrite!("double_negation_to_xor"; "(not (not (xor ?a ?b)))" => "(xor ?a ?b)"),
        rewrite!("xor-with-negation"; "(xor ?a (not ?b))" => "(not (xor ?a ?b))"),
        rewrite!("nor-to-not-or"; "(nor ?a ?b)" => "(not (or ?a ?b))"),
        rewrite!("simplify-nor"; "(nor ?a ?a)" => "(not ?a)"),
        rewrite!("xnor-to-and-or"; "(xnor ?a ?b)" => "(or (and ?a ?b) (and (not ?a) (not ?b)))"),
        // rewrite!("simplify-nested-not-and"; "(and (not (not ?a)) ?b)" => "(and ?a ?b)"),
        // rewrite!("simplify-nested-not-or"; "(or (not (not ?a)) ?b)" => "(or ?a ?b)"),
        rewrite!("merge-consecutive-xor"; "(xor (xor ?a ?b) ?c)" => "(xor ?a (xor ?b ?c))"),
        rewrite!("eliminate-double-negation-xor"; "(xor (not (not ?a)) ?b)" => "(xor ?a ?b)"),
        rewrite!("nand-simplify-pattern"; "(nand (nand ?a ?b) ?a)" => "(nand ?a ?b)"),
        rewrite!("nor-simplify-pattern"; "(nor (nor ?a ?b) ?a)" => "(nor ?a ?b)"),
    ]
}

struct GarbleCost;
impl CostFunction<CircuitLang> for GarbleCost {
    type Cost = f64;
    fn cost<C>(&mut self, enode: &CircuitLang, mut costs: C) -> Self::Cost
    where
        C: FnMut(Id) -> Self::Cost,
    {
        // Cost function is AST size weighted by the cost of each operator
        let op_cost = match enode {
            CircuitLang::And(_) => 4.0,
            CircuitLang::Or(_) => 4.0,
            CircuitLang::Not(_) => 2.0,
            CircuitLang::Xor(_) => 1.0,
            CircuitLang::OrNot(_) => 4.0,
            CircuitLang::Nor(_) => 4.0,
            CircuitLang::Nand(_) => 4.0,
            CircuitLang::AndNot(_) => 4.0,
            CircuitLang::Xnor(_) => 4.0,
            _ => 4.0,
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}
