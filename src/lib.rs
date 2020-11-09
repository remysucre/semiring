pub mod analysis;
pub mod lang;
pub mod rewrites;
pub mod smt;

pub type EGraph = egg::EGraph<lang::Semiring, analysis::SemiringAnalysis>;
