use egg::*;
use semiring::*;
use std::time;

fn main() {
        env_logger::init();
        let r = Runner::<Semiring, BindAnalysis>::default()
                .with_time_limit(time::Duration::from_secs(20))
                .with_node_limit(50_000)
                .with_iter_limit(60)

                .with_expr(
            &"(+ (sum (var j) (sum (var w)
(* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
(* (* (I (< (var j) (var t))) (I (> (var t) 1)))
(* (I (<= 1 (var j))) (I (<= (var j) (var t))))))))
            (sum (var j) (sum (var w)
(* (* (rel v (var j) (var w)) (var w))
(* (I (= (var t) (var j))) (* (I (<= 1 (var j))) (I (<= (var j) (var t)))))))))"
                .parse()
                .unwrap()
        )
                .with_expr(
            &"(+ (* (I (> (var t) 1)) (sum (var j) (sum (var w)
(* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
(* (I (<= (var j) (- (var t) 1))) (I (<= 1 (var j))))))))
            (sum (var j) (sum (var w)
(* (* (rel v (var j) (var w)) (var w))
(* (I (= (var t) (var j))) (I (<= 1 (var j))))))))"
                .parse()
                .unwrap()
        )
//         .with_expr(
//             &"(+ (sum (var j) (sum (var w)
// (* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
// (* (* (I (< (var j) (var t))) (I (> (var t) 1)))
// (* (I (<= 1 (var j))) (I (<= (var j) (var t))))))))
// (sum (var j) (sum (var w)
// (* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
// (* (* (I (< (var j) (var t))) (I (> (var t) 1)))
// (* (I (<= 1 (var j))) (I (<= (var j) (var t)))))))))"
//                 .parse()
//                 .unwrap()
//         )
//         .with_expr(
//             &"(+ (* (I (> (var t) 1)) (sum (var j) (sum (var w)
// (* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
// (* (I (<= (var j) (- (var t) 1))) (I (<= 1 (var j))))))))
// (sum (var j) (sum (var w)
// (* (* (rel v (var j) (var w)) (var w))
// (* (I (= (var t) (var j))) (I (<= 1 (var j))))))))"
//                 .parse()
//                 .unwrap()
//         )
//         .with_expr(
//             &"(sum (var j) (sum (var w)
// (* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
// (* (* (I (< (var j) (var t))) (I (> (var t) 1)))
// (* (I (<= 1 (var j))) (I (<= (var j) (var t))))))))"
//                 .parse()
//                 .unwrap()
//         )
//         .with_expr(
//             &"(* (I (> (var t) 1)) (sum (var j) (sum (var w)
// (* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
// (* (I (<= (var j) (- (var t) 1))) (I (<= 1 (var j))))))))"
//                 .parse()
//                 .unwrap()
//         )
//         .with_expr(
//             &"(sum (var j) (sum (var w)
// (* (* (rel v (var j) (var w)) (var w))
// (* (I (= (var t) (var j))) (* (I (<= 1 (var j))) (I (<= (var j) (var t))))))))"
//                 .parse()
//                 .unwrap()
//         )
//         .with_expr(
//             &"(sum (var j) (sum (var w)
// (* (* (rel v (var j) (var w)) (var w))
// (* (I (= (var t) (var j))) (I (<= 1 (var j)))))))"
//                 .parse()
//                 .unwrap()
//         )
        // .with_expr(
        //     &"(sum (var j)
        // (sum (var w)
        //    (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
        //          (* (rel R (- (var t) 1) (var j) (var w))
        //             (* (I (< (var j) (var t)))
        //                (I (> (var t) 1)))))
        //       (* (var w)
        //         (* (I (<= 1 (var j)))
        //            (I (<= (var j) (var t))))))) )"
        //         .parse()
        //         .unwrap()
        // )
        // .with_expr(
        //     &"(+ (sum (var j)
        // (sum (var w)
        //     (* (* (rel v (var j) (var w)) (var w))
        //        (* (I (= (var t) (var j))) (I (<= 1 (var j)))))))
        // (* (I (> (var t) 1))
        //     (sum (var j)
        //         (sum (var w)
        //             (* (* (rel R (- (var t) 1) (var j) (var w)) (var w))
        //                 (* (I (<= (var j) (- (var t) 1)))
        //                     (I (<= 1 (var j)))))))))"
        //         .parse()
        //         .unwrap()
        // )
        .with_hook(solve_eqs)
        .run(&rules());
        // .with_expr(
        //     &"(+ (var x) (I (< (var j) (var t))))".parse().unwrap()
        // )
        // .with_expr(
        //     &"(+ (var x) (I (<= (var j) (- (var t) 1))))".parse().unwrap()
        // )
        r.print_report();
        assert_eq!(r.egraph.find(r.roots[0]), r.egraph.find(r.roots[1]));
}
