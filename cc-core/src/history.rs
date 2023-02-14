use std::ops::{Index, IndexMut};

/// History is a vector with support for go forward and go back
pub struct History<T> {
    vec: Vec<T>,
    pub index: usize,
}

impl<T> History<T> {
    /// Creates a new history
    pub const fn new() -> History<T> {
        History {
            vec: Vec::new(),
            index: 0usize,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns whether you can go backwards
    #[inline]
    pub const fn can_go_backward(&self) -> bool {
        self.index > 1usize
    }

    /// Returns whether you can go forwards
    #[inline]
    pub fn can_go_forward(&self) -> bool {
        0usize < self.vec.len() && self.index < self.len()
    }

    /// Go forward
    pub fn go_forward(&mut self) {
        self.go_multi_forward(1);
    }

    /// Go `count` times forward
    #[inline]
    pub fn go_multi_forward(&mut self, count: usize) {
        self.index += count;
    }

    /// Go backwards
    pub fn go_backward(&mut self) {
        self.go_multi_backward(1);
    }

    /// Go `count` times backwards
    #[inline]
    pub fn go_multi_backward(&mut self, count: usize) {
        self.index -= count;
    }

    /// Pushes a new element to history
    pub fn push(&mut self, element: T) {
        let vec_i = self.vec.len();
        if self.index != vec_i {
            let diff: usize = vec_i - self.index;
            for _ in 0..diff {
                self.pop();
            }
        }
        self.vec.push(element);
        self.index += 1;
    }

    /// Removes the last element from a vector and returns it, or `None` if it is empty
    pub fn pop(&mut self) -> Option<T> {
        self.index -= 1;
        self.vec.pop()
    }

    /// Returns the current element
    pub fn get_current(&self) -> &T {
        &self.vec[self.index - 1]
    }

    /// Returns the length of the history
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<T> Index<usize> for History<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<T> IndexMut<usize> for History<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_order() {
        let mut history: History<&str> = History::new();
        history.push("Element 1");
        history.push("Element 2");
        history.push("Element 3");
        history.go_backward();
        assert_eq!(*history.get_current(), "Element 2");
    }

    #[test]
    fn test_multi_back_for_switch() {
        let mut history: History<usize> = History::new();
        history.push(3);
        history.push(6);
        history.push(7);
        history.go_backward();
        history.push(1);
        history.push(9);
        history.go_backward();
        assert_eq!(*history.get_current(), 3usize);
    }
}
