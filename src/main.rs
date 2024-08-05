/* 
    todo-rs is a CLI TODO list written in Rust

    - stored in todo.txt

    - "add" creates a new line in the TODO list with corresponding data
        - added with a prefix, i.e. '* "item"'
        - double quotes are added
    - "rm" finds the item in the list and removes it
    - "done" adds "-s" as a suffix to the selected item, i.e. '* "item" -s'
    - "list" parses this data and prints it out, i.e. '1. Item' or '1. Item -s" (replace -s with strikethrough)
    - "clear" clears the entire todo.txt file
*/

use clap::Parser;

use std::{
    env::current_exe, fs::{
        File, OpenOptions
    }, io::{
        Read, Result, Write
    },
    path::PathBuf
};

use lazy_static::lazy_static;

const ABOUT_MESSAGE: &str =
"todo-rs
-------
todo-rs is a CLI TODO list written in Rust for a
super fast response time, utilizing the `clap` and `lazy_static`
libraries.
------
supported commands: add, rm, done, undone, list, clear";

#[derive(Parser, Debug)]
#[command(version="1.0.0", long_about=ABOUT_MESSAGE)]
struct CommandArguments {
    command: String, // add, rm, done, list
    #[clap(short='M', long="message", default_value="0", long_help="Add a message to your command, used for 'add'")]
    message: String,
    #[clap(short='I', long="index", default_value="0", long_help="Add an index value to your command, used for 'rm' and 'done'")]
    index: String
}

fn main() {
    let args = CommandArguments::parse();

    let command: String = args.command;
    let message: String = args.message;
    let index: String = args.index.to_string();

    if !command.is_empty() {
        match &command as &str {
            "add" => {
                append_to_list(&message);
                println!("Added to your TODO list: {}", message);
            },

            "rm" => {
                match index.parse::<usize>() {
                    Ok(parsed_index) => {
                        remove_from_list(parsed_index);
                        println!("Removed from your TODO list: {}", index);
                    },
                    Err(e) => {
                        eprintln!("Error parsing index: {}", e);
                    }
                }
            },

            "done" => {
                match index.parse::<usize>() {
                    Ok(parsed_index) => {
                        mark_as_done(parsed_index, true);
                        println!("Checked off item from your TODO list: {}", index);
                    },
                    Err(e) => {
                        eprintln!("Error parsing index: {}", e);
                    }
                }
            },

            "undone" => {
                match index.parse::<usize>() {
                    Ok(parsed_index) => {
                        mark_as_done(parsed_index, false);
                        println!("Unchecked item from your TODO list: {}", index);
                    },
                    Err(e) => {
                        eprintln!("Error parsing index: {}", e);
                    }
                }
            },

            "list" => {
                let list_content = get_list_content();
                let parsed_list: String = parse_list_content(list_content);
                
                if parsed_list.is_empty() {
                    println!("Nothing was found in your TODO list! 😊");
                } else {
                    println!("TODO list:\n{}", parsed_list);
                }
            },

            "clear" => {
                set_list_length(0);
                println!("Your TODO list has been cleared!");
            },

            _ => println!("`{}` is not a valid command, run todo --help for more information.", command)
        }
    }
}

// Get the executable path
fn inner_main() -> Result<PathBuf> {
    let exe = current_exe()?;
    let dir = exe.parent().expect("Executable must be in some directory").to_path_buf();
    Ok(dir)
}

lazy_static! {
    static ref TODO_PATH: PathBuf = {
        let mut path = inner_main().expect("Failed to get executable path");
        path.push("todo.txt");
        path
    };
}

// Clear list by setting the length of the file to 0
fn set_list_length(size: u64) {
    let file = get_file(false, true, false, false);
    file.set_len(size).expect("Unable to clear file");
}

// Read the list content as a string
fn get_list_content() -> String {
    let mut file = get_file(true, true, false, false);
    let mut file_content = String::new();
    
    file.read_to_string(&mut file_content).unwrap();

    file_content
}

// Parse list content and apply formatting
fn parse_list_content(content: String) -> String {
    let mut result = String::new();
    for line in content.lines() {
        let mut parts = line.splitn(2, ' '); // Split on the first space
        if let (Some(number), Some(rest)) = (parts.next(), parts.next()) {
            let mut formatted_line = format!("{} {}", number, rest.trim_start());
            if rest.ends_with("-s") {
                formatted_line = format!("{} \x1b[9m{}\x1b[0m", number, rest.trim_end_matches("-s").trim_start().trim_end());
            }
            result.push_str(&formatted_line);
            result.push('\n');
        }
    }
    result
}

// Find the next index for the list
fn find_next_index(content: &str) -> usize {
    let mut max_index = 0;
    for line in content.lines() {
        if let Some((index, _)) = line.split_once('.') {
            if let Ok(num) = index.trim().parse::<usize>() {
                if num > max_index {
                    max_index = num;
                }
            }
        }
    }
    max_index + 1
}

// Append to the list by writing to it
fn append_to_list(message: &str) {
    let file_content = get_list_content();
    let next_index = find_next_index(&file_content);

    let formatted_message = format!("{}. {}\n", next_index, message);

    let mut file = get_file(true, true, false, false);
    file.write_all(formatted_message.as_bytes()).expect("Unable to write to file");
}

// Remove a list item by index
fn remove_from_list(index_number: usize) {
    let file_content = get_list_content();
    let mut new_content = String::new();
    let mut item_found = false;

    for line in file_content.lines() {
        if let Some((index, rest)) = line.split_once('.') {
            if let Ok(num) = index.trim().parse::<usize>() {
                if num == index_number {
                    item_found = true;
                } else {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    if !item_found {
        eprintln!("Item with index {} not found.", index_number);
        return;
    }

    let mut file = get_file(true, true, false, true);
    file.set_len(0).expect("Unable to clear file");
    file.write_all(new_content.as_bytes()).expect("Unable to write to file");
}

// Mark an item as done by appending -s to the end
fn mark_as_done(index_number: usize, done: bool) {
    let file_content = get_list_content();
    let mut new_content = String::new();
    let mut item_found = false;

    for line in file_content.lines() {
        if let Some((index, rest)) = line.split_once('.') {
            if let Ok(num) = index.trim().parse::<usize>() {
                if num == index_number {
                    let trimmed_rest = rest.trim_start();
                    let updated_line;

                    if done {
                        if trimmed_rest.ends_with(" -s") {
                            updated_line = format!("{}. {}\n", index_number, trimmed_rest);
                        } else {
                            updated_line = format!("{}. {} -s\n", index_number, trimmed_rest);
                        }
                    } else {
                        if trimmed_rest.ends_with(" -s") {
                            updated_line = format!("{}. {}\n", index_number, trimmed_rest.trim_end_matches(" -s"));
                        } else {
                            updated_line = format!("{}. {}\n", index_number, trimmed_rest);
                        }
                    }

                    new_content.push_str(&updated_line);
                    item_found = true;
                } else {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    if !item_found {
        eprintln!("Item with index {} not found.", index_number);
    }

    let mut file = get_file(true, true, false, true);
    file.set_len(0).expect("Unable to clear file"); // Clear the file content
    file.write_all(new_content.as_bytes()).expect("Unable to write to file");
}

// Get the file using OpenOptions with required parameters regarding the permissions
// ^ In every command used, if todo.txt doesnt exist it will create it for them.
fn get_file(read: bool, write: bool, append: bool, truncate: bool) -> File {
    if write == false && append == false {
        panic!("Either `append` or `write` must be true in `get_file`")
    }

    let file = OpenOptions::new()
        .read(read)
        .create(true)
        .write(write)
        .append(append)
        .truncate(truncate)
        .open(&*TODO_PATH)
        .expect("Unable to open or create file");

    file
}