use std::collections::HashSet;

pub struct DataCleaner<T> {
    data: Vec<T>,
}

impl<T> DataCleaner<T> {
    pub fn new(data: Vec<T>) -> Self {
        DataCleaner { data }
    }

    pub fn remove_nulls(self) -> Self
    where
        T: PartialEq,
    {
        let filtered_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| *item != None)
            .collect();
        DataCleaner { data: filtered_data }
    }

    pub fn deduplicate(self) -> Self
    where
        T: Eq + std::hash::Hash + Clone,
    {
        let mut seen = HashSet::new();
        let unique_data: Vec<T> = self
            .data
            .into_iter()
            .filter(|item| seen.insert(item.clone()))
            .collect();
        DataCleaner { data: unique_data }
    }

    pub fn get_data(self) -> Vec<T> {
        self.data
    }
}

pub fn clean_dataset<T>(data: Vec<T>) -> Vec<T>
where
    T: Eq + std::hash::Hash + Clone + PartialEq,
{
    let cleaner = DataCleaner::new(data);
    cleaner
        .remove_nulls()
        .deduplicate()
        .get_data()
}