use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

/// A History Stack.
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct LocationStack {
    prev: Vec<String>,
    next: VecDeque<String>,
    current: String,
}

impl LocationStack {
    fn current(&self) -> String {
        self.current.clone()
    }

    fn len(&self) -> usize {
        self.prev.len() + self.next.len() + 1
    }

    fn prev_len(&self) -> usize {
        self.prev.len()
    }

    fn next_len(&self) -> usize {
        self.next.len()
    }

    fn go(&mut self, delta: isize) {
        match delta.cmp(&0) {
            // Go forward.
            Ordering::Greater => {
                for _i in 0..delta {
                    if let Some(mut m) = self.next.pop_front() {
                        std::mem::swap(&mut m, &mut self.current);

                        self.prev.push(m);
                    }
                }
            }
            // Go backward.
            Ordering::Less => {
                for _i in 0..-delta {
                    if let Some(mut m) = self.prev.pop() {
                        std::mem::swap(&mut m, &mut self.current);

                        self.next.push_front(m);
                    }
                }
            }
            // Do nothing.
            Ordering::Equal => {}
        }
    }

    fn push(&mut self, mut location: String) {
        std::mem::swap(&mut location, &mut self.current);

        self.prev.push(location);
        // When a history is pushed, we clear all forward states.
        self.next.clear();
    }

    fn replace(&mut self, location: String) {
        self.current = location;
    }
}

/// A [`History`] that is implemented with in memory history stack and is usable in most targets.
#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct MemoryHistory {
    inner: Rc<RefCell<LocationStack>>,
}

impl PartialEq for MemoryHistory {
    fn eq(&self, rhs: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &rhs.inner)
    }
}

impl fmt::Debug for MemoryHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryHistory").finish()
    }
}

impl MemoryHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`MemoryHistory`] with entires.
    pub fn with_entries<'a>(entries: impl IntoIterator<Item = impl Into<Cow<'a, str>>>) -> Self {
        let self_ = Self::new();

        for (index, entry) in entries.into_iter().enumerate() {
            if index == 0 {
                self_.replace(entry);
            } else {
                self_.push(entry);
            }
        }

        self_
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    pub fn can_go_back(&self) -> bool {
        self.inner.borrow().prev_len() > 1
    }

    pub fn can_go_forward(&self) -> bool {
        self.inner.borrow().next_len() > 0
    }

    pub fn go(&self, delta: isize) {
        self.inner.borrow_mut().go(delta)
    }

    pub fn push<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        let location = route.to_string().into();

        self.inner.borrow_mut().push(location);
    }

    pub fn replace<'a>(&self, route: impl Into<Cow<'a, str>>) {
        let route = route.into();

        let location = route.to_string().into();

        self.inner.borrow_mut().replace(location);
    }

    pub fn location(&self) -> String {
        self.inner.borrow().current()
    }

    pub fn clear(&self) {
        self.inner.take();
    }
}
