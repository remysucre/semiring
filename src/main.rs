use egg::*;
use semiring::*;

fn main() {
        env_logger::init();
        let r = Runner::<Semiring, BindAnalysis>::default()
        .with_expr(
            &"(+ (var x) (< (var j) (var t)))".parse().unwrap()
        )
        .with_expr(
            &"(+ (var x) (<= (var j) (- (var t) 1)))".parse().unwrap()
        )
        .with_hook(solve_eqs)
        .run(&normalize());
        // .with_expr(
        //     &"(sum (var j)
        // (sum (var w)
        //    (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
        //          (* (rel R (- (var t) 1) (var j) (var w))
        //             (* (I (< (var j) (var t)))
        //                (I (> (var t) 1)))))
        //       (* (var w)
        //         (* (I (<= (- (var t) (var k)) (var j)))
        //            (I (<= (var j) (var t))))))))"
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
        // .with_expr(
        //     &"(+ 3 (< 6 8))".parse().unwrap()
        // )
        // .with_expr(
        //     &"(+ 3 (< 2 3))".parse().unwrap()
        // )

        r.print_report()
}
