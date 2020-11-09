use egg::*;
use semiring::rewrites::*;

fn check_eq(x: &str, y:&str) {
    let runner = Runner::default()
        .with_iter_limit(60)
        .with_node_limit(50_000)
        .with_expr(&x.parse().unwrap())
        .with_expr(&y.parse().unwrap())
        .run(&rules());
    let lhs = runner.roots[0];
    let rhs = runner.roots[1];
    assert_eq!(runner.egraph.find(lhs), runner.egraph.find(rhs))
}

#[test]
fn apsp() {
    check_eq("
(sum (var w)
     (* (+ (rel E (var x) (var z) (var w))
           (sum (var y)
                (sum (var w1)
                     (sum (var w2)
                          (* (* (rel R (var x) (var y) (var w1))
                                (rel E (var y) (var z) (var w2)))
                             (I (= (var w) (* (var w1) (var w2)))))))))
        (var w)))
", "
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
    )
}

#[test]
fn running_total() {
    check_eq("
(sum (var w)
     (sum (var j)
          (* (* (var w) (* (I (<= 1 (var j)))
                           (I (<= (var j) (var t)))))
             (+ (* (I (= (var t) (var j)))
                   (rel v (var j) (var w)))
                (* (rel R (- (var t) 1) (var j) (var w))
                   (* (I (< (var j) (var t)))
                      (I (> (var t) 1))))))))
", "
(+ (* (I (> (var t) 1))
      (sum (var j)
           (sum (var w)
                (* (* (rel R (- (var t) 1) (var j) (var w))
                      (var w))
                   (* (I (<= (var j) (- (var t) 1)))
                      (I (<= 1 (var j))))))))
   (sum (var j)
        (sum (var w)
             (* (* (rel v (var j) (var w))
                   (var w))
                (* (I (= (var t) (var j)))
                   (I (<= 1 (var j))))))))"
    )
}

#[test]
fn sliding_window() {
    check_eq("
(- (sum (var w)
     (sum (var j)
          (* (* (var w) (* (I (<= 1 (var j)))
                           (I (<= (var j) (var t)))))
             (+ (* (I (= (var t) (var j)))
                   (rel v (var j) (var w)))
                (* (rel R (- (var t) 1) (var j) (var w))
                   (* (I (< (var j) (var t)))
                      (I (> (var t) 1))))))))
   (sum (var w)
     (sum (var j)
          (* (* (var w) (* (I (<= 1 (var j)))
                           (I (<= (var j) (- (var t) (var k))))))
             (+ (* (I (= (- (var t) (var k)) (var j)))
                   (rel v (var j) (var w)))
                (* (rel R (- (- (var t) (var k)) 1) (var j) (var w))
                   (* (I (< (var j) (- (var t) (var k))))
                      (I (> (- (var t) (var k)) 1)))))))))
", "
(+ (- (* (I (> (var t) 1))
         (sum (var j)
              (sum (var w)
                   (* (* (rel R (- (var t) 1) (var j) (var w))
                         (var w))
                      (* (I (<= (var j) (- (var t) 1)))
                         (I (<= 1 (var j))))))))
      (* (I (> (- (var t) (var k)) 1))
         (sum (var j)
              (sum (var w)
                   (* (* (rel R (- (- (var t) (var k)) 1) (var j) (var w))
                         (var w))
                      (* (I (<= (var j) (- (- (var t) (var k)) 1)))
                         (I (<= 1 (var j)))))))))
   (- (sum (var j)
           (sum (var w)
                (* (* (rel v (var j) (var w))
                      (var w))
                   (* (I (= (var t) (var j)))
                      (I (<= 1 (var j)))))))
      (sum (var j)
           (sum (var w)
                (* (* (rel v (var j) (var w))
                      (var w))
                   (* (I (= (- (var t) (var k)) (var j)))
                      (I (<= 1 (var j)))))))))
"
    )
}
