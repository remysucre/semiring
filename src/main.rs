use egg::*;
// use semiring::analysis::*;
use semiring::lang::*;
use semiring::rewrites::*;
// use std::time;
use std::io::{self, Read};

fn main() {
    let mut start = String::new();
    io::stdin().read_to_string(&mut start).unwrap();
    let runner = Runner::default().with_expr(&start.parse().unwrap())
                                  .run(&elim_sums());
    let (egraph, root) = (runner.egraph, runner.roots[0]);

    let mut extractor = Extractor::new(&egraph, VarCost);
    let (_, best) = extractor.find_best(root);

    let normalize_runner = Runner::default().with_expr(&best).run(&normalize());
    let (egraph, root) = (normalize_runner.egraph, normalize_runner.roots[0]);
    let mut extractor = Extractor::new(&egraph, AstSize);
    let (_, best) = extractor.find_best(root);
    println!("{}", best.pretty(40));
}
