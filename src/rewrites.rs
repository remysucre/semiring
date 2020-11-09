use egg::rewrite as rw;
use egg::*;
use std::collections::HashMap;

use crate::analysis::*;
use crate::lang::*;
use crate::EGraph;

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

// REVIEW
pub struct Destroy<A: Applier<Semiring, SemiringAnalysis>> {
    e: A,
}

pub struct RenameSum {
    fresh: Var,
    e: Pattern<Semiring>,
}

// REVIEW
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
pub fn rules() -> Vec<Rewrite<Semiring, SemiringAnalysis>> {
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
        // FIXME var or no var?
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
        // NOTE lemmas
        rw!("l-49";  "(I (< ?j ?t))" <=> "(+ (I (< ?j (- ?t 1))) (I (= ?j (- ?t 1))))"),
        // rw!("l-50";  "0" <=> "(* (I (< ?j ?s)) (I (= ?j ?s)))"),
        rw!("l-51";  "(I (< ?j ?t))" <=> "(* (I (< ?j ?t)) (I (<= ?j ?t)))"),
        rw!("l-52";  "(I (= ?j ?t))" <=> "(* (I (= ?j ?t)) (I (<= ?j ?t)))"),
        rw!("l-53";  "(I (< ?j ?t))" <=> "(I (<= ?j (- ?t 1)))"),
        // rw!("l-80";  "(I (<= ?j ?t))" <=> "(I (< (- ?j 1) ?t))"),
        // rw!("l-81";  "(I (< ?j ?t))" <=> "(- (I (<= ?j ?t)) (I (= ?j ?t)))"),
        // rw!("l-82";  "(I (<= ?j ?t))" <=> "(- (I (<= (- ?j 1) ?t)) (I (= (- ?j 1) ?t)))"),
    ].concat());
    rs.extend(
        vec![rw!("sigma-induction";
            "(* (* (I (E (var v) (var t)))
                   (I (= (D (var s) (var t)) (+ (D (var s) (var v)) 1))))
                (sig (var s) (var v)))"
            <=>
            "(* (I (E (var v) (var t))) (sig (var s) (var v) (var t)))"
        )]
        .concat(),
    );
    rs.extend(vec![rw!("trivial";
        "(sum ?w (* ?w (I (= ?x ?w))))"
        =>
        "?x"
    // // FIXME t cannot be free in e
    // rw!("UDP-14";
    //     "(sum ?t (I (= ?t ?e)))"
    //     =>
    //     "1"
    // ),
    // // TODO generalize to arbitrary subst as in paper
    // rw!("UDP-13";
    //     "(* ?a (I (= (var ?x) (var ?y))))"
    //     =>
    //     "(* (let ?x (var ?y) ?a) (I (= (var ?x) (var ?y))))"
    )]);
    rs.extend(vec![
        rw!("R-definition";
            "(def R ?t ?j ?w)"
            =>
            "
(+ (* (I (= ?j ?t))
      (rel v ?j ?w))
   (* (rel R (- ?t 1) ?j ?w)
      (* (I (< ?j ?t))
         (I (> ?t 1)))))
"
        ),
        rw!("RT-definition";
            "(def RT ?t)"
            =>
            "
(sum (var w)
     (sum (var j)
          (* (* (var w) (* (I (<= 1 (var j)))
                           (I (<= (var j) ?t))))
             (def R ?t (var j) (var w)))))
"
        ),
        rw!("RT-rhs";
            "(def RT-rhs ?t)"
            =>
            "
(sum (var w)
     (sum (var j)
          (* (* (var w) (* (I (<= 1 (var j)))
                           (I (<= (var j) ?t))))
             (rel R ?t (var j) (var w)))))
"
        ),
        rw!("def-vec";
            "(vec ?t)"
            =>
            "
(sum (var j)
        (sum (var w)
             (* (* (rel v (var j) (var w))
                   (var w))
                (* (I (= ?t (var j)))
                   (I (<= 1 (var j)))))))
"
        ),
        rw!("S-definition";
            "(def S ?t)"
            =>
            "(- (def RT ?t) (def RT (- ?t (var k))))"
        ),
        rw!("S-rhs";
            "(def S-rhs ?t)"
            =>
            "(+ (- (def RT-rhs ?t) (def RT-rhs (- ?t (var k)))) (- (vec (var t)) (vec (- (var t) (var k)))))"
        ),
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
// REVIEW
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
pub fn solve_eqs(runner: &mut Runner<Semiring, SemiringAnalysis>) -> Result<(), String> {
    let mut fingerprints: HashMap<&Vec<i32>, Vec<Id>> = HashMap::new();
    for class in runner.egraph.classes() {
        if let Some(fp) = &class.data.fingerprint {
            fingerprints.entry(fp).or_insert(vec![]).push(class.id);
        }
    }
    let mut to_union = vec![];
    for matches in fingerprints.values() {
        for c_1 in matches.iter() {
            for c_2 in matches.iter() {
                let mut extractor = Extractor::new(&runner.egraph, AstSize);
                let (_, e_1) = extractor.find_best(*c_1);
                let (_, e_2) = extractor.find_best(*c_2);
                let local_runner = Runner::default()
                    .with_expr(&e_1)
                    .with_expr(&e_2)
                    .run(&lemmas());
                if local_runner.egraph.find(local_runner.roots[0])
                    == local_runner.egraph.find(local_runner.roots[1])
                {
                    to_union.push((*c_1, *c_2));
                }
            }
        }
    }
    for (c_1, c_2) in to_union {
        runner.egraph.union(c_1, c_2);
    }
    Ok(())
}

pub fn gen_rosette(lhs: &str, rhs: &str, fvs: &Vec<Id>) -> String {
    // FIXME this won't work, really needs to get the vars
    let fvs_s: Vec<String> = fvs.iter().map(|id| format!("v_{}", id)).collect();
    format!(
        "#lang rosette
         (define (I b) (if b 1 0))
         (define-symbolic {fvs} integer?)
         (if (unsat? (verify (assert (eq? {lhs} {rhs}))))
             (display \"true\")
             (display \"false\"))",
        fvs = &fvs_s.join(" "),
        lhs = lhs,
        rhs = rhs
    )
}
