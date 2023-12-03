use std::{env, time::{UNIX_EPOCH, SystemTime}, io, fmt::Display};

use colored::Colorize;
use serde_derive::{Serialize, Deserialize};

use crate::output;



pub(crate) fn process(args: super::Args, store: Store) {
    // delete first, because delete operate dependency NO which will changed caused by add operate
    if let Some(no) = args.delete {
        if let Some((k, v)) = store.find_by_no(no) {
            // todo confirm delete operate
            if !is_confirm(no, &v) {
                output::error(format!("cancel delete item: {}", v));
                return 
            }
            // exec delete
            if let Err(msg) = store.delete(k) {
                output::error(format!("delete item fail, {}", msg));   
            }
        } else {
            output::error(format!("delete item fail, {}", ERR_ITEM_NOT_FOUND));
        }
    }
    if let Some(content) =  args.add {
        if let Err(msg) = store.add(Item { content }) {
            output::error(format!("add item fail, {}", msg));
        }
    }
    for (i, ele) in store.list(args.list).iter().enumerate() {
        output::list_print(i+1, ele);
    }
}

fn is_confirm(no: usize, v: &Item) -> bool {
    output::info(format!("confirm delete this item: NO {}: {}? (Y/N)", no, v.content));
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    return input.trim().to_lowercase() == "y"
}


trait Operate {
    fn add(&self, item: Item) -> Result<u32, &'static str>;
    fn find_by_no(&self, no: usize) -> Option<(sled::IVec, Item)>;
    fn delete(&self, k: sled::IVec) -> Result<Item, &'static str>;
    fn list(&self, keyword: Option<String>) -> Vec<Item>;

}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Item {
    content: String,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.content.blue()))
    }
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

pub(crate) struct  Store {
    db: sled::Db
}



static PATH_ENV_VAR: &str = "rtodo_path";

impl Store {
    pub fn new() -> Self {
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

    fn find_by_no(&self, no: usize) -> Option<(sled::IVec, Item)> {
        if let Some(Ok((k, v))) = self.db.into_iter().nth(no-1) {
            return Some((k, v.into()))
        }
        None
    }

    fn delete(&self, k: sled::IVec) -> Result<Item, &'static str> {
        if let Ok(Some(item)) = self.db.remove(k) {
            return Ok(item.into())
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