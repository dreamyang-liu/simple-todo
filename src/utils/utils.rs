use std::{collections::HashMap, fmt::Display};

pub fn print_record_sorted_by_key<K, V>(record_map: HashMap<K, V>)
where
    K: Display + Ord,
    V: Display,
{
    let mut record_vec: Vec<_> = record_map.iter().collect();
    record_vec.sort_by(|a, b| a.0.cmp(b.0));
    for record in record_vec {
        println!("🔥 {}: {}", record.0, record.1);
    }
}
