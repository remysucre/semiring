use egg::*;
use semiring::*;

fn main() {
    let r = Runner::<Semiring, BindAnalysis>::default()
        .with_expr(
            &"(sum (var j)
        (sum (var w)
           (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
                 (* (rel R (- (var t) (lit 1)) (var j) (var w))
                    (* (I (< (var j) (var t)))
                       (I (> (var t) (lit 1))))))
              (* (var w)
                (* (I (<= (- (var t) (var k)) (var j)))
                   (I (<= (var j) (var t))))))))"
                .parse()
                .unwrap()
        )
        .run(&normalize());
    r.egraph.dot().to_png("normalized.png").unwrap();

    let (egraph, root) = (r.egraph, r.roots[0]);
    let mut extractor = Extractor::new(&egraph, AstSize);
    let (_best_cost, best) = extractor.find_best(root);
    println!("{}", best.pretty(60));
}
