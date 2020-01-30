use std::path::{Path, PathBuf};

use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Result;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};

pub struct OpenOn;

impl Preprocessor for OpenOn {
    fn name(&self) -> &str {
        "open-on-gh"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let book_root = &ctx.root;
        let git_root = find_git(book_root).unwrap();
        log::debug!("Book root: {}", book_root.display());
        log::debug!("Git root: {}", git_root.display());

        let repository_url = match ctx.config.get("output.html.git-repository-url") {
            None => return Ok(book),
            Some(url) => url,
        };
        let repository_url = match repository_url {
            toml::Value::String(s) => s,
            _ => return Ok(book),
        };
        log::debug!("Repository URL: {}", repository_url);

        if repository_url.find("github.com").is_none() {
            return Ok(book)
        }

        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(open_on(&git_root, book_root, &repository_url, chapter).map(|md| {
                    chapter.content = md;
                }));
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }
}

fn open_on(git_root: &Path, book_root: &Path, base_url: &str, chapter: &mut Chapter) -> Result<String> {
    let content = &chapter.content;
    let path = book_root.join(&chapter.path);
    let relpath = path.strip_prefix(git_root).unwrap();

    let link = format!("<a href=\"{}/edit/master/{}\">Edit this file on GitHub.</a>", base_url, relpath.display());
    let content = format!("{}\n<footer id=\"open-on-gh\">Found a bug? {}</footer>", content, link);

    Ok(content)
}

fn find_git(path: &Path) -> Option<PathBuf> {
    let mut current_path = path;
    let mut git_dir = current_path.join(".git");
    let root = Path::new("/");

    while !git_dir.exists() {
        current_path = match current_path.parent() {
            Some(p) => p,
            None => return None
        };

        if current_path == root {
            return None;
        }

        git_dir = current_path.join(".git");
    }

    git_dir.parent().map(|p| p.to_owned())
}
