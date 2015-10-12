use super::handlebars::{Handlebars, JsonRender};
use super::pulldown_cmark::{Parser, html};

use std::path::{Path, PathBuf};
use std::fs::File;
use std::error::Error;
use std::io::{self, Read, Write};

use super::{context, helpers, theme};
use renderer::Renderer;
use book::MDBook;
use book::bookitem::BookItem;
use utils;


pub struct HtmlHandlebars;

impl HtmlHandlebars {
    pub fn new() -> Self {
        HtmlHandlebars
    }
}

impl Renderer for HtmlHandlebars {
    fn render(&self, book: &MDBook) -> Result<(), Box<Error>> {
        debug!("[fn]: render");

        // Check if destination directory exists
        debug!("[*]: Check if destination directory exists");
        if let Err(_) = utils::create_path(book.get_dest()) {
            return Err(Box::new(io::Error::new(io::ErrorKind::Other, "Unexpected error when constructing destination path")))
        }

        // Load theme
        let theme = theme::Theme::new(book.get_src());


        let mut handlebars = Handlebars::new();

        // Register template
        debug!("[*]: Register handlebars template");
        try!(handlebars.register_template_string("index", try!(String::from_utf8(theme.index))));

        // Register helpers
        debug!("[*]: Register handlebars helpers");
        handlebars.register_helper("toc", Box::new(helpers::toc::RenderToc));
        handlebars.register_helper("previous", Box::new(helpers::navigation::previous));
        handlebars.register_helper("next", Box::new(helpers::navigation::next));


        // Variables for print and index file
        let mut print_content: String = String::new();
        let mut index = true;


        // Common Context for all handlebars template
        let context = try!(context::create_context(book));


        // Render a file for every entry in the book
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


                        // Extend the common context (above) to include chapter specific info
                        let chapter_context = try!(context::extend_context(&context, &content, &ch.path));


                        // Rendere the handlebars template with the data
                        debug!("[*]: Render template");
                        let rendered = try!(handlebars.render("index", &chapter_context));


                        // Write to file
                        debug!("[*]: Create file {:?}", &book.get_dest().join(&ch.path).with_extension("html"));
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
        let print_context = try!(context::extend_context(&context, &print_content, &Path::new("print.md")));

        // Rendere the handlebars template with the data
        debug!("[*]: Render template");
        let rendered = try!(handlebars.render("index", &print_context));
        let mut file = try!(utils::create_file(&book.get_dest().join("print").with_extension("html")));
        try!(file.write_all(&rendered.into_bytes()));
        output!("[*] Creating print.html ✓");

        try!(theme::copy_static_files(book));

        Ok(())
    }

    fn copy_theme(&self, book: &::book::MDBook) -> Result<(), Box<Error>> {
        debug!("[fn]: copy_theme");
        try!(theme::copy_theme(book));
        Ok(())
    }
}

fn render_html(text: &str) -> String {
    let mut s = String::with_capacity(text.len() * 3 / 2);
    let p = Parser::new(&text);
    html::push_html(&mut s, p);
    s
}
