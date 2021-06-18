// Midpoint Iterator
// Use like so, for an initial iterator that samples 10 values form the inner vec:
// ```
// let original_data = [0, 100];
// let iterator = MidpointIterator::new(original_data, 10)
// assert_eq!(iterator.len(), 10);
// let lod_1 = iterator.next_lod();
// assert_eq!(lod_1.len(), 19);
// let lod_2 = lod_1.next_lod();
// assert_eq!(lod_1.len(), 19 + 18);


pub trait LodMidpointIterator<'a, T>: Iterator {
    fn next_lod(&self) -> Option<&dyn LodMidpointIterator<T, Item = T>>;
    fn at_max_lod(&self) -> bool;
    fn indices(&self) -> Vec<usize>;
}

#[derive(Clone)]
struct MidpointIterator<T> 
where T: Clone {
    inner: Vec<T>,
    indices: Vec<usize>,
    current_index: usize,
}

impl <T> MidpointIterator<T> 
where T: Clone {
    pub fn new(inner: Vec<T>, first_lod: usize) -> Self {
        let step_size = (inner.len() as f32 / (first_lod as f32 - 1.)).round() as usize;
        let mut indices: Vec<usize> = inner
            .iter()
            .step_by(step_size)
            .enumerate()
            .map(|(i, _)| i * step_size)
            .collect();
        if let None = indices.iter().find(|&i| *i == inner.len() - 1) {
            indices.push(inner.len() - 1);
        }
        Self {
            inner,
            indices,
            current_index: 0,
        }
    }

    pub fn next_lod_from_midpoint_iterator(from: &MidpointIterator<T>) -> Option<MidpointIterator<T>> {
        let new_indices: Vec<usize> = from
            .indices()
            .iter()
            .enumerate()
            .flat_map(|(i, &actual_index)| {
                // if we are at the last element of the original iterator, we dont need to get the midpoint (there is none)
                if let Some(&next) = from.indices.get(i+1) {
                    let midpoint: usize = (actual_index + next) / 2;
                    // the indices are adjacent (there is no gap)
                    if midpoint == actual_index || midpoint == next {
                        return vec!(actual_index)
                    }
                    return vec!(actual_index, midpoint);
                }
                vec!(actual_index)
            })
            .collect();

        let new_index = *from.indices().get(from.current_index).unwrap();
        let new_index = new_indices
            .iter()
            .position(|&idx| new_index == idx)
            .unwrap();

        Some(MidpointIterator {
            inner: from.inner.clone(),
            indices: new_indices,
            current_index: new_index,
        })
    }

    pub fn next_lod(&self) -> Option<MidpointIterator<T>> {
        Self::next_lod_from_midpoint_iterator(self)
    }

    pub fn indices(&self) -> Vec<usize> {
        self.indices.clone()
    }

    pub fn len(&self) -> usize {
        self.indices().len()
    }

}

impl <T> Iterator for MidpointIterator<T> 
where T: Clone {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.indices.get(self.current_index);
        if let Some(index) = index {
            let next = self.inner.get(*index);
                if let Some(next) = next {
                    self.current_index = self.current_index + 1;
                    return Some((*next).clone())
                }
            }

        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three() {
        let three = vec![1, 2, 3];
        let mpi = MidpointIterator::new(three, 2);
        assert_eq!(mpi.len(), 2);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 3);
    }
    
    #[test]
    fn test_one_hundred() {
        let hundred = [0; 100];
        let mpi = MidpointIterator::new(hundred.into(), 10);
        assert_eq!(mpi.len(), 10);
        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19 + 18);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 37 + 36 );

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 100);
    }

    #[test]
    fn test_one_thousand() {
        let hundred = [0; 1000];
        let mpi = MidpointIterator::new(hundred.into(), 10);
        assert_eq!(mpi.len(), 10);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 19 + 18);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 37 + 36 );

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 73 + 72);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 145 + 144);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 289 + 288);

        let mpi = mpi.next_lod().unwrap();
        assert_eq!(mpi.len(), 1000);
    }


}