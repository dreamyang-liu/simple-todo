use utils::database::{Database, Record, Operation};
use utils::cli::{Cli, Commands};
use clap::Parser;
use log::error;

mod utils;

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let mut db = Database::open("./db");
    match args.command {
        Commands::Info => {
            println!("Rodo is a simple todo list manager.");
        },
        Commands::Add { content } => {
            if let Some(content) = content {
                let id = db.get_next_id();
                let record = Record {
                    id,
                    content: Some(content),
                    operation: Operation::APPEND
                };
                db.add_record(&record);
            } else {
                error!("You need to specify the content of the todo item.");
            }
        },
        Commands::Remove { id } => {
            if let Some(id) = id {
                match id.parse::<u64>() {
                    Ok(id) => {
                        db.add_record(&Record { id: id, content: None, operation: Operation::REMOVE });
                    },
                    Err(_) => error!("Invalid id, id should be and u64 integer.")
                }
            }
        },
        Commands::List => {
            if let Ok(records) = db.aggregate() {
                match records.len() {
                    0 => println!("No records. You can add one with `rodo add [content]`"),
                    _ => {
                        for record in records {
                            println!(" ⬜️ {}: {}", record.0, record.1);
                        }
                    }
                }
            }
        }
    }
}