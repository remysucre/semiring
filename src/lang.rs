use egg::*;

define_language! {
    pub enum Semiring {
        // A bare number is a literal
        Num(i32),

        // All variables are tagged with var
        // to distinguish from relations
        // e.g. (var x)
        "var" = Var(Id),

        // Relations are tagged with rel
        // e.g. (rel R (var x) (var y))
        "rel" = Rel(Box<[Id]>),

        "+" = Add([Id; 2]),
        "-" = Min([Id; 2]),
        "*" = Mul([Id; 2]),

        // NOTE Remember the `var` tag for the
        // aggregated variable, e.g. (sum (var i) ...)
        "sum" = Sum([Id; 2]),

        // NOTE Remember the `var` tag for variables,
        // e.g. let (var v1) (var v2) = e: e[v1 |-> v2]
        "let" = Let([Id; 3]),

        // Indicator, i.e. (I true) = 1, (I false) = 0
        "I" = Ind(Id),

        "<" = Lt([Id; 2]),
        "<=" = Leq([Id; 2]),
        ">" = Gt([Id; 2]),
        ">=" = Geq([Id; 2]),
        "=" = Eq([Id; 2]),

        Symbol(egg::Symbol),

        // Fallback to arbitrary "UDF"
        Other(Symbol, Vec<Id>),
    }
}

// Extract literal numbers if any
impl Semiring {
    pub fn num(&self) -> Option<i32> {
        match self {
            Semiring::Num(n) => Some(*n),
            _ => None,
        }
    }
}
