use egg::*;

// REVIEW
define_language! {
    pub enum Semiring {
        // a bare number is a literal
        Num(i32),
        // all variables are tagged with var
        // to distinguish from relations
        "var" = Var(Id),

        // relations are tagged with rel
        "rel" = Rel(Box<[Id]>),

        "+" = Add([Id; 2]),
        "-" = Min([Id; 2]),
        "*" = Mul([Id; 2]),

        // NOTE the "var" in (sum (var i) ...)
        "sum" = Sum([Id; 2]),

        // NOTE the "var" in let (var x) (var y) ...
        // let (var v1) (var v2) e: e[v1 |-> v2]
        "let" = Let([Id; 3]),

        // indicator, i.e. (I true) = 1, (I false) = 0
        "I" = Ind(Id),
        "<" = Lt([Id; 2]),
        "<=" = Leq([Id; 2]),

        ">" = Gt([Id; 2]),
        ">=" = Geq([Id; 2]),

        "=" = Eq([Id; 2]),

        Symbol(egg::Symbol),

        // fallback to arbitrary "UDF"
        Other(Symbol, Vec<Id>),
    }
}

// extract literal numbers if any
impl Semiring {
    pub fn num(&self) -> Option<i32> {
        match self {
            Semiring::Num(n) => Some(*n),
            _ => None,
        }
    }
}
