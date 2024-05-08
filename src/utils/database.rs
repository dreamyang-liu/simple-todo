use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Error, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use log::{error, info, warn};

use super::error::DatabaseError;

pub fn parse_line(line: &str) -> Result<Record, DatabaseError> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() == 1 {
        error!("Found error line: {}", line);
    }
    if fields.len() == 2 {
        let operation = match fields[1].parse::<i32>().unwrap() {
            0 => Operation::APPEND,
            1 => Operation::REMOVE,
            _ => return Err(DatabaseError::new("Undefined operation")),
        };
        return Ok(Record {
            id: fields[0].parse::<u64>().unwrap(),
            content: None,
            operation: operation
        });
    }
    if fields.len() == 3 {
        let operation = match fields[1].parse::<i32>().unwrap() {
            0 => Operation::APPEND,
            1 => Operation::REMOVE,
            _ => return Err(DatabaseError::new("Undefined operation"))
        };
        return Ok(Record {
            id: fields[0].parse::<u64>().unwrap(),
            content: Some(fields[2].to_string()),
            operation: operation
        });
    }
    panic!("dirty record");
}

pub enum Operation {
    APPEND = 0,
    REMOVE = 1
}

pub struct Record {
    pub id: u64,
    pub content: Option<String>,
    pub operation: Operation,
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.operation {
            Operation::APPEND => {
                write!(f, "APPEND: ({}, {})", self.id, self.content.as_ref().unwrap())
            },
            Operation::REMOVE => {
                write!(f, "REMOVE: ({})", self.id)
            }
        }
    }
}

pub struct Database {
    checkpoint: File,
    change: File
}

impl Database {

    pub fn open(db_name: &str) -> Database {
        let checkpoint_path = db_name;
        let change_path = db_name.to_string() + ".change";
        let checkpoint = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(checkpoint_path)
            .unwrap();
        let change = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(change_path)
            .unwrap();
        Database {
            checkpoint,
            change
        }
    }

    pub fn aggregate(&mut self) -> Result<HashMap<u64, String>, DatabaseError> {
        let change_record = self.read_change_records();
        let mut result = self.read_checkpoint()?;
        for record in change_record.iter() {
            match result.contains_key(&record.id) {
                true => {
                    match record.operation {
                        Operation::REMOVE =>  {
                            result.remove(&record.id);
                        },
                        _ => warn!("Undefined operation for existing record, Skipping")
                    }
                },
                false => {
                    match record.operation {
                        Operation::APPEND =>  {
                            result.insert(record.id, record.content.clone().unwrap());
                        },
                        _ => warn!("Undefined operation for new record, Skipping")
                    }
                }
            }
        }
        self.write_checkpoint(&result);
        self.flush_change()?;
        return Ok(result);

    }

    fn flush_change(&mut self) -> Result<(), DatabaseError> {
        match self.change.set_len(0) {
            Ok(_) => Ok(()),
            Err(e) => Err(DatabaseError::new(format!("Error when flushing change file: {}", e.to_string()).as_str()))
        }
    }

    pub fn get_next_id(&mut self) -> u64 {
        return SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    }

    fn write_checkpoint(&mut self, checkpoint_map: &HashMap<u64, String>) {
        self.checkpoint.set_len(0).unwrap();
        for (key, value) in checkpoint_map {
            writeln!(self.checkpoint, "{},{}", key, value).unwrap();
        }
    }

    fn read_checkpoint(&mut self) -> Result<HashMap<u64, String>, DatabaseError> {
        let mut map = HashMap::<u64, String>::new();
        let reader = BufReader::new(&self.checkpoint);
        for line in reader.lines() {
            if let Ok(line) = line {
                let fields: Vec<&str> = line.split(',').collect();
                map.insert(fields[0].parse::<u64>().unwrap(), fields[1].to_string());
            } else {
                return Err(DatabaseError::new(format!("Error when reading checkpoint file at line: {}", line.as_ref().unwrap()).as_str()))
            }
        }
        return Ok(map);
    }

    fn read_change_records(&mut self) -> Vec<Record> {
        let reader = BufReader::new(&self.change);
        reader
            .lines()
            .map_while(Result::ok)
            .filter(|line| !line.is_empty())
            .map(|line| parse_line(&line))
            .take_while(|result| result.is_ok())
            .map(|x| x.ok().unwrap())
            .collect()
    }

    pub fn add_record(&mut self, record: &Record) -> Result<(), Error> {
        let line = match &record.content {
            Some(content) => format!("{},{},{}", record.id, 0, content),
            None => format!("{},{}", record.id, 1)
        };
        writeln!(self.change, "{}", line).unwrap();
        info!("Operation {}", record);
        Ok(())
    }
}