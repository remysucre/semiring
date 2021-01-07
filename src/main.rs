use egg::*;
// use semiring::analysis::*;
use semiring::lang::*;
use semiring::rewrites::*;
// use std::time;

// TODO could optimze for summation depth too
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
    let start = "(sum t
     (* (I (= (rel D (var s) (var t))
              (+ (rel D (var s) (var v))
                 (rel D (var v) (var t)))))
        (/ (* (rel sigma (var s) (var v))
              (+ (I (rel E (var v) (var t)))
                 (sum u (* (* (rel sigma (var u) (var t)) (I (rel E (var v) (var u))))
                           (I (= (rel D (var v) (var t))
                                 (+ 1 (rel D (var u) (var t)))))))))
           (rel sigma (var s) (var t)))))".parse().unwrap();
    let runner = Runner::default().with_expr(&start).run(&elim_sums());
    let (egraph, root) = (runner.egraph, runner.roots[0]);

    let mut extractor = Extractor::new(&egraph, VarCost);
    let (best_cost, best) = extractor.find_best(root);
    println!("{}", best.pretty(40));
    println!("{}", best_cost);

    let normalize_runner = Runner::default().with_expr(&best).run(&normalize());
    let (egraph, root) = (normalize_runner.egraph, normalize_runner.roots[0]);
    let mut extractor = Extractor::new(&egraph, AstSize);
    let (best_cost, best) = extractor.find_best(root);
    println!("{}", best.pretty(40));
    println!("{}", best_cost);
}
