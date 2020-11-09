use semiring::rewrites::*;
use egg::*;

egg::test_fn! {
    apsp, rules(),
    runner = Runner::default()
        .with_node_limit(50_000)
        .with_iter_limit(60),
    "
(sum (var w)
     (* (+ (rel E (var x) (var z) (var w))
           (sum (var y)
                (sum (var w1)
                     (sum (var w2)
                          (* (* (rel R (var x) (var y) (var w1))
                                (rel E (var y) (var z) (var w2)))
                             (I (= (var w) (* (var w1) (var w2)))))))))
        (var w)))
"
    =>
    "
(+ (sum (var w)
        (* (var w)
           (rel E (var x) (var z) (var w))))
   (sum (var y)
        (sum (var w1)
             (sum (var w2)
                  (* (* (rel R (var x) (var y) (var w1))
                        (var w1))
                     (* (rel E (var y) (var z) (var w2))
                        (var w2)))))))
"
}
