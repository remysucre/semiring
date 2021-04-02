use egg::*;
use std::collections::HashSet;

use crate::lang::*;
use crate::EGraph;

#[derive(Default, Clone)]
pub struct SemiringAnalysis;

// Metadata for each class
#[derive(Debug, PartialEq, Eq)]
pub struct Data {
    // Set of free variables by their class ID
    pub free: HashSet<Id>,
    pub constant: Option<Semiring>,
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
                    assert_eq!(&c_from, c_to, "merging classes with different constants");
                } else {
                    to.constant = Some(c_from);
                }
            }
            true
        }
    }

    fn make(egraph: &EGraph, enode: &Semiring) -> Data {
        let fvs = |i: &Id| egraph[*i].data.free.iter().copied();
        let mut free = HashSet::default();
        match enode {
            Semiring::Var(v) => {
                free.insert(*v);
            }
            Semiring::Let([v, a, b]) => {
                free.extend(fvs(b));
                // NOTE only do this if v free in b?
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
        Data { free, constant }
    }

    fn modify(egraph: &mut EGraph, id: Id) {
        if let Some(c) = egraph[id].data.constant.clone() {
            let const_id = egraph.add(c);
            egraph.union(id, const_id);
        }
    }
    fn pre_union(_egraph: &egg::EGraph<Semiring, Self>, _id1: Id, _id2: Id) {}
}

fn eval(egraph: &EGraph, enode: &Semiring) -> Option<Semiring> {
    let x = |i: &Id| egraph[*i].data.constant.clone();
    match enode {
        Semiring::Num(n) => Some(Semiring::Num(*n)),
        Semiring::Add([a, b]) => Some(Semiring::Num(x(a)?.num()? + x(b)?.num()?)),
        Semiring::Min([a, b]) => Some(Semiring::Num(x(a)?.num()? - x(b)?.num()?)),
        Semiring::Mul([a, b]) => Some(Semiring::Num(x(a)?.num()? * x(b)?.num()?)),
        _ => None,
    }
}
