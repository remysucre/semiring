use egg::*;
use semiring::*;
use std::time;

fn main() {
        env_logger::init();
        let r = Runner::<Semiring, BindAnalysis>::default()
                .with_time_limit(time::Duration::from_secs(60))
                .with_node_limit(50_000)
                .with_iter_limit(500)
                .with_expr(
                        &"(def RT (var t))"
//                        &"
// (+ (* (I (> (var t) 1))
//       (sum (var j)
//            (sum (var w)
//                 (* (* (rel R (- (var t) 1) (var j) (var w))
//                       (var w))
//                    (* (I (<= (var j) (- (var t) 1)))
//                       (I (<= 1 (var j))))))))
//    (sum (var j)
//         (sum (var w)
//              (* (* (rel v (var j) (var w))
//                    (var w))
//                 (* (I (= (var t) (var j)))
//                    (I (<= 1 (var j))))))))
// "
                .parse().unwrap())
                .with_expr(
                        &"(+ (* (I (> (var t) 1)) (def RT-rhs (- (var t) 1))) (vec (var t)))"
//                         &"
// (sum (var w)
//      (sum (var j)
//           (* (* (var w) (* (I (<= 1 (var j)))
//                            (I (<= (var j) (var t)))))
//              (+ (* (I (= (var t) (var j)))
//                    (rel v (var j) (var w)))
//                 (* (rel R (- (var t) 1) (var j) (var w))
//                    (* (I (< (var j) (var t)))
//                       (I (> (var t) 1))))))))
// "
                .parse()
                .unwrap()
        )
        // .with_hook(solve_eqs)
        .run(&rules());
        r.print_report();
        assert_eq!(r.egraph.find(r.roots[0]), r.egraph.find(r.roots[1]));
}
