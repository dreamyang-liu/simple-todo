use clap::Parser;
use log::{error, warn};
use utils::cli::{Cli, Commands};
use utils::database::{Database, Operation, Record};
use utils::utils::print_record_sorted_by_key;
mod utils;

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let mut db = Database::open("./db");
    match args.command {
        Commands::Info => {
            println!("simple-todo is a simple todo list manager.");
        }
        Commands::Add { content } => {
            if let Some(content) = content {
                let id = db.get_next_id();
                let record = Record {
                    id,
                    content: Some(content),
                    operation: Operation::APPEND,
                };
                db.add_record(&record);
            } else {
                error!("You need to specify the content of the todo item.");
            }
        }
        Commands::Remove { id } => {
            if let Some(id) = id {
                match id.parse::<u64>() {
                    Ok(id) => {
                        let _ = db.add_record(&Record {
                            id: id,
                            content: None,
                            operation: Operation::REMOVE,
                        });
                    }
                    Err(_) => error!("Invalid id, id should be and u64 integer."),
                }
            }
        }
        Commands::List => {
            let records = db.aggregate();
            match records.len() {
                0 => warn!("No records. You can add one with `simple-todo add [content]`"),
                _ => {
                    print_record_sorted_by_key(records);
                }
            }
        }
    }
}
