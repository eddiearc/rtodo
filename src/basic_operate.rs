use std::{env, time::{UNIX_EPOCH, SystemTime}, io, fmt::Display};

use colored::Colorize;
use serde_derive::{Serialize, Deserialize};

use crate::output;



pub(crate) fn process(args: super::Args, store: impl Operate) {
    // delete first, because delete operate dependency NO which will changed caused by add operate
    if args.delete_all {
        delete_all(&store);
    } else if let Some(no) = args.delete {
        delete(&store, no);
    } else if let Some(no) = args.done {
        done(&store, no);
    }
    if let Some(content) =  args.add {
        if let Err(msg) = store.add(Item { content, done: false }) {
            output::error(format!("add item fail, {}", msg));
        }
    }
    list_items(&store, args.list);
}

fn delete_all(store: &impl Operate) {
    if !confirm(format!("confirm delete all items? (Y/N)")) {
        output::error(format!("cancel delete all items"));
        return
    }
     if let Err(msg) = store.delete_all() {
        output::error(msg.to_string())
    }
}

fn delete(store: &impl Operate, no: usize) {
    if let Some((k, v)) = store.find_by_no(no) {
        // todo confirm delete operate
        if !confirm(format!("confirm delete this item: NO {}: {}? (Y/N)", no, v.content)) {
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

fn done(store: &impl Operate, no: usize) {
    if let Some((k, v)) = store.find_by_no(no) {
        // todo confirm done operate
        if !confirm(format!("confirm done this item: NO {}: {}? (Y/N)", no, v.content)) {
            output::error(format!("cancel done item: {}", v));
            return 
        }
        // exec done
        if let Err(msg) = store.done(k) {
            output::error(format!("done item fail, {}", msg));   
        }
    } else {
        output::error(format!("done item fail, {}", ERR_ITEM_NOT_FOUND));
    }
}

fn list_items(store: &impl Operate, keyword: Option<String>) {
    if let Err(err) = store.list(keyword.clone()) {
        panic!("{}", err)
    }
    for (_, ele) in  store.list(keyword).expect("list items").iter().enumerate() {
        output::list_print( ele);
    }
}

fn confirm(hits: String) -> bool {
    output::info(hits);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    return input.trim().to_lowercase() == "y"
}

pub(crate) struct Key(String);


impl From<sled::IVec> for Key {
    fn from(value: sled::IVec) -> Self {
        Key(std::str::from_utf8(&value).unwrap().to_string())
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Key(value)
    }
}

impl Into<sled::IVec> for Key {
    fn into(self) -> sled::IVec {
        self.0.as_bytes().into()
    }
}


pub(crate) trait Operate {
    fn add(&self, item: Item) -> Result<u32, anyhow::Error>;
    fn find_by_no(&self, no: usize) -> Option<(sled::IVec, Item)>;
    fn delete(&self, k: sled::IVec) -> Result<Item, anyhow::Error>;
    fn delete_all(&self) -> anyhow::Result<()>;
    fn list(&self, keyword: Option<String>) -> Result<Vec<KeyWithItem>, anyhow::Error>;
    fn done(&self, k: sled::IVec) -> Result<(), anyhow::Error>;
}

pub(crate) struct KeyWithItem {
    pub(crate) k: Key,
    pub(crate) v: Item,
}

impl Display for KeyWithItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.k.0, self.v))
    }
    
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Item {
    content: String,
    done: bool,
}

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.content.blue()))
    }
}

impl Into<sled::IVec> for Item {
    fn into(self) -> sled::IVec {
        serde_json::to_vec(&self).expect("convert item fail").into()
    }
}

impl TryFrom<sled::IVec> for Item {
    type Error = anyhow::Error;

    fn try_from(value: sled::IVec) -> Result<Self, Self::Error> {
        let res: Item = serde_json::from_str(std::str::from_utf8(&value)?)?;
        Ok(res)
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

static ERR_ADD_ITEM: &'static str ="add item fail";

static ERR_ITEM_NOT_FOUND: &'static str = "item not found";

impl Operate for Store {
    fn add(&self, item: Item) -> Result<u32, anyhow::Error> {
        match self.db.insert::<std::string::String, Item>(self.gen_key().to_string(), item) {
            Ok(_) => Ok(self.count()),
            Err(_) => Err(anyhow::anyhow!(ERR_ADD_ITEM))
        }
    }

    fn find_by_no(&self, no: usize) -> Option<(sled::IVec, Item)> {
        if let Some(Ok((k, v))) = self.db.into_iter().nth(no-1) {
            if let Ok(item) = v.try_into() {
                return Some((k, item));
            }
        }
        None
    }

    fn delete(&self, k: sled::IVec) -> Result<Item, anyhow::Error> {
        if let Ok(Some(item)) = self.db.remove(k) {
            return Ok(item.try_into()?);
        }
    
        Err(anyhow::anyhow!(ERR_ITEM_NOT_FOUND))
    }

    fn delete_all(&self) -> anyhow::Result<()> {
        Ok(self.db.clear()?)
    }

    fn list(&self, _: Option<String>) -> Result<Vec<KeyWithItem>, anyhow::Error> {
        self.db.into_iter()
        .filter_map(|res| {
            match res {
                Ok((key, val)) => {
                let res = String::from_utf8(key.to_vec());
                if res.is_err() {
                    return None
                }
                match Item::try_from(val) {
                    Ok(item) if !item.done => Some(Ok(KeyWithItem { k: res.unwrap().into(), v: item })),
                    _ => None
                }
                },
                Err(_) => None
            }
        }).collect()
    }

    fn done(&self, k: sled::IVec) -> Result<(), anyhow::Error> {
        if let Some(v) = self.db.get(k.clone())? {
            let mut item: Item = v.try_into()?;
            item.done = true;
            self.db.insert(k, item)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!(ERR_ITEM_NOT_FOUND))
        }
    }
}