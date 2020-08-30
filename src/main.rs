use egg::*;
use semiring::*;

fn main() {
        let r = Runner::<Semiring, BindAnalysis>::default()
                .with_expr(&"(+ (+ (var a) (lit 0)) (var a))".parse().unwrap())
                .run(&normalizing_rules());
        r.egraph.dot().to_png("normalized.png").unwrap();
}
