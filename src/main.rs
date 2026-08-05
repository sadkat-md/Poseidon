use poseidon::{storage::engine::StorageEngine};
use std::io::{self, Write};


fn main() -> io::Result<()> {
    
    let mut engine =  StorageEngine::open("storage", 1_048_576)?;
        
    loop {              
        println!("Welcome to Poseidon! ");
        
        println!("List of Operations Available : GET | PUT | DELETE | EXIT");
        
        print!("Enter your command : ");
        
        io::stdout().flush()?;

        let mut input = String::new();
        
        io::stdin().read_line(&mut input)?;

        let clean_input = input.trim();
        let mut words = clean_input.split_whitespace();

        let command_name = words.next();
        let key = words.next();
        let value = words.next();

        if clean_input == "exit" {
            break;
        }

        match command_name {
            Some("GET") => {
                 if let Some(k) = key {
                    match engine.get(k)? {
                        Some(val) => println!("Value : {}", val),
                        None => println!("Key not found")
                    }
                 } else {
                    print!("Usage: GET <key>");
                 }
            }
            Some("PUT") => {
                if let (Some(k),Some(v)) = (key,value) {
                    engine.put(k.to_string(), v.to_string())?;
                    println!("Key '{}' set successfully ", k);
                } else {
                    println!("You must provide a key value and a value");
                }
            }
            Some("DELETE") => {
                if let Some(k) = key {
                    engine.delete(k.to_string())?;
                    println!("Key '{}' deleted successfully", k);
                } else {
                    println!("You must provide a key value");
                }
            }
            _=>{
                println!("Unknown Command");
            }
        }
    }
  Ok(())
}
