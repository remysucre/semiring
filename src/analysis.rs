use egg::*;
use rand::prelude::*;
use std::collections::HashSet;

use crate::lang::*;
use crate::EGraph;

// Initial length of fingerprint vector
const FP_LEN: usize = 32;

#[derive(Default, Clone)]
pub struct SemiringAnalysis;

// Metadata for each class
#[derive(Debug, PartialEq, Eq)]
pub struct Data {
    // Set of free variables by their class ID
    pub free: HashSet<Id>,
    pub constant: Option<Semiring>,
    pub fingerprint: Option<Vec<i32>>,
}

impl Analysis<Semiring> for SemiringAnalysis {
    type Data = Data;
    fn merge(&self, to: &mut Data, from: Data) -> bool {
        if *to == from {
            false
        } else {
            // The free vars may differ due to constant folding
            to.free.retain(|i| from.free.contains(i));

            // Merged classes must agree on the constant value,
            // if both have one.
            if let Some(c_from) = from.constant {
                if let Some(c_to) = &to.constant {
                    assert_eq!(
                        &c_from, c_to,
                        "merging classes with different constants"
                    );
                } else {
                    to.constant = Some(c_from);
                }
            }
            // TODO Not sure how to merge fingerprints
            if let Some(fp_from) = from.fingerprint {
                if let Some(_fp_to) = &to.fingerprint {
                    // Do nothing for now
                } else {
                    to.fingerprint = Some(fp_from);
                }
            }
            true
        }
    }

    fn make(egraph: &EGraph, enode: &Semiring) -> Data {
        let fvs = |i: &Id| egraph[*i].data.free.iter().cloned();
        let mut free = HashSet::default();
        match enode {
            Semiring::Var(v) => {
                free.insert(*v);
            }
            Semiring::Let([v, a, b]) => {
                free.extend(fvs(b));
                free.remove(v);
                free.extend(fvs(a));
            }
            Semiring::Sum([v, a]) => {
                free.extend(fvs(a));
                free.remove(v);
            }
            _ => enode.for_each(|c| free.extend(&egraph[c].data.free)),
        }
        let constant = eval(egraph, enode);
        let fingerprint = fingerprint(egraph, enode);
        Data {
            free,
            constant,
            fingerprint,
        }
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        if let Some(c) = egraph[id].data.constant.clone() {
            let const_id = egraph.add(c);
            egraph.union(id, const_id);
        }
    }
}

fn combine_fp<F>(x: &Option<Vec<i32>>, y: &Option<Vec<i32>>, f: F) -> Option<Vec<i32>>
where
    F: Fn((&i32, &i32)) -> i32,
{
    let v_x = x.as_deref()?;
    let v_y = y.as_deref()?;
    assert_eq!(v_x.len(), v_y.len(), "fingerprint lengths differ");
    Some(v_x.iter().zip(v_y.iter()).map(f).collect())
}

fn fingerprint(egraph: &EGraph, enode: &Semiring) -> Option<Vec<i32>> {
    let fp = |i: &Id| &egraph[*i].data.fingerprint;
    match enode {
        Semiring::Num(n) => Some((0..FP_LEN).map(|_| *n).collect()),
        Semiring::Var(_v) => Some((0..FP_LEN).map(|_| thread_rng().gen()).collect()),
        Semiring::Add([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| x + y),
        Semiring::Min([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| x - y),
        Semiring::Mul([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| x * y),
        Semiring::Ind(b) => fp(b).clone(),
        Semiring::Lt([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| if x < y { 1 } else { 0 }),
        Semiring::Leq([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| if x <= y { 1 } else { 0 }),
        Semiring::Eq([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| if x == y { 1 } else { 0 }),
        Semiring::Gt([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| if x > y { 1 } else { 0 }),
        Semiring::Geq([a, b]) => combine_fp(fp(a), fp(b), |(x, y)| if x >= y { 1 } else { 0 }),
        _ => None,
    }
}

fn eval(egraph: &EGraph, enode: &Semiring) -> Option<Semiring> {
    let x = |i: &Id| egraph[*i].data.constant.clone();
    match enode {
        Semiring::Num(_) => Some(enode.clone()),
        Semiring::Add([a, b]) => Some(Semiring::Num(x(a)?.num()? + x(b)?.num()?)),
        Semiring::Min([a, b]) => Some(Semiring::Num(x(a)?.num()? - x(b)?.num()?)),
        Semiring::Mul([a, b]) => Some(Semiring::Num(x(a)?.num()? * x(b)?.num()?)),
        _ => None,
    }
}
