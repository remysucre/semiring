use egg::*;
use std::collections::HashMap;

use crate::rewrites::*;
use crate::lang::*;
use crate::analysis::*;

// TODO use iteration data to compute this incrementally
pub fn solve_eqs(runner: &mut Runner<Semiring, SemiringAnalysis>) -> Result<(), String> {
    let mut fingerprints: HashMap<&Vec<i32>, Vec<Id>> = HashMap::new();
    for class in runner.egraph.classes() {
        if let Some(fp) = &class.data.fingerprint {
            fingerprints.entry(fp).or_insert(vec![]).push(class.id);
        }
    }
    let mut to_union = vec![];
    for matches in fingerprints.values() {
        for c_1 in matches.iter() {
            for c_2 in matches.iter() {
                let mut extractor = Extractor::new(&runner.egraph, AstSize);
                let (_, e_1) = extractor.find_best(*c_1);
                let (_, e_2) = extractor.find_best(*c_2);
                let local_runner = Runner::default()
                    .with_expr(&e_1)
                    .with_expr(&e_2)
                    .run(&lemmas());
                if local_runner.egraph.find(local_runner.roots[0])
                    == local_runner.egraph.find(local_runner.roots[1])
                {
                    to_union.push((*c_1, *c_2));
                }
            }
        }
    }
    for (c_1, c_2) in to_union {
        runner.egraph.union(c_1, c_2);
    }
    Ok(())
}

pub fn gen_rosette(lhs: &str, rhs: &str, fvs: &Vec<Id>) -> String {
    // FIXME this won't work, really needs to get the vars
    let fvs_s: Vec<String> = fvs.iter().map(|id| format!("v_{}", id)).collect();
    format!(
        "#lang rosette
         (define (I b) (if b 1 0))
         (define-symbolic {fvs} integer?)
         (if (unsat? (verify (assert (eq? {lhs} {rhs}))))
             (display \"true\")
             (display \"false\"))",
        fvs = &fvs_s.join(" "),
        lhs = lhs,
        rhs = rhs
    )
}
