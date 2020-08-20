use egg::{rewrite as rw, *};
use std::collections::HashSet;

define_language! {
    enum Semiring {
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

type EGraph = egg::EGraph<Semiring, BindAnalysis>;

#[derive(Default, Clone)]
struct BindAnalysis;

#[derive(Debug)]
struct Data {
    free: HashSet<Id>,
}

impl Analysis<Semiring> for BindAnalysis {
    type Data = Data;
    fn merge(&self, to: &mut Data, from: Data) -> bool {
        let before_len = to.free.len();
        to.free.retain(|i| from.free.contains(i));
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
        Data { free }
    }

    fn modify(_egraph: &mut EGraph, _id: Id) {}
}

struct CaptureAvoid {
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

struct RenameSum {
    fresh: Var,
    e: Pattern<Semiring>,
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

fn rules() -> Vec<Rewrite<Semiring, BindAnalysis>> {
    let mut rs = vec![
        // subst rules
        rw!("let-add";  "(let ?v ?e (+   ?a ?b))" => "(+   (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-eq";   "(let ?v ?e (=   ?a ?b))" => "(=   (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-lit"; "(let ?v1 ?e (lit ?n))" => "(lit ?n)"),
        rw!("let-var-same"; "(let ?v1 ?e (var ?v1))" => "?e"),
        rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
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
                if_free: "(sum ?fresh (let ?v1 ?e (let ?v2 (var ?fresh) ?body)))".parse().unwrap(),
            }}
            if is_not_same_var(var("?v1"), var("?v2"))),
    ];
    rs.extend(vec![
        // open term rules
        rw!("add-comm";  "(+ ?a ?b)"        <=> "(+ ?b ?a)"),
        rw!("add-assoc"; "(+ (+ ?a ?b) ?c)" <=> "(+ ?a (+ ?b ?c))"),
        rw!("mul-comm";  "(* ?a ?b)"        <=> "(* ?b ?a)"),
        rw!("mul-assoc"; "(* (* ?a ?b) ?c)" <=> "(* ?a (* ?b ?c))"),
        rw!("eq-comm";   "(= ?a ?b)"        <=> "(= ?b ?a)"),
        rw!("add-mul-dist"; "(* (+ ?a ?b) ?c)" <=> "(+ (* ?a ?c) (* ?b ?c))"),
        rw!("add-sum-dist"; "(sum (var ?x) (+ ?a ?b))" <=> "(+ (sum (var ?x) ?a) (sum (var ?x) ?b))"),
        rw!("pushdown-sum-bound"; "(* ?b (sum ?x ?a))" <=> "(sum ?x (* ?b ?a))" if not_free(var("?x"), var("?b"))),

    ].concat());
    rs.extend(vec![
        rw!("trivial-id"; "(sum ?w (* (I (= ?x ?w)) ?w))" => "?x"),
    ]);
    rs.extend(vec![
        rw!("id-51"; "(* (I (< ?j ?t)) (I (<= ?j ?t)))" => "(I (< ?j ?t))"),
        rw!("id-52"; "(* (I (= ?j ?t)) (I (<= ?j ?t)))" => "(I (= ?j ?t))"),
        rw!("id-53"; "(I (< ?j ?t))" => "(I (<= ?j (- ?t (lit 1))))"),
    ]);
    rs.extend(vec![
        //rw!("S-def-APSP"; "(sum ?w1 (* (rel R ?x ?y ?w1) ?w1))" => "(rel S ?x ?y)"),
        rw!("S-def-running-total"; "(sum ?j (sum ?w (* (* (rel R ?t ?j ?w) ?w) (* (I (<= (lit 1) ?j)) (I (<= ?j ?t))))))" => "
(rel S ?t)"),
    ]);
    rs
}

//egg::test_fn! {
//    apsp, rules(),
//    runner = Runner::default()
//        .with_time_limit(std::time::Duration::from_secs(20))
//        .with_node_limit(100_000)
//        .with_iter_limit(60),
//    "(sum (var w)
//        (* (+ (rel E (var x) (var y) (var w))
//              (sum (var y)
//                 (sum (var w1)
//                    (sum (var w2)
//                       (* (rel R (var x) (var y) (var w1))
//                          (* (rel E (var y) (var z) (var w2))
//                             (I (= (* (var w1) (var w2)) (var w)))))))))
//           (var w)))"
//    =>
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) w))
//    //    (sum (var w)
//    //       (sum (var y)
//    //          (sum (var w1)
//    //              (sum (var w2)
//    //                   (* w (* (rel R (var x) (var y) (var w1))
//    //                      (* (rel E (var y) (var z) (var w2))
//    //                         (I (= (* w1 w2) w))))))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y) (sum (var w1) (sum (var w2)
//    //      (* (rel R (var x) (var y) (var w1))
//    //         (* (rel E (var y) (var z) (var w2))
//    //            (sum (var w) (* (I (= (* (var w1) (var w2)) (var w))) (var w)))))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y) (sum (var w1) (sum (var w2)
//    //      (* (rel R (var x) (var y) (var w1))
//    //         (* (rel E (var y) (var z) (var w2))
//    //            (* (var w1) (var w2))))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y) (sum (var w1) (sum (var w2)
//    //      (* (* (rel R (var x) (var y) (var w1))
//    //            (var w1))
//    //         (* (rel E (var y) (var z) (var w2))
//    //            (var w2)))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y)
//    //       (* (sum (var w1) (* (rel R (var x) (var y) (var w1)) (var w1)))
//    //          (sum (var w2) (* (rel E (var y) (var z) (var w2)) (var w2))))))"
//    "(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//        (sum (var y)
//           (* (rel S (var x) (var y))
//              (sum (var w2) (* (rel E (var y) (var z) (var w2)) (var w2))))))"
//    ,
//}

egg::test_fn! {
    running_total, rules(),
    runner = Runner::default()
        .with_time_limit(std::time::Duration::from_secs(20))
        .with_node_limit(100_000)
        .with_iter_limit(60),
    "(sum (var j) (sum (var w)
       (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
             (* (rel R (- (var t) (lit 1)) (var j) (var w))
                (* (I (< (var j) (var t))) (I (> (var t) (lit 1))))))
          (* (var w) (* (I (<= (lit 1) (var j))) (I (<= (var j) (var t))))))))"
    =>
    //"(+ (sum (var j) (sum (var w)
    //       (* (* (rel v (var j) (var w)) (I (= (var t) (var j))))
    //          (* (var w)
    //             (* (I (<= (lit 1) (var j)))
    //                (I (<= (var j) (var t))))))))
    //    (sum (var j) (sum (var w)
    //       (* (* (rel R (- (var t) (lit 1)) (var j) (var w))
    //             (* (I (< (var j) (var t)))
    //                (I (> (var t) (lit 1)))))
    //          (* (var w)
    //             (* (I (<= (lit 1) (var j)))
    //                (I (<= (var j) (var t)))))))))"
    //"(+ (sum (var j) (sum (var w)
    //       (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
    //          (* (var w) (I (<= (lit 1) (var j)))))))
    //    (sum (var j) (sum (var w)
    //       (* (* (rel R (- (var t) (lit 1)) (var j) (var w))
    //             (* (I (< (var j) (var t)))
    //                (I (> (var t) (lit 1)))))
    //          (* (var w) (I (<= (lit 1) (var j))))))))"
    //"(+ (sum (var j) (sum (var w)
    //       (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
    //          (* (var w) (I (<= (lit 1) (var j)))))))
    //    (* (I (> (var t) (lit 1)))
    //    (sum (var j) (sum (var w)
    //       (* (* (rel R (- (var t) (lit 1)) (var j) (var w))
    //             (I (<= (var j) (- (var t) (lit 1)))))
    //          (* (var w) (I (<= (lit 1) (var j)))))))))"
    "(+ (sum (var j) (sum (var w)
           (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
              (* (var w) (I (<= (lit 1) (var j)))))))
        (* (I (> (var t) (lit 1))) (rel S (- (var ?t) (lit 1)))))"
    ,
}
