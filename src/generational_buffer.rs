use std::{
    fmt,
    marker::PhantomData,
};

/// A handle that combines an index with a generation counter.
///
/// The handle is typed according to the type of data it refers to,
/// but doesn't hold it.
#[derive(Debug)]
pub struct Handle<T> {
    index: usize,
    generation: u32,
    phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    fn new(index: usize, generation: u32) -> Self {
        Self {
            index,
            generation,
            phantom: PhantomData,
        }
    }
}

/// A generic append-only circular buffer with generational IDs
///
/// Inserting returns a `Handle` that can be used to access the value later,
/// checking the item hasn't been replaced in the meantime.
pub struct GenerationalBuffer<T> {
    entries: Vec<T>,
    max_capacity: usize,
    next_index: usize,
    current_generation: u32,
}

impl<T> GenerationalBuffer<T> {
    /// Creates a new generational buffer with the specified capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_capacity,
            next_index: 0,
            current_generation: 0,
        }
    }

    /// Returns the maximum capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }

    /// Returns the current number of entries in the buffer
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns true if the buffer has reached its maximum capacity
    pub fn is_full(&self) -> bool {
        self.entries.len() == self.max_capacity
    }

    /// Inserts a value into the buffer and returns a handle to it.
    ///
    /// This removes the oldest entry if the buffer is full.
    pub fn push(&mut self, value: T) -> Handle<T> {
        let index = self.next_index;
        let generation = self.current_generation;

        if self.entries.len() < self.max_capacity {
            // Buffer is not full yet, just append
            self.entries.push(value);
        } else {
            // Buffer is full, overwrite the oldest entry
            self.entries[index] = value;
        }

        // Create handle with current generation
        let handle = Handle::new(index, generation);

        // Advance to the next position
        self.next_index = (self.next_index + 1) % self.max_capacity;

        // If we've wrapped around, increment the generation
        if self.next_index == 0 {
            self.current_generation = self.current_generation.wrapping_add(1);
        }

        handle
    }

    /// Gets a reference to the value associated with the handle
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        if handle.index >= self.entries.len() {
            return None;
        }

        // Calculate the generation that should be at this index
        let expected_generation = self.calculate_generation_at_index(handle.index);

        // Check if the generation matches
        if handle.generation == expected_generation {
            Some(&self.entries[handle.index])
        } else {
            None
        }
    }

    /// Gets a mutable reference to the value associated with the handle
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        if handle.index >= self.entries.len() {
            return None;
        }

        // Calculate the generation that should be at this index
        let expected_generation = self.calculate_generation_at_index(handle.index);

        // Check if the generation matches
        if handle.generation == expected_generation {
            Some(&mut self.entries[handle.index])
        } else {
            None
        }
    }

    /// Checks if a handle is still valid (points to existing data)
    pub fn is_valid(&self, handle: Handle<T>) -> bool {
        if handle.index >= self.entries.len() {
            return false;
        }

        let expected_generation = self.calculate_generation_at_index(handle.index);
        handle.generation == expected_generation
    }

    /// Returns an iterator over all entries with their handles,
    ///  in no particular order
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> {
        self.entries.iter().enumerate().map(|(i, value)| {
            let generation = self.calculate_generation_at_index(i);
            (Handle::new(i, generation), value)
        })
    }

    /// Returns an iterator over all entries, in no particular order
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.entries.iter()
    }

    /// Returns an iterator over all valid handles, in no particular order
    pub fn handles(&self) -> impl Iterator<Item = Handle<T>> + '_ {
        (0..self.entries.len()).map(move |i| {
            let generation = self.calculate_generation_at_index(i);
            Handle::new(i, generation)
        })
    }

    /// Calculate what generation should be at a given index
    fn calculate_generation_at_index(&self, index: usize) -> u32 {
        if index < self.next_index {
            self.current_generation
        } else {
            self.current_generation.saturating_sub(1)
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for GenerationalBuffer<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationalBuffer")
            .field("capacity", &self.capacity())
            .field("len", &self.len())
            .field("next_index", &self.next_index)
            .field("current_generation", &self.current_generation)
            .field("entries", &self.entries)
            .finish()
    }
}
impl<O> PartialEq for Handle<O> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}
impl<O> Eq for Handle<O> {}
impl<O> Clone for Handle<O> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<O> Copy for Handle<O> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut buffer = GenerationalBuffer::new(3);

        // Insert some values
        let h1 = buffer.push(10);
        let h2 = buffer.push(20);
        let h3 = buffer.push(30);

        // Check values
        assert_eq!(buffer.get(h1), Some(&10));
        assert_eq!(buffer.get(h2), Some(&20));
        assert_eq!(buffer.get(h3), Some(&30));
        assert_eq!(buffer.len(), 3);
        assert!(buffer.is_full());
    }

    #[test]
    fn test_circular_wrapping() {
        let mut buffer = GenerationalBuffer::new(2);

        // Fill the buffer
        let h1 = buffer.push(10);
        let h2 = buffer.push(20);

        // Wrap around - this should invalidate h1
        let h3 = buffer.push(30);

        // h1 should now be invalid due to generation mismatch
        assert_eq!(buffer.get(h1), None);
        assert_eq!(buffer.get(h2), Some(&20));
        assert_eq!(buffer.get(h3), Some(&30));
        assert!(!buffer.is_valid(h1));
        assert!(buffer.is_valid(h2));
        assert!(buffer.is_valid(h3));
        assert_eq!(buffer.len(), 2);

        // let's do one more turn
        let h4 = buffer.push(40); // This should overwrite h2
        let h5 = buffer.push(50); // This should overwrite h3
        assert_eq!(buffer.get(h4), Some(&40));
        assert_eq!(buffer.get(h5), Some(&50));
        assert!(!buffer.is_valid(h2)); // h2 should be invalid now
        assert!(!buffer.is_valid(h3)); // h3 should be invalid now
    }

    #[test]
    fn test_generation_calculation() {
        let mut buffer = GenerationalBuffer::new(3);

        // Fill buffer multiple times
        let handles: Vec<_> = (0..10).map(|i| buffer.push(i)).collect();

        // Only the last 3 handles should be valid
        for (i, &handle) in handles.iter().enumerate() {
            if i < 7 {
                assert!(!buffer.is_valid(handle), "Handle {i} should be invalid");
            } else {
                assert!(buffer.is_valid(handle), "Handle {i} should be valid");
            }
        }
    }

    #[test]
    fn test_iterator() {
        let mut buffer = GenerationalBuffer::new(3);

        buffer.push(10);
        buffer.push(20);
        buffer.push(30);
        buffer.push(40);
        buffer.push(50);
        buffer.push(60);
        buffer.push(70);
        buffer.push(80);

        let mut values: Vec<i32> = buffer.values().cloned().collect();
        values.sort(); // no order is currently guaranteed
        assert_eq!(values, vec![60, 70, 80]);

        let handles: Vec<_> = buffer.handles().collect();
        assert_eq!(handles.len(), 3);

        // Verify all handles are valid
        for handle in handles {
            assert!(buffer.is_valid(handle));
        }
    }

    #[test]
    fn test_growing_buffer() {
        let mut buffer = GenerationalBuffer::new(5);

        // Add elements one by one
        let h1 = buffer.push(1);
        let h2 = buffer.push(2);
        let h3 = buffer.push(3);

        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_full());

        // All handles should be valid
        assert!(buffer.is_valid(h1));
        assert!(buffer.is_valid(h2));
        assert!(buffer.is_valid(h3));

        // Values should be accessible
        assert_eq!(buffer.get(h1), Some(&1));
        assert_eq!(buffer.get(h2), Some(&2));
        assert_eq!(buffer.get(h3), Some(&3));
    }
}
