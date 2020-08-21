use egg::*;
use semiring::*;

fn main() {
    Runner::<Semiring, BindAnalysis>::default()
        .with_time_limit(std::time::Duration::from_secs(20))
        .with_node_limit(100_000)
        .with_iter_limit(100)
        .with_scheduler(BackoffScheduler::default().with_initial_match_limit(500))
        .with_expr(&"(sum (var j) (sum (var w)
       (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
             (* (rel R (- (var t) (lit 1)) (var j) (var w))
                (* (I (< (var j) (var t))) (I (> (var t) (lit 1))))))
          (* (var w) (* (I (<= (lit 1) (var j))) (I (<= (var j) (var t))))))))".parse().unwrap())
        .with_expr(&"(+ (sum (var j) (sum (var w)
           (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
              (* (var w) (I (<= (lit 1) (var j)))))))
        (* (I (> (var t) (lit 1))) (rel S (- (var ?t) (lit 1)))))".parse().unwrap())
        .with_hook(|runner| {
            let eg = &runner.egraph;
            if eg.find(runner.roots[0]) == eg.find(runner.roots[1]) {
                Err("proven eq".to_string())
            } else {
                Ok(())
            }
        })
        .run(&rules());
}
