use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::error::Error;

use utils::{self, PathExt, FileManipulation};

use ::book::MDBook;

pub static INDEX: &'static [u8] = include_bytes!("index.hbs");
pub static CSS: &'static [u8] = include_bytes!("book.css");
pub static JS: &'static [u8] = include_bytes!("book.js");
pub static HIGHLIGHT_JS: &'static [u8] = include_bytes!("highlight.js");
pub static TOMORROW_NIGHT_CSS: &'static [u8] = include_bytes!("tomorrow-night.css");
pub static HIGHLIGHT_CSS: &'static [u8] = include_bytes!("highlight.css");
pub static JQUERY: &'static [u8] = include_bytes!("jquery-2.1.4.min.js");
pub static FONT_AWESOME: &'static [u8] = include_bytes!("_FontAwesome/css/font-awesome.min.css");
pub static FONT_AWESOME_EOT: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.eot");
pub static FONT_AWESOME_SVG: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.svg");
pub static FONT_AWESOME_TTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.ttf");
pub static FONT_AWESOME_WOFF: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff");
pub static FONT_AWESOME_WOFF2: &'static [u8] = include_bytes!("_FontAwesome/fonts/fontawesome-webfont.woff2");
pub static FONT_AWESOME_OTF: &'static [u8] = include_bytes!("_FontAwesome/fonts/FontAwesome.otf");

/// The `Theme` struct should be used instead of the static variables because the `new()` method
/// will look if the user has a theme directory in his source folder and use the users theme instead
/// of the default.
///
/// You should exceptionnaly use the static variables only if you need the default theme even if the
/// user has specified another theme.
pub struct Theme {
    pub index: Vec<u8>,
    pub css: Vec<u8>,
    pub js: Vec<u8>,
    pub highlight_css: Vec<u8>,
    pub tomorrow_night_css: Vec<u8>,
    pub highlight_js: Vec<u8>,
    pub jquery: Vec<u8>,
}

impl Theme {
    pub fn new(src: &Path) -> Self {

        // Default theme
        let mut theme = Theme {
            index: INDEX.to_owned(),
            css: CSS.to_owned(),
            js: JS.to_owned(),
            highlight_css: HIGHLIGHT_CSS.to_owned(),
            tomorrow_night_css: TOMORROW_NIGHT_CSS.to_owned(),
            highlight_js: HIGHLIGHT_JS.to_owned(),
            jquery: JQUERY.to_owned(),
        };

        // Check if the given path exists
        if !src.exists() || !src.is_dir() {
            return theme
        }

        let src = src.join("theme");
        // If src does exist, check if there is a theme directory in it
        if !src.exists() || !src.is_dir() {
            return theme
        }

        // Check for individual files if they exist

        // index.hbs
        if let Ok(mut f) = File::open(&src.join("index.hbs")) {
            theme.index.clear(); // Reset the value, because read_to_string appends...
            let _ = f.read_to_end(&mut theme.index);
        }

        // book.js
        if let Ok(mut f) = File::open(&src.join("book.js")) {
            theme.js.clear();
            let _ =  f.read_to_end(&mut theme.js);
        }

        // book.css
        if let Ok(mut f) = File::open(&src.join("book.css")) {
            theme.css.clear();
            let _ = f.read_to_end(&mut theme.css);
        }

        // highlight.js
        if let Ok(mut f) = File::open(&src.join("highlight.js")) {
            theme.highlight_js.clear();
            let _ = f.read_to_end(&mut theme.highlight_js);
        }

        // highlight.css
        if let Ok(mut f) = File::open(&src.join("highlight.css")) {
            theme.highlight_css.clear();
            let _ = f.read_to_end(&mut theme.highlight_css);
        }

        // tomorrow-night.css
        if let Ok(mut f) = File::open(&src.join("tomorrow-night.css")) {
            theme.tomorrow_night_css.clear();
            let _ = f.read_to_end(&mut theme.tomorrow_night_css);
        }

        theme
    }
}

pub fn copy_static_files(book: &MDBook) -> Result<(), Box<Error>> {

    // Load theme
    let theme = Theme::new(book.get_src());
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
    try!(base.join("_FontAwesome/css/font-awesome.css").create_write(&FONT_AWESOME));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.eot").create_write(&FONT_AWESOME_EOT));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.svg").create_write(&FONT_AWESOME_SVG));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.ttf").create_write(&FONT_AWESOME_TTF));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.woff").create_write(&FONT_AWESOME_WOFF));
    try!(base.join("_FontAwesome/fonts/fontawesome-webfont.woff2").create_write(&FONT_AWESOME_WOFF2));
    try!(base.join("_FontAwesome/fonts/FontAwesome.otf").create_write(&FONT_AWESOME_OTF));
    // syntax highlighting
    try!(base.join("highlight.css").create_write(&theme.highlight_css));
    try!(base.join("tomorrow-night.css").create_write(&theme.tomorrow_night_css));
    try!(base.join("highlight.js").create_write(&theme.highlight_js));

    // Copy all remaining files
    try!(utils::copy_files_except_ext(book.get_src(), book.get_dest(), true, &["md"]));

    Ok(())
}

pub fn copy_theme(book: &::book::MDBook) -> Result<(), Box<Error>> {

    debug!("[fn]: copy_theme");

    let theme_dir = book.get_src().join("theme");

    if !theme_dir.exists() {
        debug!("[*]: {:?} does not exist, trying to create directory", theme_dir);
        try!(::std::fs::create_dir(&theme_dir));
    }

    // index.hbs
    let mut index = try!(File::create(&theme_dir.join("index.hbs")));
    try!(index.write_all(INDEX));

    // book.css
    let mut css = try!(File::create(&theme_dir.join("book.css")));
    try!(css.write_all(CSS));

    // book.js
    let mut js = try!(File::create(&theme_dir.join("book.js")));
    try!(js.write_all(JS));

    // highlight.css
    let mut highlight_css = try!(File::create(&theme_dir.join("highlight.css")));
    try!(highlight_css.write_all(HIGHLIGHT_CSS));

    // highlight.js
    let mut highlight_js = try!(File::create(&theme_dir.join("highlight.js")));
    try!(highlight_js.write_all(HIGHLIGHT_JS));

    Ok(())

}
