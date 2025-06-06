use std::collections::{
    hash_map::{Keys, Values},
    BTreeSet, HashMap,
};

use crate::gradescope::types::SubmissionTrait;

use super::Emission;

#[derive(Clone)]
pub struct EmissionsGroup<'a> {
    submission: &'a dyn SubmissionTrait,
    emissions_map: HashMap<String, Emission>,
}

impl<'a> EmissionsGroup<'a> {
    pub fn new(submission: &'a dyn SubmissionTrait, emissions: Vec<Emission>) -> Self {
        EmissionsGroup {
            submission,
            emissions_map: emissions
                .into_iter()
                .map(|e| (e.id().clone(), e))
                .collect::<HashMap<String, Emission>>(),
        }
    }

    pub fn len(&self) -> usize {
        self.emissions_map.len()
    }

    pub fn submission(&self) -> &dyn SubmissionTrait {
        self.submission
    }

    pub fn emissions_map(&self) -> &HashMap<String, Emission> {
        &self.emissions_map
    }

    pub fn emission_ids(&self) -> Keys<String, Emission> {
        self.emissions_map.keys()
    }

    pub fn emissions(&self) -> Values<String, Emission> {
        self.emissions_map.values()
    }

    pub fn matches(&self, other: &EmissionsGroup) -> bool {
        self.matches_on_ids(other, None)
    }

    pub fn matches_on_ids(
        &self,
        other: &EmissionsGroup,
        on_ids: Option<&BTreeSet<&String>>,
    ) -> bool {
        let on_ids: BTreeSet<_> = match on_ids {
            Some(ids) => ids.iter().cloned().collect::<BTreeSet<&String>>(),
            None => {
                let me = self.emission_ids().collect::<BTreeSet<&String>>();
                let them = other.emission_ids().collect::<BTreeSet<&String>>();
                me.union(&them).cloned().collect::<BTreeSet<&String>>()
            }
        };

        // Check if the emissions in `self` and `other` match on the specified IDs
        on_ids.iter().all(|id| {
            let mine = self.emissions_map.get(*id);
            let theirs = other.emissions_map.get(*id);
            match (mine, theirs) {
                (Some(mine), Some(theirs)) => mine.value() == theirs.value(),
                _ => false, // if at least one is None, it is not a match
            }
        })
    }
}
