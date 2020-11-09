use egg::*;
use semiring::rewrites::*;

#[test]
fn apsp() {
    let runner = Runner::default()
        .with_expr(
            &"
(sum (var w)
     (* (+ (rel E (var x) (var z) (var w))
           (sum (var y)
                (sum (var w1)
                     (sum (var w2)
                          (* (* (rel R (var x) (var y) (var w1))
                                (rel E (var y) (var z) (var w2)))
                             (I (= (var w) (* (var w1) (var w2)))))))))
        (var w)))"
                .parse()
                .unwrap(),
        )
        .with_expr(
            &"
(+ (sum (var w)
        (* (var w)
           (rel E (var x) (var z) (var w))))
   (sum (var y)
        (* (sum (var w1)
                (* (var w1)
                   (rel R (var x) (var y) (var w1))))
           (sum (var w2)
                (* (var w2)
                   (rel E (var y) (var z) (var w2)))))))"
                .parse()
                .unwrap(),
        )
        .run(&rules());
    let lhs = runner.roots[0];
    let rhs = runner.roots[1];
    assert_eq!(runner.egraph.find(lhs), runner.egraph.find(rhs))
}
