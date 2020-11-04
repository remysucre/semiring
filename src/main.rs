use egg::*;
use semiring::*;
use std::time;

fn main() {
        env_logger::init();
        let r = Runner::<Semiring, BindAnalysis>::default()
                // .with_time_limit(time::Duration::from_secs(20))
                // .with_node_limit(50_000)
                // .with_iter_limit(60)
                .with_expr(
                        &"
(- (sum (var j)
        (sum (var w)
             (* (* (def R (var t) (var j) (var w))
                   (var w))
                (* (I (<= (var j) (var t)))
                   (I (>= (var j) 1))))))
   (sum (var j)
        (sum (var w)
             (* (* (def R (var t) (var j) (var w))
                   (var w))
                (* (I (>= (var j) 1))
                   (* (I (<= (var j)
                             (- (var t) (var k))))
                      (I (> (var t) (var k)))))))))
"
                .parse().unwrap())
                .with_expr(
                        &"
(- (+ (def S (- (var t) 1))
      (* (var w)
         (rel v (var t) (var w))))
   (* (* (var w)
         (rel v (- (var t) (var k)) (var w)))
      (I (> (var t) (var k)))))
"
                .parse()
                .unwrap()
        )
        // .with_hook(solve_eqs)
        .run(&rules());
        r.print_report();
        assert_eq!(r.egraph.find(r.roots[0]), r.egraph.find(r.roots[1]));
}
