use std::cmp::Ord;

pub struct SortedVec<T: Ord> {
    vec: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn from_vec(mut vec: Vec<T>) -> Self {
        vec.sort();

        Self { vec }
    }

    /// Find the rank of an element in `O(log(n) + c)`, where `c` is the
    /// maximum amount of duplicates. In an array with few duplicates `c <= O(1)`.
    pub fn rank(&self, key: &T) -> usize {
        if self.vec.is_empty() {
            return 0;
        }

        let mut l = 0;
        let mut r = self.vec.len() - 1;
        let mut mid = 0;

        while l < r {
            mid = (l + r) / 2;

            if &self.vec[mid] < key {
                l = mid + 1;
            } else if &self.vec[mid] > key {
                r = mid;
            } else {
                //found, return
                break;
            }
        }

        // Find actual rightmost element
        let offset_last_le = self.vec[mid..].iter().take_while(|&num| num <= key).count();

        mid + offset_last_le
    }

    /// Insert element in `O(log n)`.
    /// Element is inserted *after* every other less or equal element.
    pub fn insert(&mut self, key: T) {
        let index = self.rank(&key);

        self.vec.insert(index, key);
    }

    /// Position of the last element equal to key, or none
    pub fn position(&self, key: &T) -> Option<usize> {
        let pos = self.rank(key);

        if pos != 0 && &self.vec[pos - 1] == key {
            Some(pos - 1)
        } else {
            None
        }
    }

    pub fn get_le(&self, key: &T) -> &[T] {
        let index = self.rank(key);

        &self.vec[0..index]
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn remove_le(&mut self, key: &T) {
        self.vec.retain(|elem| elem > key); // only keep elements strictly greater than key
    }
}

impl<T: Ord> AsRef<[T]> for SortedVec<T> {
    fn as_ref(&self) -> &[T] {
        &self.vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length() {
        let mut vec = vec![];
        for i in 0..50 {
            vec.push(i);

            let sorted_vec = SortedVec::from_vec(vec.clone());

            assert_eq!(vec.len(), sorted_vec.len());
        }
    }

    #[test]
    fn rank() {
        let empty = SortedVec::from_vec(vec![]);
        assert_eq!(0, empty.rank(&10));

        let short = SortedVec::from_vec(vec![1]);
        assert_eq!(1, short.rank(&1));
        assert_eq!(1, short.rank(&2));
        assert_eq!(0, short.rank(&0));

        let cortino = SortedVec::from_vec(vec![1, 3]);
        assert_eq!(0, cortino.rank(&0));
        assert_eq!(1, cortino.rank(&1));
        assert_eq!(1, cortino.rank(&2));
        assert_eq!(2, cortino.rank(&3));
        assert_eq!(2, cortino.rank(&4));

        let vec1 = SortedVec::from_vec(vec![1, 2, 3, 5, 6, 7, 8]);
        assert_eq!(0, vec1.rank(&0));
        assert_eq!(1, vec1.rank(&1));
        assert_eq!(2, vec1.rank(&2));
        assert_eq!(3, vec1.rank(&3));
        assert_eq!(3, vec1.rank(&4));
        assert_eq!(4, vec1.rank(&5));
        assert_eq!(5, vec1.rank(&6));
        assert_eq!(6, vec1.rank(&7));
        assert_eq!(7, vec1.rank(&8));
        assert_eq!(7, vec1.rank(&9));

        let multiple = SortedVec::from_vec(vec![1, 1, 1, 2, 2, 2, 3]);
        assert_eq!(3, multiple.rank(&1));
        assert_eq!(6, multiple.rank(&2));
    }

    #[test]
    fn insert() {
        let mut vec = SortedVec::new();
        vec.insert(1);
        assert_eq!(&[1], vec.get_le(&1));
        assert_eq!(Some(0), vec.position(&1));
    }

    #[test]
    fn position() {
        let vec = SortedVec::from_vec(vec![1, 5, 4]);
        assert_eq!(Some(0), vec.position(&1));
        assert_eq!(Some(1), vec.position(&4));
        assert_eq!(Some(2), vec.position(&5));
        assert_eq!(None, vec.position(&3));
    }

    #[test]
    fn remove() {
        let mut vec = SortedVec::from_vec(vec![9, 8, 7, 6, 5, 4, 3, 2, 1]);
        vec.remove_le(&6);
        assert_eq!(&[7, 8, 9], vec.as_ref());
    }
}
