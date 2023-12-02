use std::{env, time::{SystemTime, UNIX_EPOCH}};
use serde_derive::{Serialize, Deserialize};
use clap::Parser;

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
    delete: Option<u32>,
}

static PATH_ENV_VAR: &str = "rtodo_path";

fn main() {
    let store = Store::new();
    let args = Args::parse();

    process(args, store);
}

fn process(args: Args, store: Store) {
    // delete first, because delete operate dependency NO which will changed caused by add operate
    if let Some(no) = args.delete {
        if no == 0 {
            output::error(&format!("delete item fail, {}", ERR_ITEM_NOT_FOUND));
        } else if let Err(msg) = store.delete(no-1) {
            output::error(&format!("delete item fail, {}", msg));
        }
    }
    if let Some(content) =  args.add {
        if let Err(msg) = store.add(Item { content }) {
            output::error(&format!("add item fail, {}", msg));
        }
    }
    for (i, ele) in store.list(args.list).iter().enumerate() {
        output::list_print(i+1, &ele.content);
    }
}

mod output {
    use colored::Colorize;

    pub fn list_print(no: usize, content: &str) {
        println!("NO {}: {}", no.to_string().blue(), content.blue());
    }

    pub fn error(err_msg: &str) {
        println!("rtodo error: {}", err_msg.red());
    }

}


trait Operate {
    fn add(&self, item: Item) -> Result<u32, &'static str>;
    fn delete(&self, no: u32) -> Result<Item, &'static str>;
    fn list(&self, keyword: Option<String>) -> Vec<Item>;
}

#[derive(Serialize, Deserialize)]
struct Item {
    content: String,
}

impl Into<sled::IVec> for Item {
    fn into(self) -> sled::IVec {
        self.content.as_bytes().into()
    }
}

impl From<sled::IVec> for Item {
    fn from(vec: sled::IVec) -> Item {
        Item { content:  std::str::from_utf8(&vec).unwrap_or_default().to_string() }
    }
}

struct  Store {
    db: sled::Db
}

impl Store {
    fn new() -> Self {
        let path_from_env = env::vars().find(|(key, _)| {
            return key == PATH_ENV_VAR
        });
    
        let path = match path_from_env {
             Some((_, x)) => x.to_owned(),
             None => "./.rtodo_db".to_string(),
        };
    
        Store {db: sled::open(path).unwrap()}
    }
    fn gen_key(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }
    fn count(&self) -> u32 {
        self.db.len() as u32
    }
}

static ERR_ADD_ITEM: &'static str = "add item failed";

static ERR_ITEM_NOT_FOUND: &'static str = "item not found";

impl Operate for Store {
    fn add(&self, item: Item) -> Result<u32, & 'static str> {
        match self.db.insert::<std::string::String, Item>(self.gen_key().to_string(), item) {
            Ok(_) => Ok(self.count()),
            Err(_) => Err(ERR_ADD_ITEM)
        }
    }

    fn delete(&self, no: u32) -> Result<Item, &'static str> {
        if let Some(Ok((k, _))) = self.db.into_iter().nth(no as usize) {
            if let Ok(Some(item)) = self.db.remove(k) {
                return Ok(item.into())
            }
        }
        Err(ERR_ITEM_NOT_FOUND)
    }

    fn list(&self, _: Option<String>) -> Vec<Item> {
        self.db.into_iter().filter_map(|res| {
            match res {
                Ok((_, val)) => Some(Item::from(val)),
                Err(_) => None
            }
        }).collect()
    }
}