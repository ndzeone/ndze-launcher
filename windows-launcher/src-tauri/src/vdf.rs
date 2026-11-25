//! Minimal parser for Valve KeyValues text files (libraryfolders.vdf, appmanifest_*.acf).

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Str(String),
    Obj(Vec<(String, Vdf)>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Obj(entries) => entries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Str(s) => Some(s),
            Vdf::Obj(_) => None,
        }
    }

    pub fn entries(&self) -> &[(String, Vdf)] {
        match self {
            Vdf::Obj(entries) => entries,
            Vdf::Str(_) => &[],
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Open,
    Close,
}

fn tokenize(src: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '{' => {
                chars.next();
                tokens.push(Token::Open);
            }
            '}' => {
                chars.next();
                tokens.push(Token::Close);
            }
            '/' => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            break;
                        }
                    }
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => s.push(other),
                            None => break,
                        },
                        other => s.push(other),
                    }
                }
                tokens.push(Token::Str(s));
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                tokens.push(Token::Str(s));
            }
        }
    }
    tokens
}

pub fn parse(src: &str) -> Vdf {
    let tokens = tokenize(src);
    let mut iter = tokens.into_iter().peekable();
    Vdf::Obj(parse_entries(&mut iter))
}

fn parse_entries(iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>) -> Vec<(String, Vdf)> {
    let mut entries = Vec::new();
    while let Some(token) = iter.next() {
        let key = match token {
            Token::Str(k) => k,
            Token::Close => break,
            Token::Open => continue,
        };
        match iter.next() {
            Some(Token::Str(v)) => entries.push((key, Vdf::Str(v))),
            Some(Token::Open) => entries.push((key, Vdf::Obj(parse_entries(iter)))),
            Some(Token::Close) | None => break,
        }
    }
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_library_folders() {
        let src = r#"
"libraryfolders"
{
    "0"
    {
        "path"      "C:\\Program Files (x86)\\Steam"
        "apps" { "730" "123" }
    }
    // comment
    "1" { "path" "D:\\SteamLibrary" }
}"#;
        let root = parse(src);
        let lf = root.get("LibraryFolders").unwrap();
        let paths: Vec<_> = lf
            .entries()
            .iter()
            .filter_map(|(_, v)| v.str("path"))
            .collect();
        assert_eq!(
            paths,
            vec![r"C:\Program Files (x86)\Steam", r"D:\SteamLibrary"]
        );
        assert_eq!(
            lf.get("0").unwrap().get("apps").unwrap().str("730"),
            Some("123")
        );
    }
}
