// Midpoint Iterator
// Use like so, for an initial iterator that samples 10 values from the inner vec:
// ```
// let original_data = [0, 100];
// let iterator = MidpointIterator::new(original_data, 10)
// assert_eq!(iterator.len(), 10);
// let lod_1 = iterator.next_lod();
// assert_eq!(lod_1.len(), 19);
// let lod_2 = lod_1.next_lod();
// assert_eq!(lod_1.len(), 19 + 18);

#[derive(Clone, Debug, PartialEq)]
pub struct MidpointIterator<T>
where
    T: Clone,
{
    inner: Vec<T>,
    indices: Vec<usize>,
    current_index: usize,
    first_lod: usize,
}

impl<T> MidpointIterator<T>
where
    T: Clone + PartialEq + AsRef<str>,
{
    pub fn new(
        inner: Vec<T>,
        first_lod: usize,
    ) -> Self {
        let mut me = Self {
            inner,
            indices: Vec::new(),
            current_index: 0,
            first_lod,
        };
        me.initialize();
        me
    }

    pub fn clear_indices(&mut self) {
        self.indices.clear();
    }

    pub fn clear(&mut self) {
        self.clear_indices();
        self.inner.clear();
    }

    pub fn initialize(&mut self) {
        let step_size = (self.inner.len() as f32
            / (self.first_lod as f32 - 1.)
                .min(self.inner.len() as f32)
                .round()) as usize;
        let step_size = step_size.max(1);
        let mut indices: Vec<usize> = self
            .inner
            .iter()
            .step_by(step_size)
            .enumerate()
            .map(|(i, _)| i * step_size)
            .collect();
        if let None = indices.iter().find(|&i| *i == self.inner.len() - 1) {
            if self.inner.len() > 0 {
                indices.push(self.inner.len() - 1);
            }
        }
        self.indices = indices;
    }

    // pub fn reinitialize(&mut self) {
    //     self.clear_indices();
    //     self.initialize();
    // }

    pub fn next_lod_from_midpoint_iterator(
        from: &MidpointIterator<T>
    ) -> Option<MidpointIterator<T>> {
        let new_indices: Vec<usize> = from
            .indices()
            .iter()
            .enumerate()
            .flat_map(|(i, &actual_index)| {
                // if we are at the last element of the original iterator, we dont need to get the midpoint (there is none)
                if let Some(&next) = from.indices.get(i + 1) {
                    let midpoint: usize = (actual_index + next) / 2;
                    // the indices are adjacent (there is no gap)
                    if midpoint == actual_index || midpoint == next {
                        return vec![actual_index];
                    }
                    return vec![actual_index, midpoint];
                }
                vec![actual_index]
            })
            .collect();

        let mut new_index = 0;
        if let Some(found_index) = from.indices().get(from.current_index) {
            new_index = new_indices
                .iter()
                .position(|&idx| *found_index == idx)
                .unwrap();
        }

        Some(MidpointIterator {
            inner: from.inner.clone(),
            indices: new_indices,
            current_index: new_index,
            first_lod: from.first_lod,
        })
    }

    pub fn get_lods(&self) -> Vec<usize> {
        let mut current_lod = Some(self.clone());
        let mut lods = Vec::new();
        if self.len() > 0 {
            lods.push(self.len());
            let mut have_lod = true;
            while have_lod {
                let next_lod = current_lod.clone().unwrap().next_lod();
                if let Some(next_lod) = next_lod {
                    lods.push(next_lod.len());
                    current_lod = Some(next_lod);
                } else {
                    have_lod = false;
                }
            }
        }
        lods
    }

    pub fn is_saturated(&self) -> bool {
        self.indices.len() == self.inner.len()
    }

    pub fn next_lod(&self) -> Option<MidpointIterator<T>> {
        if self.is_saturated() {
            return None;
        }
        Self::next_lod_from_midpoint_iterator(self)
    }

    pub fn indices(&self) -> Vec<usize> {
        self.indices.clone()
    }

    pub fn len(&self) -> usize {
        self.indices().len()
    }

    pub fn _current_index(&self) -> usize {
        self.current_index
    }

    pub fn push(
        &mut self,
        item: T,
    ) {
        self.inner.push(item)
    }

    pub fn contains(
        &self,
        item: T,
    ) -> bool {
        self.inner.iter().any(|f| f == &item)
    }

    pub fn sort(&mut self) {
        alphanumeric_sort::sort_str_slice(&mut self.inner);
    }
}

impl<T> Iterator for MidpointIterator<T>
where
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices.get(self.current_index);
        if let Some(index) = index {
            let next = self.inner.get(*index);
            if let Some(next) = next {
                self.current_index = self.current_index + 1;
                return Some((*next).clone());
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_length() {
        let zero: Vec<String> = Vec::new();
        let mpi = MidpointIterator::new(zero, 2);
        assert_eq!(mpi.len(), 0);
        assert_eq!(mpi.get_lods().len(), 0);
    }

    #[test]
    fn test_three() {
        let three = vec!["a", "b", "c"];
        let mpi = MidpointIterator::new(three, 2);

        assert_eq!(mpi.len(), 2);
        assert_eq!(mpi.get_lods(), vec![2, 3]);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 3);

        if let Some(_) = mpi.next_lod() {
            assert!(false)
        }
    }

    #[test]
    fn test_one_hundred() {
        let hundred = ["a"; 100];
        let mpi = MidpointIterator::new(hundred.into(), 10);

        assert_eq!(mpi.len(), 10);
        assert_eq!(mpi.get_lods(), [10, 19, 19 + 18, 37 + 36, 100]);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19 + 18);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 37 + 36);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 100);

        if let Some(_) = mpi.next_lod() {
            assert!(false)
        }
    }

    #[test]
    fn test_one_thousand() {
        let hundred = ["a"; 1000];
        let mpi = MidpointIterator::new(hundred.into(), 10);
        assert_eq!(mpi.len(), 10);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19 + 18);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 37 + 36);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 73 + 72);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 145 + 144);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 289 + 288);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 1000);
    }

    #[test]
    fn test_initial_lod_greater_than_available() {
        let ten = ["a"; 10];
        let mpi = MidpointIterator::new(ten.into(), 20);
        assert_eq!(mpi.len(), 10);

        assert_eq!(None, mpi.next_lod());
    }

    // #[test]
    // fn test_adding_new_elements_and_initialize_works() {
    //     let ten = ["a"; 10];
    //     let mut mpi = MidpointIterator::new(ten.into(), 20);
    //     mpi.push("b");
    //     mpi.reinitialize();
    //     assert_eq!(mpi.len(), 11);
    // }
}
