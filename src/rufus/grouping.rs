use std::collections::BTreeSet;

use crate::rufus::EmissionsGroup;

pub struct Grouping<'a> {
    on_ids: BTreeSet<&'a String>,
    groups: Vec<&'a EmissionsGroup<'a>>,
}

impl<'a> Grouping<'a> {
    pub fn new(on_ids: BTreeSet<&'a String>, groups: Vec<&'a EmissionsGroup<'a>>) -> Self {
        Grouping { on_ids, groups }
    }

    pub fn on_ids(&self) -> &BTreeSet<&'a String> {
        &self.on_ids
    }

    pub fn groups(&self) -> &Vec<&'a EmissionsGroup<'a>> {
        &self.groups
    }

    pub fn add_group(&mut self, group: &'a EmissionsGroup<'a>) {
        self.groups.push(group);
    }

    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn matches_group_on_ids(
        &self,
        group: &EmissionsGroup<'a>,
        on_ids: Option<&BTreeSet<&String>>,
    ) -> bool {
        on_ids.is_none_or(|on_ids| on_ids.is_subset(&self.on_ids))
            && self
                .groups
                .get(0)
                .is_some_and(|g| group.matches_on_ids(g, on_ids))
    }
}
