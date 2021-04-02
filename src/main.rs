use egg::*;
use egg::{rewrite as rw, *};
// use semiring::analysis::*;
use semiring::lang::*;
use semiring::rewrites::*;
// use std::time;
use std::io::{self, Read};

#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    // (+ (weight (var w) (var x) (var z))
    //    (sum y (sum w1 (* (* (var w1) (I (rel R (var x) (var y) (var w1))))
    //                      (weight (var w2) (var y) (var z))))))
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    if let Some(matches) = matches.subcommand_matches("extract") {
        let g = matches.value_of("G").unwrap();
        println!("{}", g);
        let mut start = String::new();
        io::stdin().read_to_string(&mut start).unwrap();
        let mut rls = rules();
        let extract_g = Rewrite::new(
            "extract-g",
            g.parse::<Pattern<Semiring>>().unwrap(),
            "(fun-g)".parse::<Pattern<Semiring>>().unwrap(),
        )
        .unwrap();
        rls.push(extract_g);

        let runner = Runner::default()
            .with_expr(&start.parse().unwrap())
            .run(&rls);
        let (egraph, root) = (runner.egraph, runner.roots[0]);

        let mut extractor = Extractor::new(&egraph, GCost);
        let (_, best) = extractor.find_best(root);
        println!("{}", best.pretty(40));
    } else {
        let mut start = String::new();
        io::stdin().read_to_string(&mut start).unwrap();
        let runner = Runner::default()
            .with_expr(&start.parse().unwrap())
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
}
