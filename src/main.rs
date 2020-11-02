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
                        &"(sum (var w) (* (var w) (I (= (var x) (var w)))))"
//                         &"
// (sum (var w)
//      (* (+ (rel E (var x) (var z) (var w))
//            (sum (var y)
//                 (sum (var w1)
//                      (sum (var w2)
//                           (* (* (rel R (var x) (var y) (var w1))
//                                 (rel E (var y) (var z) (var w2)))
//                              (I (= (var w) (* (var w1) (var w2)))))))))
//         (var w)))
// "
                .parse().unwrap())
                .with_expr(
                        &"(var x)"
//             &"
// (+ (sum (var w)
//         (* (var w)
//            (rel E (var x) (var z) (var w))))
//    (sum (var y)
//         (* (sum (var w1)
//                 (* (var w1)
//                    (rel R (var x) (var y) (var w1))))
//            (sum (var w2)
//                 (* (var w2)
//                    (rel E (var y) (var z) (var w2)))))))
// "
                .parse()
                .unwrap()
        )
        // .with_hook(solve_eqs)
        .run(&rules());
        r.print_report();
        assert_eq!(r.egraph.find(r.roots[0]), r.egraph.find(r.roots[1]));
}
