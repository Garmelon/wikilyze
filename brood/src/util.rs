use std::{fmt, iter, thread::panicking, time::Instant};

use regex::Regex;
use thousands::Separable;

use crate::{
    data::{Data, Page},
    graph::{Graph, NodeIdx},
};

pub struct Counter {
    n: usize,
    last_print: Instant,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            n: 0,
            last_print: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.n += 1;
        if self.n % 10_000 != 0 {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_print).as_secs() < 4 {
            return;
        }

        println!("{:>12}", self.n.separate_with_underscores());
        self.last_print = now;
    }

    pub fn done(&self) {
        println!("{:>12} (done)", self.n.separate_with_underscores());
    }
}

// https://github.com/wikimedia/mediawiki-title/blob/6880ae1a9ffdfa2eea9fd75b472493a67dabcc48/lib/mediawiki.Title.phpCharToUpper.js
struct PhpCharToUpper(char);

impl fmt::Display for PhpCharToUpper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            // Do something special, I guess
            'ᾀ' => write!(f, "ᾈ"),
            'ᾁ' => write!(f, "ᾉ"),
            'ᾂ' => write!(f, "ᾊ"),
            'ᾃ' => write!(f, "ᾋ"),
            'ᾄ' => write!(f, "ᾌ"),
            'ᾅ' => write!(f, "ᾍ"),
            'ᾆ' => write!(f, "ᾎ"),
            'ᾇ' => write!(f, "ᾏ"),
            'ᾐ' => write!(f, "ᾘ"),
            'ᾑ' => write!(f, "ᾙ"),
            'ᾒ' => write!(f, "ᾚ"),
            'ᾓ' => write!(f, "ᾛ"),
            'ᾔ' => write!(f, "ᾜ"),
            'ᾕ' => write!(f, "ᾝ"),
            'ᾖ' => write!(f, "ᾞ"),
            'ᾗ' => write!(f, "ᾟ"),
            'ᾠ' => write!(f, "ᾨ"),
            'ᾡ' => write!(f, "ᾩ"),
            'ᾢ' => write!(f, "ᾪ"),
            'ᾣ' => write!(f, "ᾫ"),
            'ᾤ' => write!(f, "ᾬ"),
            'ᾥ' => write!(f, "ᾭ"),
            'ᾦ' => write!(f, "ᾮ"),
            'ᾧ' => write!(f, "ᾯ"),
            'ᾳ' => write!(f, "ᾼ"),
            'ῃ' => write!(f, "ῌ"),
            'ῳ' => write!(f, "ῼ"),

            // Do not capitalize
            'ß' | 'ŉ' | 'ǰ' | 'ʂ' | 'ͅ' | 'ΐ' | 'ΰ' | 'և' | 'ა' | 'ბ' | 'გ' | 'დ' | 'ე' | 'ვ'
            | 'ზ' | 'თ' | 'ი' | 'კ' | 'ლ' | 'მ' | 'ნ' | 'ო' | 'პ' | 'ჟ' | 'რ' | 'ს' | 'ტ' | 'უ'
            | 'ფ' | 'ქ' | 'ღ' | 'ყ' | 'შ' | 'ჩ' | 'ც' | 'ძ' | 'წ' | 'ჭ' | 'ხ' | 'ჯ' | 'ჰ' | 'ჱ'
            | 'ჲ' | 'ჳ' | 'ჴ' | 'ჵ' | 'ჶ' | 'ჷ' | 'ჸ' | 'ჹ' | 'ჺ' | 'ჽ' | 'ჾ' | 'ჿ' | 'ᶎ' | 'ẖ'
            | 'ẗ' | 'ẘ' | 'ẙ' | 'ẚ' | 'ὐ' | 'ὒ' | 'ὔ' | 'ὖ' | 'ᾈ' | 'ᾉ' | 'ᾊ' | 'ᾋ' | 'ᾌ' | 'ᾍ'
            | 'ᾎ' | 'ᾏ' | 'ᾘ' | 'ᾙ' | 'ᾚ' | 'ᾛ' | 'ᾜ' | 'ᾝ' | 'ᾞ' | 'ᾟ' | 'ᾨ' | 'ᾩ' | 'ᾪ' | 'ᾫ'
            | 'ᾬ' | 'ᾭ' | 'ᾮ' | 'ᾯ' | 'ᾲ' | 'ᾴ' | 'ᾶ' | 'ᾷ' | 'ᾼ' | 'ῂ' | 'ῄ' | 'ῆ' | 'ῇ' | 'ῌ'
            | 'ῒ' | 'ΐ' | 'ῖ' | 'ῗ' | 'ῢ' | 'ΰ' | 'ῤ' | 'ῦ' | 'ῧ' | 'ῲ' | 'ῴ' | 'ῶ' | 'ῷ' | 'ῼ'
            | 'ⅰ' | 'ⅱ' | 'ⅲ' | 'ⅳ' | 'ⅴ' | 'ⅵ' | 'ⅶ' | 'ⅷ' | 'ⅸ' | 'ⅹ' | 'ⅺ' | 'ⅻ' | 'ⅼ' | 'ⅽ'
            | 'ⅾ' | 'ⅿ' | 'ⓐ' | 'ⓑ' | 'ⓒ' | 'ⓓ' | 'ⓔ' | 'ⓕ' | 'ⓖ' | 'ⓗ' | 'ⓘ' | 'ⓙ' | 'ⓚ' | 'ⓛ'
            | 'ⓜ' | 'ⓝ' | 'ⓞ' | 'ⓟ' | 'ⓠ' | 'ⓡ' | 'ⓢ' | 'ⓣ' | 'ⓤ' | 'ⓥ' | 'ⓦ' | 'ⓧ' | 'ⓨ' | 'ⓩ'
            | 'ꞔ' | 'ꞹ' | 'ꞻ' | 'ꞽ' | 'ꞿ' | 'ꟃ' | 'ﬀ' | 'ﬁ' | 'ﬂ' | 'ﬃ' | 'ﬄ' | 'ﬅ' | 'ﬆ' | 'ﬓ'
            | 'ﬔ' | 'ﬕ' | 'ﬖ' | 'ﬗ' | '𖹠' | '𖹡' | '𖹢' | '𖹣' | '𖹤' | '𖹥' | '𖹦' | '𖹧' | '𖹨' | '𖹩'
            | '𖹪' | '𖹫' | '𖹬' | '𖹭' | '𖹮' | '𖹯' | '𖹰' | '𖹱' | '𖹲' | '𖹳' | '𖹴' | '𖹵' | '𖹶' | '𖹷'
            | '𖹸' | '𖹹' | '𖹺' | '𖹻' | '𖹼' | '𖹽' | '𖹾' | '𖹿' => {
                write!(f, "{}", self.0)
            }

            // Capitalize normally
            c => write!(f, "{}", c.to_uppercase()),
        }
    }
}

pub struct TitleNormalizer {
    strip_bidi: Regex,
    clean_up_whitespace: Regex,
    trim_underscore_start: Regex,
    trim_underscore_end: Regex,
}

impl TitleNormalizer {
    pub fn new() -> Self {
        Self {
            strip_bidi: Regex::new("[\u{200E}\u{200F}\u{202A}-\u{202E}]").unwrap(),

            clean_up_whitespace: Regex::new(concat!(
                "[ _\u{00A0}\u{1680}\u{180E}\u{2000}-\u{200A}",
                "\u{2028}\u{2029}\u{202F}\u{205F}\u{3000}]+"
            ))
            .unwrap(),

            trim_underscore_start: Regex::new("^_+").unwrap(),

            trim_underscore_end: Regex::new("_+$").unwrap(),
        }
    }

    /// Normalize an article title.
    ///
    /// See also <https://github.com/wikimedia/mediawiki-title>.
    pub fn normalize(&self, title: &str) -> String {
        // https://github.com/wikimedia/mediawiki-title/blob/6880ae1a9ffdfa2eea9fd75b472493a67dabcc48/lib/index.js#L403

        // Strip Unicode bidi override characters
        let title = self.strip_bidi.replace_all(title, "");

        // Clean up whitespace
        let title = self.clean_up_whitespace.replace_all(&title, "_");

        // Trim _ from beginning and end
        let title = self.trim_underscore_start.replace_all(&title, "");
        let title = self.trim_underscore_end.replace_all(&title, "");

        // https://github.com/wikimedia/mediawiki-title/blob/6880ae1a9ffdfa2eea9fd75b472493a67dabcc48/lib/index.js#L206
        let Some(first) = title.chars().next() else {
            return String::new();
        };
        let rest = &title[first.len_utf8()..];
        format!("{}{rest}", PhpCharToUpper(first))
    }
}

pub fn locate_title(normalizer: &TitleNormalizer, data: &Data, title: &str) -> NodeIdx {
    let normalized = normalizer.normalize(title);
    data.pages
        .iter()
        .enumerate()
        .find(|(_, p)| normalizer.normalize(&p.title) == normalized)
        .map(|(i, _)| NodeIdx::new(i))
        .expect("invalid title")
}

pub fn resolve_redirects(data: &Data, mut page: NodeIdx) -> NodeIdx {
    while let Some(target) = data.redirect_target(page) {
        page = target;
    }
    page
}

pub fn resolve_title(normalizer: &TitleNormalizer, data: &Data, title: &str) -> NodeIdx {
    resolve_redirects(data, locate_title(normalizer, data, title))
}

pub fn fmt_page(page: &Page) -> String {
    if page.redirect {
        format!("v {}", page.title)
    } else {
        format!("- {}", page.title)
    }
}
