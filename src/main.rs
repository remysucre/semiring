use egg::*;
use semiring::*;

fn main() {
    Runner::<Semiring, BindAnalysis>::default()
    .with_expr(&"(sum (var j) (sum (var w)
       (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
             (* (rel R (- (var t) (lit 1)) (var j) (var w))
                (* (I (< (var j) (var t))) (I (> (var t) (lit 1))))))
          (* (var w) (* (I (<= (lit 1) (var j))) (I (<= (var j) (var t))))))))".parse().unwrap())
    .with_expr(&"(rel S (- (var ?t) (lit 1)))".parse().unwrap())
    .with_hook(|runner| {
        let eg = &runner.egraph;
        if eg[runner.roots[1]].len() > 1 {
            Err("proved eq".to_string())
        } else {
            Ok(())
        }
    })
    .run(&rules());
}
