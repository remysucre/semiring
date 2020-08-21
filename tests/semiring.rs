use egg::*;
use semiring::*;

//egg::test_fn! {
//    apsp, rules(),
//    runner = Runner::default()
//        .with_time_limit(std::time::Duration::from_secs(20))
//        .with_node_limit(100_000)
//        .with_iter_limit(60),
//    "(sum (var w)
//        (* (+ (rel E (var x) (var y) (var w))
//              (sum (var y)
//                 (sum (var w1)
//                    (sum (var w2)
//                       (* (rel R (var x) (var y) (var w1))
//                          (* (rel E (var y) (var z) (var w2))
//                             (I (= (* (var w1) (var w2)) (var w)))))))))
//           (var w)))"
//    =>
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) w))
//    //    (sum (var w)
//    //       (sum (var y)
//    //          (sum (var w1)
//    //              (sum (var w2)
//    //                   (* w (* (rel R (var x) (var y) (var w1))
//    //                      (* (rel E (var y) (var z) (var w2))
//    //                         (I (= (* w1 w2) w))))))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y) (sum (var w1) (sum (var w2)
//    //      (* (rel R (var x) (var y) (var w1))
//    //         (* (rel E (var y) (var z) (var w2))
//    //            (sum (var w) (* (I (= (* (var w1) (var w2)) (var w))) (var w)))))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y) (sum (var w1) (sum (var w2)
//    //      (* (rel R (var x) (var y) (var w1))
//    //         (* (rel E (var y) (var z) (var w2))
//    //            (* (var w1) (var w2))))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y) (sum (var w1) (sum (var w2)
//    //      (* (* (rel R (var x) (var y) (var w1))
//    //            (var w1))
//    //         (* (rel E (var y) (var z) (var w2))
//    //            (var w2)))))))"
//    //"(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//    //    (sum (var y)
//    //       (* (sum (var w1) (* (rel R (var x) (var y) (var w1)) (var w1)))
//    //          (sum (var w2) (* (rel E (var y) (var z) (var w2)) (var w2))))))"
//    "(+ (sum (var w) (* (rel E (var x) (var y) (var w)) (var w)))
//        (sum (var y)
//           (* (rel S (var x) (var y))
//              (sum (var w2) (* (rel E (var y) (var z) (var w2)) (var w2))))))"
//    ,
//}

egg::test_fn! {
    running_total, rules(),
    runner = Runner::default()
        .with_time_limit(std::time::Duration::from_secs(20))
        .with_node_limit(100_000)
        .with_iter_limit(100)
        .with_scheduler(BackoffScheduler::default().with_initial_match_limit(500)),
    "(sum (var j) (sum (var w)
       (* (+ (* (rel v (var j) (var w)) (I (= (var t) (var j))))
             (* (rel R (- (var t) (lit 1)) (var j) (var w))
                (* (I (< (var j) (var t))) (I (> (var t) (lit 1))))))
          (* (var w) (* (I (<= (lit 1) (var j))) (I (<= (var j) (var t))))))))"
    =>
    //"(+ (sum (var j) (sum (var w)
    //       (* (* (rel v (var j) (var w)) (I (= (var t) (var j))))
    //          (* (var w)
    //             (* (I (<= (lit 1) (var j)))
    //                (I (<= (var j) (var t))))))))
    //    (sum (var j) (sum (var w)
    //       (* (* (rel R (- (var t) (lit 1)) (var j) (var w))
    //             (* (I (< (var j) (var t)))
    //                (I (> (var t) (lit 1)))))
    //          (* (var w)
    //             (* (I (<= (lit 1) (var j)))
    //                (I (<= (var j) (var t)))))))))"
    //"(+ (sum (var j) (sum (var w)
    //       (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
    //          (* (var w) (I (<= (lit 1) (var j)))))))
    //    (sum (var j) (sum (var w)
    //       (* (* (rel R (- (var t) (lit 1)) (var j) (var w))
    //             (* (I (< (var j) (var t)))
    //                (I (> (var t) (lit 1)))))
    //          (* (var w) (I (<= (lit 1) (var j))))))))"
    //"(+ (sum (var j) (sum (var w)
    //       (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
    //          (* (var w) (I (<= (lit 1) (var j)))))))
    //    (* (I (> (var t) (lit 1)))
    //    (sum (var j) (sum (var w)
    //       (* (* (rel R (- (var t) (lit 1)) (var j) (var w))
    //             (I (<= (var j) (- (var t) (lit 1)))))
    //          (* (var w) (I (<= (lit 1) (var j)))))))))"
    "(+ (sum (var j) (sum (var w)
           (* (* (rel v (var j) (var w)) (I (= (var j) (var t))))
              (* (var w) (I (<= (lit 1) (var j)))))))
        (* (I (> (var t) (lit 1))) (rel S (- (var ?t) (lit 1)))))"
    @check |r: Runner<Semiring, BindAnalysis>| r.print_report()
}
