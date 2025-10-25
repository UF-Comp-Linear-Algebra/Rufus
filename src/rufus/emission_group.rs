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

    pub fn emission_ids(&self) -> Keys<'_, String, Emission> {
        self.emissions_map.keys()
    }

    pub fn emissions(&self) -> Values<'_, String, Emission> {
        self.emissions_map.values()
    }

    pub fn matches(&self, other: &EmissionsGroup) -> bool {
        self.matches_on_ids(other, None, false)
    }

    pub fn matches_on_ids(
        &self,
        other: &EmissionsGroup,
        on_ids: Option<&BTreeSet<&String>>,
        exact: bool,
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
        let matches_on = on_ids.iter().all(|id| {
            let mine = self.emissions_map.get(*id);
            let theirs = other.emissions_map.get(*id);
            match (mine, theirs) {
                (Some(mine), Some(theirs)) => mine.value() == theirs.value(),
                _ => false, // if at least one is None, it is not a match
            }
        });

        if !matches_on {
            return false;
        }

        if exact {
            // For all ids present in both groups but not in on_ids, ensure the values do NOT match
            let self_ids = self.emission_ids().collect::<BTreeSet<&String>>();
            let other_ids = other.emission_ids().collect::<BTreeSet<&String>>();
            let common_ids = self_ids
                .intersection(&other_ids)
                .filter(|id| !on_ids.contains(*id))
                .cloned()
                .collect::<BTreeSet<&String>>();
            for id in common_ids {
                let mine = self.emissions_map.get(id);
                let theirs = other.emissions_map.get(id);
                if let (Some(mine), Some(theirs)) = (mine, theirs) {
                    if mine.value() == theirs.value() {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rufus::Emission;
    use std::collections::BTreeSet;

    struct DummySubmission;
    impl crate::gradescope::types::SubmissionTrait for DummySubmission {
        fn submitters(&self) -> &Vec<crate::gradescope::types::Submitter> {
            static S: Vec<crate::gradescope::types::Submitter> = Vec::new();
            &S
        }
        fn created_at(&self) -> &String {
            static S: String = String::new();
            &S
        }
        fn score(&self) -> &crate::gradescope::types::Score {
            static S: crate::gradescope::types::Score = 0.0;
            &S
        }
        fn status(&self) -> &String {
            static S: String = String::new();
            &S
        }
        fn results(&self) -> &Option<crate::gradescope::types::Results> {
            static S: Option<crate::gradescope::types::Results> = None;
            &S
        }
    }

    fn make_emission(id: &str, value: &str) -> Emission {
        Emission {
            id: id.to_string(),
            value: value.to_string(),
        }
    }

    fn make_group<'a>(subs: &'a DummySubmission, pairs: &[(&str, &str)]) -> EmissionsGroup<'a> {
        let emissions = pairs
            .iter()
            .map(|(id, val)| make_emission(id, val))
            .collect();
        EmissionsGroup::new(subs, emissions)
    }

    #[test]
    fn test_matches_on_ids_non_exact() {
        let sub = DummySubmission;
        let g1 = make_group(&sub, &[("a", "1"), ("b", "2"), ("c", "3")]);
        let g2 = make_group(&sub, &[("a", "1"), ("b", "2"), ("c", "DIFF")]);
        let g3 = make_group(&sub, &[("a", "1"), ("b", "DIFF"), ("c", "3")]);
        let g4 = make_group(&sub, &[("a", "1"), ("b", "2"), ("c", "3")]);

        let all_ids: Vec<&String> = [
            g1.emissions_map(),
            g2.emissions_map(),
            g3.emissions_map(),
            g4.emissions_map(),
        ]
        .iter()
        .flat_map(|m| m.keys())
        .collect();
        let on_ids: BTreeSet<&String> = all_ids
            .iter()
            .filter(|id| id.as_str() == "a" || id.as_str() == "b")
            .map(|id| *id)
            .collect();

        // g1 and g2 match on a, b (non-exact)
        assert!(g1.matches_on_ids(&g2, Some(&on_ids), false));
        // g1 and g3 do not match on b
        assert!(!g1.matches_on_ids(&g3, Some(&on_ids), false));
        // g1 and g4 match on all
        assert!(g1.matches_on_ids(&g4, Some(&on_ids), false));
    }

    #[test]
    fn test_matches_on_ids_exact() {
        let sub = DummySubmission;
        let g1 = make_group(&sub, &[("a", "1"), ("b", "2"), ("c", "3")]);
        let g2 = make_group(&sub, &[("a", "1"), ("b", "2"), ("c", "DIFF")]);
        let g3 = make_group(&sub, &[("a", "1"), ("b", "2"), ("c", "3")]);
        let g4 = make_group(&sub, &[("a", "1"), ("b", "2"), ("d", "3")]);

        let all_ids: Vec<&String> = [
            g1.emissions_map(),
            g2.emissions_map(),
            g3.emissions_map(),
            g4.emissions_map(),
        ]
        .iter()
        .flat_map(|m| m.keys())
        .collect();
        let on_ids: BTreeSet<&String> = all_ids
            .iter()
            .filter(|id| id.as_str() == "a" || id.as_str() == "b")
            .map(|id| *id)
            .collect();

        // g1 and g2 match on a, b, but c differs, so exact
        assert!(g1.matches_on_ids(&g2, Some(&on_ids), true));
        // g1 and g3 match on a, b, but also match on c, so not exact
        assert!(!g1.matches_on_ids(&g3, Some(&on_ids), true));
        // g1 and g4 match on a, b, and have no other common ids, so exact
        assert!(g1.matches_on_ids(&g4, Some(&on_ids), true));
    }
}
