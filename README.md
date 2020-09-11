[![Crates.io](https://img.shields.io/crates/v/pq-bincode.svg)](https://crates.io/crates/pq-bincode)

# Persistent Queue - Bincode (pq-bincode)

A wrapper crate of [queue-file](https://github.com/ing-systems/queue-file) for [bincode](https://github.com/servo/bincode) serializable structs.

## Usage Example

```rust
use chrono::{DateTime, Utc};
use pq_bincode::{IBincodeSerializable, PQBincode};
use serde::{Deserialize, Serialize};
use std::io::{Result as IOResult, Write};

#[derive(Clone, Debug, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Person {
    pub name: String,
    pub birthdate: DateTime<Utc>,
}

impl Default for Person {
    fn default() -> Person {
        Person {
            name: "John Doe".into(),
            birthdate: Utc::now(),
        }
    }
}

impl IBincodeSerializable for Person {}

fn pq_test(pq: &mut PQBincode<Person>) -> IOResult<()> {
    println!("PQ path is \"{}\"", pq.get_persistent_path());
    println!("Current PQ count is {}", pq.count()); // 0 count

    print!("Inserting two identical John Doe...");
    std::io::stdout().flush()?;
    let john_doe = Person::default();
    pq.enqueue(john_doe.clone())?;
    pq.enqueue(john_doe)?;
    println!("DONE");
    println!("Current PQ count is {}", pq.count()); // 2 count

    print!("Trying cancellable dequeue (Cancel)...");
    std::io::stdout().flush()?;
    pq.cancellable_dequeue(|_| false)?;
    println!("DONE");
    println!("Current PQ count is {}", pq.count()); // 2 count

    print!("Trying cancellable dequeue (Proceed)...");
    std::io::stdout().flush()?;
    pq.cancellable_dequeue(|_| true)?;
    println!("DONE");
    println!("Current PQ count is {}", pq.count()); // 1 count

    print!("Trying a simple dequeue...");
    std::io::stdout().flush()?;
    let dequeued_item = pq.dequeue()?;
    println!("DONE");
    println!("Current PQ count is {}", pq.count()); // 0 count
    println!("Dequeued item:\n{:#?}", dequeued_item);

    Ok(())
}

fn main() -> IOResult<()> {
    let mut pq_person = PQBincode::<Person>::new("person.pqb")?;
    pq_test(&mut pq_person)?;

    Ok(())
}

```

Or, just run `cargo run --example allfeatures` it will yield:

```bash
PQ path is "person.pqb"
Current PQ count is 0
Inserting two identical John Doe...DONE
Current PQ count is 2
Trying cancellable dequeue (Cancel)...DONE
Current PQ count is 2
Trying cancellable dequeue (Proceed)...DONE
Current PQ count is 1
Trying a simple dequeue...DONE
Current PQ count is 0
Dequeued item:
Some(
    Person {
        name: "John Doe",
        birthdate: 2020-09-11T09:48:32.470091527Z,
    },
)
```

## Author(s)

- [Ujang360](https://github.com/Ujang360)
