use egg::{rewrite as rw, *};
use std::collections::HashSet;

define_language! {
    pub enum Semiring {
        Num(i32),
        "lit" = Lit(Id),
        "var" = Var(Id),

        "rel" = Rel(Box<[Id]>),

        "+" = Add([Id; 2]),
        "-" = Min([Id; 2]),
        "*" = Mul([Id; 2]),

        "sum" = Sum([Id; 2]),
        "let" = Let([Id; 3]),

        "I" = Ind(Id),
        "<" = Lt([Id; 2]),
        "<=" = Leq([Id; 2]),

        ">" = Gt([Id; 2]),
        ">=" = Geq([Id; 2]),

        "=" = Eq([Id; 2]),

        Symbol(egg::Symbol),
    }
}

pub type EGraph = egg::EGraph<Semiring, BindAnalysis>;

#[derive(Default, Clone)]
pub struct BindAnalysis {
    pub found: bool,
}

#[derive(Debug)]
pub struct Data {
    free: HashSet<Id>,
    pub found: bool,
}

impl Analysis<Semiring> for BindAnalysis {
    type Data = Data;
    fn merge(&self, to: &mut Data, from: Data) -> bool {
        let before_len = to.free.len();
        to.free.retain(|i| from.free.contains(i));
        to.found = from.found || to.found;
        before_len != to.free.len()
    }

    fn make(egraph: &EGraph, enode: &Semiring) -> Data {
        let f = |i: &Id| egraph[*i].data.free.iter().cloned();
        let mut free = HashSet::default();
        match enode {
            Semiring::Var(v) => {
                free.insert(*v);
            }
            Semiring::Let([v, a, b]) => {
                free.extend(f(b));
                free.remove(v);
                free.extend(f(a));
            }
            Semiring::Sum([v, a]) => {
                free.extend(f(a));
                free.remove(v);
            }
            _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
        }
        Data {
            free,
            found: egraph.analysis.found,
        }
    }

    fn modify(_egraph: &mut EGraph, _id: Id) {}
}

pub struct Found {
    msg: &'static str,
}

pub struct CaptureAvoid {
    fresh: Var,
    v2: Var,
    e: Var,
    if_not_free: Pattern<Semiring>,
    if_free: Pattern<Semiring>,
}

impl Applier<Semiring, BindAnalysis> for CaptureAvoid {
    fn apply_one(&self, egraph: &mut EGraph, eclass: Id, subst: &Subst) -> Vec<Id> {
        let e = subst[self.e];
        let v2 = subst[self.v2];
        let v2_free_in_e = egraph[e].data.free.contains(&v2);
        if v2_free_in_e {
            let mut subst = subst.clone();
            let sym = Semiring::Symbol(format!("_{}", eclass).into());
            subst.insert(self.fresh, egraph.add(sym));
            self.if_free.apply_one(egraph, eclass, &subst)
        } else {
            self.if_not_free.apply_one(egraph, eclass, &subst)
        }
    }
}

pub struct Destroy<A: Applier<Semiring, BindAnalysis>> {
    e: A,
}

pub struct RenameSum {
    fresh: Var,
    e: Pattern<Semiring>,
}

impl<A: Applier<Semiring, BindAnalysis>> Applier<Semiring, BindAnalysis> for Destroy<A> {
    fn apply_one(&self, egraph: &mut EGraph, eclass: Id, subst: &Subst) -> Vec<Id> {
        egraph[eclass].nodes.clear();
        self.e.apply_one(egraph, eclass, subst)
    }
}

impl Applier<Semiring, BindAnalysis> for Found {
    fn apply_one(&self, _egraph: &mut EGraph, _eclass: Id, _subst: &Subst) -> Vec<Id> {
        panic!("Found {}", self.msg)
    }
}

impl Applier<Semiring, BindAnalysis> for RenameSum {
    fn apply_one(&self, egraph: &mut EGraph, eclass: Id, subst: &Subst) -> Vec<Id> {
        let mut subst = subst.clone();
        let sym = Semiring::Symbol(format!("_{}", eclass).into());
        subst.insert(self.fresh, egraph.add(sym));
        self.e.apply_one(egraph, eclass, &subst)
    }
}

fn var(s: &str) -> Var {
    s.parse().unwrap()
}

fn is_not_same_var(v1: Var, v2: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph.find(subst[v1]) != egraph.find(subst[v2])
}

fn not_free(x: Var, b: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let f = free(x, b);
    move |egraph, id, subst| !f(egraph, id, subst)
}

fn free(x: Var, b: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    // NOTE might want to call `find`
    move |egraph, _, subst| egraph[(subst[b])].data.free.contains(&subst[x])
}

pub fn rules() -> Vec<Rewrite<Semiring, BindAnalysis>> {
    let mut rs = vec![
        rw!("let-lit"; "(let ?v1 ?e (lit ?n))" => "(lit ?n)"),
        rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
        rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw!("swap-sum"; "(sum ?x (sum ?y ?e))" => "(sum ?y (sum ?x ?e))"),
        // NOTE can be generating a bunch of stuff here
        rw!("pushdown-sum-free";
            "(* ?b (sum ?x ?a))" =>
            { RenameSum {
                fresh: var("?fresh"),
                e: "(sum ?fresh (* ?b (let ?x ?fresh ?a)))".parse().unwrap()
            }}
            if free(var("?x"), var("?b"))),
        rw!("let-sum-same"; "(let ?v1 ?e (sum ?v1 ?body))" => "(sum ?v1 ?body)"),
        rw!("let-sum-diff";
            "(let ?v1 ?e (sum ?v2 ?body))" =>
            { CaptureAvoid {
                fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
                if_not_free: "(sum ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                if_free: "(sum ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
            }}
            if is_not_same_var(var("?v1"), var("?v2"))),
    ];
    rs.extend(vec![
        // subst rules
        rw!("let-add";  "(let ?v ?e (+   ?a ?b))" <=> "(+   (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-eq";   "(let ?v ?e (=   ?a ?b))" <=> "(=   (let ?v ?e ?a) (let ?v ?e ?b))"),
        // open term rules
        rw!("add-comm";  "(+ ?a ?b)"        <=> "(+ ?b ?a)"),
        rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" <=> "(+ ?a (+ ?b ?c))"),
        rw!("mul-comm";  "(* ?a ?b)"        <=> "(* ?b ?a)"),
        rw!("mul-assoc"; "(* (* ?a ?b) ?c)" <=> "(* ?a (* ?b ?c))"),
        rw!("subtract";  "(- ?a ?b)" <=> "(+ ?a (* (lit -1) ?b))"),
        rw!("eq-comm";   "(= ?a ?b)"        <=> "(= ?b ?a)"),
        rw!("add-mul-dist"; "(* (+ ?a ?b) ?c)" <=> "(+ (* ?a ?c) (* ?b ?c))"),
        rw!("add-sum-dist"; "(sum (var ?x) (+ ?a ?b))" <=> "(+ (sum (var ?x) ?a) (sum (var ?x) ?b))"),
        rw!("pushdown-sum-bound"; "(* ?b (sum ?x ?a))" <=> "(sum ?x (* ?b ?a))" if not_free(var("?x"), var("?b"))),

    ].concat());
    rs
}

// one way destructive rewrite
fn rw_1(
    name: &'static str,
    lhs: &'static str,
    rhs: &'static str,
) -> Rewrite<Semiring, BindAnalysis> {
    Rewrite::new(
        name,
        lhs.parse::<Pattern<Semiring>>().unwrap(),
        Destroy {
            e: rhs.parse::<Pattern<Semiring>>().unwrap(),
        },
    )
    .unwrap()
}

pub fn normalize() -> Vec<Rewrite<Semiring, BindAnalysis>> {
    vec![
        rw_1(
            "pushdown-mul",
            "(* ?a (+ ?b ?c))",
            "(+ (* ?a ?b) (* ?a ?c))",
        ),
        rw_1(
            "pushdown-sum-add",
            "(sum ?i (+ ?a ?b))",
            "(+ (sum ?i ?a) (sum ?i ?b))",
        ),
        rw!("pushdown-sum-bound"; "(* ?b (sum ?x ?a))" => {
            Destroy { e: "(sum ?x (* ?b ?a))".parse::<Pattern<Semiring>>().unwrap() }
        } if not_free(var("?x"), var("?b"))),
        rw!("pushdown-sum-free";
            "(* ?b (sum ?x ?a))" =>
            { Destroy { e: RenameSum {
                fresh: var("?fresh"),
                e: "(sum ?fresh (* ?b (let ?x ?fresh ?a)))".parse().unwrap()
            }}}
            if free(var("?x"), var("?b"))),
        rw_1("let-lit", "(let ?v1 ?e (lit ?n))", "(lit ?n)"),
        rw_1("let-var-same", "(let ?v1 ?e (var ?v1))", "?e"),
        rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" =>
            { Destroy { e: "(var ?v2)".parse::<Pattern<Semiring>>().unwrap() }}
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw_1(
            "let-sum-same",
            "(let ?v1 ?e (sum ?v1 ?body))",
            "(sum ?v1 ?body)",
        ),
        rw!("let-sum-diff";
            "(let ?v1 ?e (sum ?v2 ?body))" =>
            { Destroy { e: { CaptureAvoid {
                fresh: var("?fresh"), v2: var("?v2"), e: var("?e"),
                if_not_free: "(sum ?v2 (let ?v1 ?e ?body))".parse().unwrap(),
                if_free: "(sum ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
            }}}}
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw_1(
            "let-add",
            "(let ?v ?e (+ ?a ?b))",
            "(+ (let ?v ?e ?a) (let ?v ?e ?b))",
        ),
        rw_1(
            "let-eq",
            "(let ?v ?e (= ?a ?b))",
            "(= (let ?v ?e ?a) (let ?v ?e ?b))",
        ),
        rw_1("subtract", "(- ?a ?b)", "(+ ?a (* (lit -1) ?b))"),
    ]
}
