use egg::{rewrite as rw, *};

use crate::analysis::*;
use crate::lang::*;
use crate::EGraph;

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

impl Applier<Semiring, SemiringAnalysis> for CaptureAvoid {
    fn apply_one(&self, egraph: &mut EGraph, eclass: Id, subst: &Subst) -> Vec<Id> {
        let e = subst[self.e];
        let v2 = subst[self.v2];
        let v2_free_in_e = egraph[e].data.free.contains(&v2);
        if v2_free_in_e {
            let mut subst = subst.clone();
            let sym = egraph.add(Semiring::Symbol(format!("_{}", eclass).into()));
            let var = Semiring::Var(sym);
            subst.insert(self.fresh, egraph.add(var));
            self.if_free.apply_one(egraph, eclass, &subst)
        } else {
            self.if_not_free.apply_one(egraph, eclass, &subst)
        }
    }
}

pub struct Destroy<A: Applier<Semiring, SemiringAnalysis>> {
    e: A,
}

pub struct RenameSum {
    fresh: Var,
    e: Pattern<Semiring>,
}

impl<A: Applier<Semiring, SemiringAnalysis>> Applier<Semiring, SemiringAnalysis> for Destroy<A> {
    fn apply_one(&self, egraph: &mut EGraph, eclass: Id, subst: &Subst) -> Vec<Id> {
        egraph[eclass].nodes.clear();
        self.e.apply_one(egraph, eclass, subst)
    }
}

impl Applier<Semiring, SemiringAnalysis> for Found {
    fn apply_one(&self, _egraph: &mut EGraph, _eclass: Id, _subst: &Subst) -> Vec<Id> {
        panic!("Found {}", self.msg)
    }
}

impl Applier<Semiring, SemiringAnalysis> for RenameSum {
    fn apply_one(&self, egraph: &mut EGraph, eclass: Id, subst: &Subst) -> Vec<Id> {
        let mut subst = subst.clone();
        let sym = egraph.add(Semiring::Symbol(format!("_{}", eclass).into()));
        let var = Semiring::Var(sym);
        subst.insert(self.fresh, egraph.add(var));
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
    move |egraph, _, subst| egraph[subst[b]].data.free.contains(&subst[x])
}

pub fn rules() -> Vec<Rewrite<Semiring, SemiringAnalysis>> {
    let mut rs = vec![
        rw!("let-const";
            "(let ?v ?e ?c)" => "?c" if is_const(var("?c"))),
        rw!("let-var-same"; "(let (var ?v1) ?e (var ?v1))" => "?e"),
        rw!("let-var-diff"; "(let (var ?v1) ?e (var ?v2))" => "(var ?v2)"
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw!("swap-sum"; "(sum ?x (sum ?y ?e))" => "(sum ?y (sum ?x ?e))"),
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
                if_free: "(sum ?fresh (let ?v1 ?e (let ?v2 ?fresh ?body)))".parse().unwrap(),
            }}
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw!("mul-1"; "(* ?a 1)" => "?a"),
        rw!("pow0"; "(pow ?x 0)" => "1"),
    ];
    rs.extend(
        vec![
            // subst rules
            rw!("let-add";  "(let ?v ?e (+ ?a ?b))" <=> "(+ (let ?v ?e ?a) (let ?v ?e ?b))"),
            rw!("let-eq";   "(let ?v ?e (= ?a ?b))" <=> "(= (let ?v ?e ?a) (let ?v ?e ?b))"),
            // open term rules
            rw!("add-comm";  "(+ ?a ?b)"        <=> "(+ ?b ?a)"),
            rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" <=> "(+ ?a (+ ?b ?c))"),
            rw!("mul-comm";  "(* ?a ?b)"        <=> "(* ?b ?a)"),
            rw!("mul-assoc"; "(* (* ?a ?b) ?c)" <=> "(* ?a (* ?b ?c))"),
            rw!("subtract";  "(- ?a ?b)" <=> "(+ ?a (* -1 ?b))"),
            // rw!("div-canon"; "(/ ?a ?b)" <=> "(* ?a (pow ?b -1))"),
            rw!("eq-comm";   "(= ?a ?b)"        <=> "(= ?b ?a)"),
            rw!("add-mul-dist"; "(* (+ ?a ?b) ?c)" <=> "(+ (* ?a ?c) (* ?b ?c))"),
            rw!("add-sum-dist"; "(sum ?x (+ ?a ?b))" <=> "(+ (sum ?x ?a) (sum ?x ?b))"),
            rw!("pushdown-sum-bound"; "(* ?b (sum ?x ?a))" <=> "(sum ?x (* ?b ?a))"
            if not_free(var("?x"), var("?b"))),
            // NOTE bang!
            // rw!("exp-mul"; "(* (pow ?a ?b) (pow ?a ?c))" <=> "(pow ?a (+ ?b ?c))"),
            // rw!("base-mul"; "(* (pow ?a ?b) (pow ?c ?b))" <=> "(pow (* ?a ?c) ?b)"),
            // rw!("pow1"; "(pow ?x 1)" <=> "?x"),
            // rw!("pow2"; "(pow ?x 2)" <=> "(* ?x ?x)"),
            // rw!("pow-recip"; "(pow ?x -1)" <=> "(/ 1 ?x)"),
            // NOTE lemmas
            rw!("l-49";  "(I (< ?j ?t))" <=> "(+ (I (< ?j (- ?t 1))) (I (= ?j (- ?t 1))))"),
            // rw!("l-50";  "0" <=> "(* (I (< ?j ?s)) (I (= ?j ?s)))"),
            rw!("l-51";  "(I (< ?j ?t))" <=> "(* (I (< ?j ?t)) (I (<= ?j ?t)))"),
            rw!("l-52";  "(I (= ?j ?t))" <=> "(* (I (= ?j ?t)) (I (<= ?j ?t)))"),
            rw!("l-53";  "(I (< ?j ?t))" <=> "(I (<= ?j (- ?t 1)))"),
            // rw!("l-80";  "(I (<= ?j ?t))" <=> "(I (< (- ?j 1) ?t))"),
            // rw!("l-81";  "(I (< ?j ?t))" <=> "(- (I (<= ?j ?t)) (I (= ?j ?t)))"),
            // rw!("l-82";  "(I (<= ?j ?t))" <=> "(- (I (<= (- ?j 1) ?t)) (I (= (- ?j 1) ?t)))"),
        ]
        .concat(),
    );
    rs.extend(vec![
        rw!("trivial"   ; "(sum ?w (* ?w (I (= ?x ?w))))"          => "?x"),
        rw!("nat"       ; "(* (I (> (- ?a ?b) ?c)) (I (> ?a ?c)))" => "(I (> (- ?a ?b) ?c))"),
        rw!("exclusion" ; "(- 1 (I (<= ?a ?b)))"                   => "(I (> ?a ?b))"),
    ]);
    rs
}

fn is_const(v: Var) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph[subst[v]].data.constant.is_some()
}

// one way destructive rewrite
fn rw_1(
    name: &'static str,
    lhs: &'static str,
    rhs: &'static str,
) -> Rewrite<Semiring, SemiringAnalysis> {
    Rewrite::new(
        name,
        lhs.parse::<Pattern<Semiring>>().unwrap(),
        Destroy {
            e: rhs.parse::<Pattern<Semiring>>().unwrap(),
        },
    )
    .unwrap()
}

pub fn normalize() -> Vec<Rewrite<Semiring, SemiringAnalysis>> {
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
        rw!("let-const"; "(let ?v1 ?e ?n))" => "?n" if is_const(var("?n"))),
        rw_1("let-var-same", "(let ?v1 ?e ?v1)", "?e"),
        rw!("let-var-diff"; "(let (var ?v1) ?e (var ?v2))" =>
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

pub fn lemmas() -> Vec<Rewrite<Semiring, SemiringAnalysis>> {
    vec![
        rw!("l-49";  "(I (< ?j ?t))" <=> "(+ (I (< ?j (- ?t 1))) (I (= ?j (- ?t 1))))"),
        // rw!("l-50";  "0" <=> "(* (I (< ?j ?s)) (I (= ?j ?s)))"),
        rw!("l-51";  "(I (< ?j ?t))" <=> "(* (I (< ?j ?t)) (I (<= ?j ?t)))"),
        rw!("l-52";  "(I (= ?j ?t))" <=> "(* (I (= ?j ?t)) (I (<= ?j ?t)))"),
        rw!("l-53";  "(I (< ?j ?t))" <=> "(I (<= ?j (- ?t 1)))"),
    ]
    .concat()
}
