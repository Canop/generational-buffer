[![MIT][s2]][l2] [![Latest Version][s1]][l1] [![docs][s3]][l3] [![Chat on Miaou][s4]][l4]

[s1]: https://img.shields.io/crates/v/generational-buffer.svg
[l1]: https://crates.io/crates/generational-buffer

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://docs.rs/generational-buffer/badge.svg
[l3]: https://docs.rs/generational-buffer/

[s4]: https://miaou.dystroy.org/static/shields/room.svg
[l4]: https://miaou.dystroy.org/3


A ring buffer returning generational handles on insertion, so that
you can check if an item has been replaced since you got the handle.

This is safe and efficient, the storage is a simple vector not taking
more space than a minimal ring buffer.

```
let mut buffer = generational_buffer::GenerationalBuffer::new(2);

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
```
