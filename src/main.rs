use clap::Parser;

mod interactive;
mod basic_operate;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author = "eddie", version = "0.1", about = "a simple todo-list program written by rust")]
struct Args {

    /// Add a todo item
    #[arg(short, long)]
    add: Option<String>,
    
    /// list not done todo items, if present fuzzy match item content
    #[arg(short, long, default_value = "")]
    list: Option<String>,

    /// list all todo items, include done, if present fuzzy match item content
    #[arg(long, default_value = "")]
    list_all: Option<String>,

    /// delete todo item by item Key
    #[arg(long)]
    delete: Option<String>,

     /// delete todo item by item Key
     #[arg(long)]
     done: Option<String>,

    /// delete_all delete all item list
    #[arg(long)]
    delete_all: bool,

    /// use interactive mode to use rtodo
    #[arg(short, long)]
    interactive: bool,
}

fn main() {
    let store = basic_operate::Store::new();
    let args = Args::parse();

    if args.interactive {
        let _ = interactive::process();
    } else {
        basic_operate::process(args, store);
    }
}

pub(crate) mod output {
    use colored::Colorize;

    use crate::basic_operate;

    pub(crate) fn list_print(item: &basic_operate::KeyWithItem) {
        println!("{}", item);
    }

    pub fn error(err_msg: String) {
        println!("{}", format!("rtodo error: {}", err_msg).red());
    }

    pub fn info(msg: String) {
        println!("{}", msg.blue());
    }
}

