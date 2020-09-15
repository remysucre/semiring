use egg::*;
use semiring::*;

test_fn! {
    betweenness_centrality, rules(),
    runner = Runner::default()
        .with_node_limit(750_000)
        .with_iter_limit(400),
    "(sum (var t)
       (* (* (I (E (var v) (var t)))
             (I (= (D (var s) (var t)) (+ (D (var s) (var v)) (lit 1)))))
          (* (/ (sig (var s) (var v)) (sig (var s) (var t)))
             (+ (lit 1) (C (var s) (var t))))))"
    =>
    "(+ (sum (var t)
          (* (I (E (var v) (var t)))
             (/ (sig (var s) (var v) (var t)) (sig (var s) (var t)))))
        (sum (var t)
          (* (I (E (var v) (var t)))
             (* (/ (sig (var s) (var v) (var t)) (sig (var s) (var t)))
                (C (var s) (var t))))))"
    // "(+ (sum (var t)
    //       (* (I (E (var v) (var t)))
    //          (/ (sig (var s) (var v) (var t)) (sig (var s) (var t)))))
    //     (sum (var t)
    //       (* (* (I (E (var v) (var t))) (I (neq (var u) (var t))))
    //          (* (/ (* (sig (var s) (var v) (var t)) (sig (var s) (var t) (var u)))
    //                (* (sig (var s) (var t)) (sig (var s) (var u))))
    //             (+ (lit 1) (C (var s) (var t)))))))"
    // =>
    // "(sum (var t)
    //    (* (I (E (var v) (var t)))
    //       (* (/ (sig (var s) (var v) (var t)) (sig (var s) (var t)))
    //          (+ (lit 1) (C (var s) (var t))))))"
}
