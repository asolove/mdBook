extern crate rustc_serialize;
use self::rustc_serialize::json::{Json, ToJson};

use std::collections::BTreeMap;
use std::path::Path;
use std::error::Error;
use std::io;

use ::book::MDBook;
use book::bookitem::BookItem;
use utils;

pub fn create_context(book: &MDBook) -> Result<BTreeMap<String,Json>, Box<Error>> {
    debug!("[fn]: create_context");

    let mut context  = BTreeMap::new();
    context.insert("language".to_owned(), "en".to_json());
    context.insert("title".to_owned(), book.get_title().to_json());

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the context to inject in the template
        let mut chapter = BTreeMap::new();

        match *item {
            BookItem::Affix(ref ch) => {
                chapter.insert("name".to_owned(), ch.name.to_json());
                match ch.path.to_str() {
                    Some(p) => { chapter.insert("path".to_owned(), p.to_json()); },
                    None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))),
                }
            },
            BookItem::Chapter(ref s, ref ch) => {
                chapter.insert("section".to_owned(), s.to_json());
                chapter.insert("name".to_owned(), ch.name.to_json());
                match ch.path.to_str() {
                    Some(p) => { chapter.insert("path".to_owned(), p.to_json()); },
                    None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))),
                }
            },
            BookItem::Spacer => {
                chapter.insert("spacer".to_owned(), "_spacer_".to_json());
            }

        }

        chapters.push(chapter);
    }

    context.insert("chapters".to_owned(), chapters.to_json());

    debug!("[*]: JSON constructed");
    Ok(context)
}

pub fn extend_context(context: &BTreeMap<String,Json>, content: &str, path: &Path) -> Result<BTreeMap<String,Json>, Box<Error>> {
    let mut context = context.clone();

    context.insert("content".to_owned(), content.to_json());
    context.insert("path".to_owned(), path.to_str().expect("in extend_context: path should exist").to_json());
    context.insert("path_to_root".to_owned(), utils::path_to_root(path).to_json());

    Ok(context)
}
