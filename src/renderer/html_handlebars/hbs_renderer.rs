extern crate handlebars;
extern crate rustc_serialize;
extern crate pulldown_cmark;

use super::{helpers, theme};
use renderer::Renderer;
use book::MDBook;
use book::bookitem::BookItem;
use utils::{self, PathExt, FileManipulation};

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::error::Error;
use std::io::{self, Read, Write};
use std::collections::BTreeMap;

use self::handlebars::{Handlebars, JsonRender};
use self::rustc_serialize::json::{Json, ToJson};
use self::pulldown_cmark::{Parser, html};

pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }
}

impl Renderer for HtmlHandlebars {
    fn render(&self, book: &MDBook) -> Result<(), Box<Error>> {
        debug!("[fn]: render");
        let mut handlebars = Handlebars::new();

        // Load theme
        let theme = theme::Theme::new(book.get_src());

        // Register template
        debug!("[*]: Register handlebars template");
        try!(handlebars.register_template_string("index", try!(String::from_utf8(theme.index))));

        // Register helpers
        debug!("[*]: Register handlebars helpers");
        handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));

        let mut data = try!(make_data(book));

        // Print version
        let mut print_content: String = String::new();

        // Check if dest directory exists
        debug!("[*]: Check if destination directory exists");
        if let Err(_) = utils::create_path(book.get_dest()) {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Unexpected error when constructing destination path")))
        }

        // Render a file for every entry in the book
        let mut index = true;
        for item in book.iter() {

            match *item {
                BookItem::Chapter(_, ref ch) | BookItem::Affix(ref ch) => {
                    if ch.path != PathBuf::new() {

                        let path = book.get_src().join(&ch.path);

                        debug!("[*]: Opening file: {:?}", path);
                        let mut f = try!(File::open(&path));
                        let mut content: String = String::new();

                        debug!("[*]: Reading file");
                        try!(f.read_to_string(&mut content));

                        // Render markdown using the pulldown-cmark crate
                        content = render_html(&content);
                        print_content.push_str(&content);

                        // Remove content from previous file and render content for this one
                        data.remove("path");
                        match ch.path.to_str() {
                            Some(p) => { data.insert("path".to_owned(), p.to_json()); },
                            None => return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Could not convert path to str"))),
                        }


                        // Remove content from previous file and render content for this one
                        data.remove("content");
                        data.insert("content".to_owned(), content.to_json());

                        // Remove path to root from previous file and render content for this one
                        data.remove("path_to_root");
                        data.insert("path_to_root".to_owned(), utils::path_to_root(&ch.path).to_json());

                        // Rendere the handlebars template with the data
                        debug!("[*]: Render template");
                        let rendered = try!(handlebars.render("index", &data));

                        debug!("[*]: Create file {:?}", &book.get_dest().join(&ch.path).with_extension("html"));
                        // Write to file
                        let mut file = try!(utils::create_file(&book.get_dest().join(&ch.path).with_extension("html")));
                        output!("[*] Creating {:?} ✓", &book.get_dest().join(&ch.path).with_extension("html"));

                        try!(file.write_all(&rendered.into_bytes()));

                        // Create an index.html from the first element in SUMMARY.md
                        if index {
                            debug!("[*]: index.html");

                            let mut index_file = try!(File::create(book.get_dest().join("index.html")));
                            let mut content = String::new();
                            let _source = try!(File::open(book.get_dest().join(&ch.path.with_extension("html"))))
                                                        .read_to_string(&mut content);

                            // This could cause a problem when someone displays code containing <base href=...>
                            // on the front page, however this case should be very very rare...
                            content = content.lines().filter(|line| !line.contains("<base href=")).collect();

                            try!(index_file.write_all(content.as_bytes()));

                            output!(
                                "[*] Creating index.html from {:?} ✓",
                                book.get_dest().join(&ch.path.with_extension("html"))
                                );
                            index = false;
                        }
                    }
                }
                _ => {}
            }
        }

        // Print version

        // Remove content from previous file and render content for this one
        data.remove("path");
        data.insert("path".to_owned(), "print.md".to_json());

        // Remove content from previous file and render content for this one
        data.remove("content");
        data.insert("content".to_owned(), print_content.to_json());

        // Remove path to root from previous file and render content for this one
        data.remove("path_to_root");
        data.insert("path_to_root".to_owned(), utils::path_to_root(Path::new("print.md")).to_json());

        // Rendere the handlebars template with the data
        debug!("[*]: Render template");
        let rendered = try!(handlebars.render("index", &data));
        let mut file = try!(utils::create_file(&book.get_dest().join("print").with_extension("html")));
        try!(file.write_all(&rendered.into_bytes()));
        output!("[*] Creating print.html ✓");

        try!(copy_static_files(book));

        Ok(())
    }

    fn copy_theme(&self, book: &::book::MDBook) -> Result<(), Box<Error>> {

        debug!("[fn]: copy_theme");

        let theme_dir = book.get_src().join("theme");

        if !theme_dir.exists() {
            debug!("[*]: {:?} does not exist, trying to create directory", theme_dir);
            try!(fs::create_dir(&theme_dir));
        }

        // index.hbs
        let mut index = try!(File::create(&theme_dir.join("index.hbs")));
        try!(index.write_all(theme::INDEX));

        // book.css
        let mut css = try!(File::create(&theme_dir.join("book.css")));
        try!(css.write_all(theme::CSS));

        // book.js
        let mut js = try!(File::create(&theme_dir.join("book.js")));
        try!(js.write_all(theme::JS));

        // highlight.css
        let mut highlight_css = try!(File::create(&theme_dir.join("highlight.css")));
        try!(highlight_css.write_all(theme::HIGHLIGHT_CSS));

        // highlight.js
        let mut highlight_js = try!(File::create(&theme_dir.join("highlight.js")));
        try!(highlight_js.write_all(theme::HIGHLIGHT_JS));

        Ok(())

    }
}

fn copy_static_files(book: &MDBook) -> Result<(), Box<Error>> {

    // Load theme
    let theme = theme::Theme::new(book.get_src());
    let base = book.get_dest();

    // Copy static files (js, css, images, ...)
    debug!("[*] Copy static files");

    // JavaScript
    try!(base.join("book.js").create_write(&theme.js));
    // Css
    try!(base.join("book.css").create_write(&theme.css));
    // JQuery local fallback
    try!(base.join("jquery.js").create_write(&theme.jquery));
    // Font Awesome local fallback
    try!(base.join("_FontAwesome/css/font-awesome.css").create_write(&theme::FONT_AWESOME));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.eot").create_write(&theme::FONT_AWESOME_EOT));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.svg").create_write(&theme::FONT_AWESOME_SVG));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.ttf").create_write(&theme::FONT_AWESOME_TTF));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.woff").create_write(&theme::FONT_AWESOME_WOFF));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.woff2").create_write(&theme::FONT_AWESOME_WOFF2));
    try!(base.join("_FontAwesome/fonts/FontAwesome.otf").create_write(&theme::FONT_AWESOME_OTF));
    // syntax highlighting
    try!(base.join("highlight.css").create_write(&theme.highlight_css));
    try!(base.join("tomorrow-night.css").create_write(&theme.tomorrow_night_css));
    try!(base.join("highlight.js").create_write(&theme.highlight_js));

    // Copy all remaining files
    try!(utils::copy_files_except_ext(book.get_src(), book.get_dest(), true, &["md"]));

    Ok(())
}

fn make_data(book: &MDBook) -> Result<BTreeMap<String,Json>, Box<Error>> {
    debug!("[fn]: make_data");

    let mut data  = BTreeMap::new();
    data.insert("language".to_owned(), "en".to_json());
    data.insert("title".to_owned(), book.get_title().to_json());

    let mut chapters = vec![];

    for item in book.iter() {
        // Create the data to inject in the template
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

    data.insert("chapters".to_owned(), chapters.to_json());

    debug!("[*]: JSON constructed");
    Ok(data)
}

fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    s
}
