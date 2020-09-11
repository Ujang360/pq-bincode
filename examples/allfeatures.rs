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
