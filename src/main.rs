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
    
    /// list add todo items, if present fuzzy match item content
    #[arg(short, long, default_value = "")]
    list: Option<String>,

    /// delete todo item by item NO
    #[arg(short, long)]
    delete: Option<usize>,

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

mod output {
    use colored::Colorize;

    pub fn list_print(no: usize, content: &str) {
        let no = format!("NO {}", no).green().on_blue();
        println!("{}: {}", no, content.blue());
    }

    pub fn error(err_msg: &str) {
        println!("{}", format!("rtodo error: {}", err_msg).red());
    }

}

