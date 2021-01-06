use egg::*;
use semiring::analysis::*;
use semiring::lang::*;
use semiring::rewrites::*;
use std::time;

struct SemiringCost;

impl CostFunction<Semiring> for SemiringCost {
    type Cost = u64;
    fn cost<C>(&mut self, enode: &Semiring, mut costs: C) -> Self::Cost
    where C: FnMut(Id) -> Self::Cost
    {
        let op_cost = match enode {
            Semiring::Symbol(s) => if s.as_str() == "R" { 1000 } else { 0 },
            _ => 0
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}

struct VarCost;

impl CostFunction<Semiring> for VarCost {
    type Cost = u64;
    fn cost<C>(&mut self, enode: &Semiring, mut costs: C) -> Self::Cost
    where C: FnMut(Id) -> Self::Cost
    {
        let op_cost = match enode {
            Semiring::Sum(_) => 1000,
            _ => 0
        };
        enode.fold(op_cost, |sum, id| sum + costs(id))
    }
}

fn main() {
    let start = "(sum w
     (* (+ (rel E (var x) (var z) (var w))
           (sum y
                (sum w1
                     (sum w2
                          (* (* (rel R (var x) (var y) (var w1))
                                (rel E (var y) (var z) (var w2)))
                             (I (= (var w) (* (var w1) (var w2)))))))))
        (var w)))".parse().unwrap();
    let runner = Runner::default().with_expr(&start).run(&rules());
    let (egraph, root) = (runner.egraph, runner.roots[0]);

    let mut extractor = Extractor::new(&egraph, VarCost);
    let (best_cost, best) = extractor.find_best(root);
    println!("{}", best.pretty(40));
    println!("{}", best_cost);
}
