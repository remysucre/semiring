use egg::*;
// use semiring::analysis::*;
use semiring::lang::*;
use semiring::rewrites::*;
// use std::time;

fn main() {
    let start = "(sum w
         (* (+ (I (rel E (var x) (var z) (var w)))
               (sum y
                    (sum w1
                         (sum w2
                              (* (* (I (rel R (var x) (var y) (var w1)))
                                    (I (rel E (var y) (var z) (var w2))))
                                 (I (= (var w) (* (var w1) (var w2)))))))))
            (var w)))".parse().unwrap();
    let runner = Runner::default().with_expr(&start).run(&elim_sums());
    let (egraph, root) = (runner.egraph, runner.roots[0]);

    let mut extractor = Extractor::new(&egraph, VarCost);
    let (_, best) = extractor.find_best(root);

    let normalize_runner = Runner::default().with_expr(&best).run(&normalize());
    let (egraph, root) = (normalize_runner.egraph, normalize_runner.roots[0]);
    let mut extractor = Extractor::new(&egraph, AstSize);
    let (_, best) = extractor.find_best(root);
    println!("{}", best.pretty(40));
}
