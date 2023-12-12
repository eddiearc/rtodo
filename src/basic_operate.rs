use std::{env, time::{UNIX_EPOCH, SystemTime}, io, fmt::Display};

use colored::Colorize;
use serde_derive::{Serialize, Deserialize};

use crate::output;



pub(crate) fn process(args: super::Args, store: impl Operate) -> anyhow::Result<()> {
    if args.delete_all {
        let _ =delete_all(&store)?;
    } else if let Some(key) = args.delete {
        let _ = delete(&store, key)?;
    } else if let Some(key) = args.done {
        let _ = done(&store, key.into())?;
    }
    if let Some(content) =  args.add {
        let _ = store.add(Item { content, done: false })?;
    }
    list_items(&store, args.list);
    Ok(())
}

fn delete_all(store: &impl Operate) -> anyhow::Result<()> {
    if !confirm(format!("confirm delete all items? (Y/N)")) {
        return Err(anyhow::anyhow!("cancel delete all items"))
    }
    store.delete_all()
}

fn delete(store: &impl Operate, key: Key) -> anyhow::Result<()> {
    if let Some(v) = store.find_by_key(key.clone().into()) {
        // todo confirm delete operate
        if !confirm(format!("confirm delete this item: {}: {}? (Y/N)", key, v.content)) {
            return Err(anyhow::anyhow!("cancel delete item"))
        }
        // exec delete
        if let Err(msg) = store.delete(key.into()) {
            return Err(anyhow::anyhow!(msg))   
        }
        Ok(())
    } else {
        return Err(anyhow::anyhow!(ERR_ITEM_NOT_FOUND))
    }
}

fn done(store: &impl Operate, key: Key) -> anyhow::Result<()> {
    if let Some(v) = store.find_by_key(key.clone().into()) {
        if v.done {
            return Err(anyhow::anyhow!("item already done"))
        }
        // todo confirm done operate
        if !confirm(format!("confirm done this item: {}: {}? (Y/N)", key, v.content)) {
            return Err(anyhow::anyhow!("cancel done item"))
        }
        // exec done
        if let Err(msg) = store.done(key.into()) {
            return Err(anyhow::anyhow!(msg))
        }
        return Ok(())
    } else {
        return Err(anyhow::anyhow!(ERR_ITEM_NOT_FOUND))
    }
}

fn list_items(store: &impl Operate, keyword: Option<String>) {
    for (_, ele) in store.list(keyword)
        .expect("list items error").iter().enumerate() {
        output::list_print( ele);
    }
}

fn confirm(hits: String) -> bool {
    output::info(hits);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    return input.trim().to_lowercase() == "y"
}

#[derive(Debug)]
pub struct Key(String);


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

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Clone for Key {
    fn clone(&self) -> Self {
        Key(self.0.clone())
    }
}

pub(crate) trait Operate {
    fn add(&self, item: Item) -> Result<u32, anyhow::Error>;
    fn find_by_key(&self, key: sled::IVec) -> Option<Item>;
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
        f.write_fmt(format_args!("{}: {}", self.k, self.v))
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

    fn find_by_key(&self, key: sled::IVec) -> Option<Item> {
        if let Ok(Some(v)) = self.db.get(key) {
            if let Ok(item) = v.try_into() {
                return Some(item);
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