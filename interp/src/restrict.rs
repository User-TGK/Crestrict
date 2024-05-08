use crate::evaluator::ScopeIdentifier;
use crate::memory::{Loc, SimpleLoc};

use std::collections::{BTreeSet, HashMap};

/// A Loc can be based on a specific block associated with a scope
pub type Base = (Loc, ScopeIdentifier);
// pub type SlBase = (SimpleLoc, ScopeIdentifier);
pub type RestrictMap = HashMap<SimpleLoc, RestrictState>;
pub type RestrictStack = Vec<RestrictStatus>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestrictState {
    OnlyRead(BTreeSet<BTreeSet<Base>>),

    /// Only accesses via bases are allowed
    Restricted(BTreeSet<Base>),

    Poison,
}

impl RestrictState {
    fn bases_joinable(family: &BTreeSet<BTreeSet<Base>>, set: &BTreeSet<Base>) -> bool {
        (family.is_empty() && set.is_empty())
            || (family.len() == 1 && family.first().unwrap() == set)
    }

    pub fn filter_bases(&self, scope: &ScopeIdentifier) -> Self {
        let res = match self {
            RestrictState::OnlyRead(bs_fam) => {
                let mut filtered = BTreeSet::new();
                for bas in bs_fam {
                    let bas_filtered = Self::filter_bases_helper(bas, scope);
                    filtered.insert(bas_filtered);
                }
                RestrictState::OnlyRead(filtered)
            }
            RestrictState::Restricted(bs) => {
                RestrictState::Restricted(Self::filter_bases_helper(bs, scope))
            }
            RestrictState::Poison => RestrictState::Poison,
        };

        res
    }

    /// To remove basedOn references that are invalid after a scope was finished.
    pub fn filter_bases_helper(bases: &BTreeSet<Base>, scope: &ScopeIdentifier) -> BTreeSet<Base> {
        let mut bases = bases.clone();
        bases.retain(|(_, base_scope)| base_scope != scope);

        bases
    }

    /// Assumes no conflict.
    pub fn join_restrict_state(&self, other: &Self) -> Self {
        match (self, other) {
            (RestrictState::Restricted(lhs_bases), RestrictState::Restricted(rhs_bases))
                if (lhs_bases == rhs_bases) =>
            {
                self.clone()
            }
            (RestrictState::OnlyRead(lhs_bases_fam), RestrictState::OnlyRead(rhs_bases_fam)) => {
                RestrictState::OnlyRead(lhs_bases_fam.union(rhs_bases_fam).cloned().collect())
            }
            (RestrictState::Restricted(lhs_bases), RestrictState::OnlyRead(rhs_bases_fam))
                if RestrictState::bases_joinable(rhs_bases_fam, lhs_bases) =>
            {
                RestrictState::Restricted(lhs_bases.clone())
            }
            (RestrictState::OnlyRead(lhs_bases_fam), RestrictState::Restricted(rhs_bases))
                if RestrictState::bases_joinable(lhs_bases_fam, rhs_bases) =>
            {
                RestrictState::Restricted(rhs_bases.clone())
            }
            (_, _) => RestrictState::Poison,
        }
    }
}

/// A structure to maintain restrict information for a specific scope,
/// based on the k-semantics <restrict> cell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestrictStatus {
    /// The scope for which the restrict data is maintained
    scope: ScopeIdentifier,

    /// Restrict states of the memory cells
    pub states: RestrictMap,
}

impl RestrictStatus {
    pub fn new(scope: ScopeIdentifier) -> Self {
        Self {
            scope,
            states: HashMap::new(),
        }
    }

    pub fn get_scope(&self) -> &ScopeIdentifier {
        &self.scope
    }
}
