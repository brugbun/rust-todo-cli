use std::io::{BufReader, BufWriter, BufRead, Write, Seek, SeekFrom};
use std::fs::{OpenOptions, File};
use std::fmt;
use std::collections::HashMap;

const STRIKE: &str = "\x1b[9m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

#[derive(PartialEq)]
enum State {
    INPROG,
    FINISHED,
    CLOSED,
    NORMAL,
    NONE
}

struct TodoItem {
    state: State,
    text: String
}

impl fmt::Display for TodoItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

fn output_inprog(out: &TodoItem, index: usize) {
    println!("{}) {}[?] {}{}", index, YELLOW, &out.text, RESET)
}

fn output_closed(out: &TodoItem, index: usize) {
    println!("{}) {}[#] {}{}{}", index, GRAY, STRIKE, &out.text, RESET)
}

fn output_finished(out: &TodoItem, index: usize) {
    println!("{}) {}[!] {}{}{}", index, GREEN, STRIKE, &out.text, RESET)
}

fn output_normal(out: &TodoItem, index: usize) {
    println!("{}) [-] {}", index, &out.text)
}

fn output_delete(out: &TodoItem, index: usize) {
    println!("{}) [DELETED] {}{}{}", index, STRIKE, &out.text, RESET)
}

fn serialize_todo(text: String) -> TodoItem {
    let state: State = match text.chars().next().unwrap() {
        '?' => State::INPROG,
        '!' => State::FINISHED,
        '-' => State::CLOSED,
        '.' => State::NORMAL,
        _ => State::NONE
    };
    TodoItem{state, text: String::from(&text[1..]) }
}

fn deserialize_todo(item: &TodoItem) -> String {
    match item.state {
        State::INPROG => format!("{}{}", '?', item.text),
        State::FINISHED => format!("{}{}", '!', item.text),
        State::CLOSED => format!("{}{}", '-', item.text),
        State::NORMAL => format!("{}{}", '.', item.text),
        State::NONE => String::from(&item.text)
    }
}

fn output_todo_items(items: &Vec<TodoItem>, closed: bool) {
    for (index, value) in items.iter().enumerate() {
        match value.state {
            State::INPROG => output_inprog(value, index),
            State::FINISHED => output_finished(value, index),
            State::NORMAL => output_normal(value, index),
            State::CLOSED => if closed { output_closed(value, index) },
            State::NONE => output_delete(value, index)
        }
    }
}

fn write_changes(items: &Vec<TodoItem>, mut writer: BufWriter<File>, mut archive: BufWriter<File>) -> std::io::Result<()> {
    for item in items {
        match item.state {
            State::CLOSED => writeln!(archive, "{}", deserialize_todo(item))?,
            State::NONE => (),
            _ => writeln!(writer, "{}", deserialize_todo(item))?
        }
    }
    Ok(())
}

fn parse_command(command: String, todo_items: &mut Vec<TodoItem>) -> Option<()> {
    let tokens: Vec<&str> = command.split(' ').collect();
    match tokens[0].trim() {
        "help" => help(tokens),
        "add" => add(tokens, todo_items),
        "edit" => edit(tokens, todo_items),
        "delete" => delete(tokens, todo_items),
        "quit" => return Some(()),
        _ => help(tokens)
    }
    None
}

fn help(args: Vec<&str>) {
   let info = HashMap::from([
    (
    "help", 
    "help <arg>\n\thelp : prints this output\n\thelp add : prints help for the `add` command\n\thelp edit : prints help for the `edit` command\n\thelp delete : prints help for the `delete` command"
    ),
    (
    "add",
    "add <arg>\n\tadd : prints the help page for this command\n\t<arg> : the description for the todo"
    ),
    (
    "edit",
    "edit <arg> <flags>\n\tedit : prints the help page for this command\n\t<arg> : the todo index\n\t<flags>\n\t\t-t : the todo text\n\t\t-s : the todo state \n\t\t(1||NORMAL)\n\t\t(2||INPROG)\n\t\t(3||FINSHED)\n\t\t(4||CLOSED)"
    ),
    (
    "delete",
    "delete <arg>\n\tdelete : prints the help page for this command\n\t<arg> : the todo index"
    )
    ]);

    if args.len() > 1 {
        match args[1].trim() {
            "add" => {println!("{}", info["add"]); let _ = std::io::stdin().read_line(&mut String::new()); return},
            "edit" => {println!("{}", info["edit"]); let _ = std::io::stdin().read_line(&mut String::new()); return},
            "delete" => {println!("{}", info["delete"]); let _ = std::io::stdin().read_line(&mut String::new()); return},
            _ => ()
        }
    }
    for (_, v) in info.iter() {
        println!("{}", v);
    }
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn add(args: Vec<&str>, todo_items: &mut Vec<TodoItem>) {
    if args.len() > 1 {
       todo_items.push(
            TodoItem{
                text: String::from(args[1..].join(" ").trim()), 
                state: State::NORMAL
            }
       );
       return
    }
    help(vec!["", "add"])
}

fn edit(args: Vec<&str>, todo_items: &mut Vec<TodoItem>) {
    'edit_block: {
        if args.len() > 1 && (args.contains(&"-t") || args.contains(&"-s")) {
            if args[1].trim().parse::<usize>().is_err() {
                break 'edit_block
            }
            
            let index = args[1].trim().parse::<usize>().unwrap();

            if index > todo_items.len()-1 {
                break 'edit_block
            }

            let mut text = String::new();
            let mut state: State = State::NONE;
            for (i, v) in args.iter().enumerate() {
                match v.trim() {
                    "-t" => {
                        if i == args.len()-1 {
                            break 'edit_block
                        }
                        for k in &args[i+1..] {
                            match k.trim() {
                                "-s" => break,
                                n => text.push_str(&format!("{} ", n))
                            }
                        }
                    },
                    "-s" => {
                        if i == args.len()-1 {
                            break 'edit_block
                        }
                        match args[i+1].trim() {
                            "1" => state = State::NORMAL,
                            "2" => state = State::INPROG,
                            "3" => state = State::FINISHED,
                            "4" => state = State::CLOSED,
                            _ => break 'edit_block
                        }
                    },
                    _ => ()
                }
            }
            if text != "" {
                todo_items[index].text = text.trim().to_string();
            }
            if state != State::NONE {
                todo_items[index].state = state;
            }
            return
        }
    }
    help(vec!["", "edit"])
}

fn delete(args: Vec<&str>, todo_items: &mut Vec<TodoItem>) {
    'del_block: {
        if args.len() > 1 {
            if args[1].trim().parse::<usize>().is_err() {
                break 'del_block
            }
            let index = args[1].trim().parse::<usize>().unwrap();
            todo_items[index].state = State::NONE;
            return;
        }
    }
    help(vec!["", "delete"]);
}

fn main() -> std::io::Result<()> {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/home/brug/.todo")?;
    let mut del = OpenOptions::new()
        .append(true)
        .open("/home/brug/.todo.old")?;
    let reader = BufReader::new(&f);

    println!("\x1B[2J\x1B[1;1H");

    let mut todo_items = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => return Err(e)
        };
        if line == "" { continue; }

        todo_items.push(serialize_todo(line));
    }   
    
    'program: loop {
        println!("\x1B[2J\x1B[1;1H");
        output_todo_items(&todo_items, true);

        let mut opt_buffer: String = String::new();

        print!("\n\n>: ");
        std::io::stdout()
            .flush()
            .unwrap();
        std::io::stdin()
            .read_line(&mut opt_buffer)
            .expect("Failed to read line");
        println!("\x1B[2J\x1B[1;1H");
        match parse_command(opt_buffer, &mut todo_items) {
            Some(..) => break 'program,
            None => ()
        }
    }
    //inf loop {}
    //output_todo
    //add \n
    //give cli prompt (>: )
    //add function to capture output and match it 
    //  help -> display_help(None/arg 1) 
    //  add -> display_help(add) (if invalid) -> add_todo_item(params) (if valid)
    //  edit -> display_help(edit) (if invalid) -> edit_todo_item(params) (if valid)
    //  delete -> display_help(delete) (if invalid) -> delete_todo_item(params) (if valid)
    //  quit -> break;
    //  _ -> display_help(None)
    //
    //then just this below vvvv

    f.set_len(0)?;
    f.seek(SeekFrom::Start(0))?;

    let mut writer = BufWriter::new(f);
    let mut archive = BufWriter::new(del);

    match write_changes(&todo_items, writer, archive) {
        Ok(_) => (),
        Err(e) => return Err(e)
    }

    Ok(())
}
