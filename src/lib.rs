use egg::{rewrite as rw, *};
use rand::prelude::*;
use std::collections::{HashSet, HashMap};

const FP_LEN: usize = 16;

// REVIEW
define_language! {
    pub enum Semiring {
        Num(i32),
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

        Other(Symbol, Vec<Id>),
    }
}

// REVIEW
impl Semiring {
    fn num(&self) -> Option<i32> {
        match self {
            Semiring::Num(n) => Some(*n),
            _ => None,
        }
    }
}

pub type EGraph = egg::EGraph<Semiring, BindAnalysis>;

#[derive(Default, Clone)]
pub struct BindAnalysis {
    pub found: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Data {
    free: HashSet<Id>,
    pub found: bool,
    constant: Option<Semiring>,
    fingerprint: Option<Vec<i32>>,
}

// REVIEW
impl Analysis<Semiring> for BindAnalysis {
    type Data = Data;
    fn merge(&self, to: &mut Data, from: Data) -> bool {
        if *to == from {
            false
        } else {
            // FIXME this might be wrong
            to.free.retain(|i| from.free.contains(&*i));
            if from.constant.is_some() {
                to.constant = from.constant;
            }
            if from.fingerprint.is_some() {
                to.fingerprint = from.fingerprint;
            }
            to.found = from.found || to.found;
            true
        }
    }

    // REVIEW
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
        let constant = eval(egraph, enode);
        let fingerprint = fingerprint(egraph, enode);
        Data {
            free,
            found: egraph.analysis.found,
            constant,
            fingerprint
        }
    }

    // REVIEW
    fn modify(egraph: &mut EGraph, id: Id)
    {
        if let Some(c) = egraph[id].data.constant.clone() {
            let const_id = egraph.add(c);
            egraph.union(id, const_id);
        }
    }
}

// REVIEW
fn combine_fp<F>(x: &Option<Vec<i32>>, y: &Option<Vec<i32>>, f: F) -> Option<Vec<i32>>
    where F: Fn((&i32, &i32))-> i32
{
    if let (Some(v_x), Some(v_y)) = (x, y) {
            Some(v_x.iter().zip(v_y.iter()).map(f).collect())
    } else { None }
}

// REVIEW
fn fingerprint(egraph: &EGraph, enode: &Semiring) -> Option<Vec<i32>> {
    let f = |i: &Id| &egraph[*i].data.fingerprint;
    match enode {
        Semiring::Var(v) => Some({
            println!("{:?}", v);
            (0..FP_LEN).map(|_| thread_rng().gen()).collect()
        }),
        Semiring::Num(n) => Some({
            (0..FP_LEN).map(|_| *n).collect()
        }),
        Semiring::Add([a, b]) => combine_fp(f(a), f(b), |(x,y)| x + y),
        Semiring::Min([a, b]) => combine_fp(f(a), f(b), |(x,y)| x - y),
        Semiring::Mul([a, b]) => combine_fp(f(a), f(b), |(x,y)| x * y),
        Semiring::Ind(b) => f(b).clone(),
        Semiring::Lt([a, b]) => combine_fp(f(a), f(b), |(x,y)| if x < y {1} else {0}),
        Semiring::Leq([a, b]) => combine_fp(f(a), f(b), |(x,y)| if x <= y {1} else {0}),
        Semiring::Eq([a, b]) => combine_fp(f(a), f(b), |(x,y)| if x == y {1} else {0}),
        Semiring::Gt([a, b]) => combine_fp(f(a), f(b), |(x,y)| if x > y {1} else {0}),
        Semiring::Geq([a, b]) => combine_fp(f(a), f(b), |(x,y)| if x >= y {1} else {0}),
        _ => None,
    }
}

// REVIEW
fn eval(egraph: &EGraph, enode: &Semiring) -> Option<Semiring> {
    let x = |i: &Id| egraph[*i].data.constant.clone();
    match enode {
        Semiring::Num(_) => Some(enode.clone()),
        Semiring::Add([a, b]) => Some(Semiring::Num(x(a)?.num()? + x(b)?.num()?)),
        Semiring::Min([a, b]) => Some(Semiring::Num(x(a)?.num()? - x(b)?.num()?)),
        Semiring::Mul([a, b]) => Some(Semiring::Num(x(a)?.num()? * x(b)?.num()?)),
        _ => None,
    }
}

// REVIEW
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

// REVIEW
pub struct Destroy<A: Applier<Semiring, BindAnalysis>> {
    e: A,
}

pub struct RenameSum {
    fresh: Var,
    e: Pattern<Semiring>,
}

// REVIEW
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

// REVIEW
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

// REVIEW
pub fn rules() -> Vec<Rewrite<Semiring, BindAnalysis>> {
    let mut rs = vec![
        rw!("let-const";
            "(let ?v ?e ?c)" => "?c" if is_const(var("?c"))),
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
        rw!("mul-1"; "(* ?a 1)" => "?a"),
        rw!("pow0"; "(pow ?x 0)" => "1"),
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
        rw!("subtract";  "(- ?a ?b)" <=> "(+ ?a (* -1 ?b))"),
        // NOTE boom!
        rw!("div-canon"; "(/ ?a ?b)" <=> "(* ?a (pow ?b -1))"),
        rw!("eq-comm";   "(= ?a ?b)"        <=> "(= ?b ?a)"),
        rw!("add-mul-dist"; "(* (+ ?a ?b) ?c)" <=> "(+ (* ?a ?c) (* ?b ?c))"),
        rw!("add-sum-dist"; "(sum (var ?x) (+ ?a ?b))" <=> "(+ (sum (var ?x) ?a) (sum (var ?x) ?b))"),
        rw!("pushdown-sum-bound"; "(* ?b (sum ?x ?a))" <=> "(sum ?x (* ?b ?a))" if not_free(var("?x"), var("?b"))),
        // NOTE bang!
        rw!("exp-mul"; "(* (pow ?a ?b) (pow ?a ?c))" <=> "(pow ?a (+ ?b ?c))"),
        rw!("base-mul"; "(* (pow ?a ?b) (pow ?c ?b))" <=> "(pow (* ?a ?c) ?b)"),
        rw!("pow1"; "(pow ?x 1)" <=> "?x"),
        rw!("pow2"; "(pow ?x 2)" <=> "(* ?x ?x)"),
        rw!("pow-recip"; "(pow ?x -1)" <=> "(/ 1 ?x)"),
    ].concat());
    rs.extend(vec![
        rw!("sigma-induction";
            "(* (* (I (E (var v) (var t)))
                   (I (= (D (var s) (var t)) (+ (D (var s) (var v)) 1))))
                (sig (var s) (var v)))"
            <=>
            "(* (I (E (var v) (var t))) (sig (var s) (var v) (var t)))"
        ),
    ].concat());
    rs.extend(vec![
        rw!("C-definition";
            "(C ?s ?v)"
            =>
            "(sum (var u) (* (I (neq (var u) ?v)) (/ (sig ?s ?v (var u)) (sig ?s (var u)))))"
        ),
        rw!("C-inline";
            "(sum ?t (* (I (neq ?t ?v)) (/ (sig ?s ?v ?t) (sig ?s ?t))))"
            =>
            "(C ?s ?v)"
        ),
        rw!("sig-svt";
            "(sig ?s ?v ?t)"
            =>
            "(* (I (= (D ?s ?t) (+ (D ?s ?v) (D ?v ?t)))) (* (sig ?s ?v) (sig ?v ?t)))"
        )
    ]);
    rs
}

// REVIEW
fn is_const(v: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph[subst[v]].data.constant.is_some()
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

// REVIEW
pub fn normalize() -> Vec<Rewrite<Semiring, BindAnalysis>> {
    vec![
        rw_1(
            "pushdown-mul",
            "(* ?a (+ ?b ?c))",
            "(+ (* ?a ?b) (* ?a ?c))",
        ),
        rw_1(
            "pushdown-mul-2",
            "(* (+ ?b ?c) ?a)",
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
        // rw_1("let-const", "(let ?v1 ?e ?n))", "?n" if is_const(var("?c"))),
        rw!("let-const"; "(let ?v1 ?e ?n))" => "?n" if is_const(var("?c"))),
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
        rw_1("subtract", "(- ?a ?b)", "(+ ?a (* -1 ?b))"),
    ]
}

// REVIEW
// TODO use iteration data to compute this incrementally
pub fn solve_eqs(runner: &mut Runner<Semiring, BindAnalysis>) -> Result<(), String> {
    let mut fingerprints: HashMap<&Vec<i32>, Id> = HashMap::new();
    for class in runner.egraph.classes() {
        if let Some(fp) = &class.data.fingerprint {
            if let Some(_c) = fingerprints.get(fp) {
                todo!()
            } else {
                fingerprints.insert(fp, class.id);
                todo!()
            }
        }
    }
    Ok(())
}
