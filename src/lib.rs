//! A ring buffer returning generational handles on insertion, so that
//! you can check if an item has been replaced since you got the handle.
//!
//! This is safe and efficient, the storage is a simple vector not taking
//! more space than a minimal ring buffer.
//!
//! ```
//! let mut buffer = generational_buffer::GenerationalBuffer::new(2);
//!
//! // Fill the buffer
//! let h1 = buffer.push(10);
//! let h2 = buffer.push(20);
//!
//! // Wrap around - this should invalidate h1
//! let h3 = buffer.push(30);
//!
//! // h1 should now be invalid due to generation mismatch
//! assert_eq!(buffer.get(h1), None);
//! assert_eq!(buffer.get(h2), Some(&20));
//! assert_eq!(buffer.get(h3), Some(&30));
//! assert!(!buffer.is_valid(h1));
//! assert!(buffer.is_valid(h2));
//! assert!(buffer.is_valid(h3));
//! assert_eq!(buffer.len(), 2);
//!
//! // let's do one more turn
//! let h4 = buffer.push(40); // This should overwrite h2
//! let h5 = buffer.push(50); // This should overwrite h3
//! assert_eq!(buffer.get(h4), Some(&40));
//! assert_eq!(buffer.get(h5), Some(&50));
//! assert!(!buffer.is_valid(h2)); // h2 should be invalid now
//! assert!(!buffer.is_valid(h3)); // h3 should be invalid now
//! ```

mod generational_buffer;

pub use generational_buffer::*;
