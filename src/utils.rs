// use {
//     alloc::string::String,
//     core::{fmt, ops::Range},
//     sptr::Strict,
// };
use core::fmt::{self, Write};

pub(crate) struct Fmt<F>(pub(crate) F)
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

impl<F> fmt::Display for Fmt<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}
impl<F> fmt::Debug for Fmt<F>
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.0)(f)
    }
}

// macro_rules! unreachable_unsafe {
//     ($($args:tt)*) => {
//         if cfg!(debug_assertions) {
//             unreachable!($($args)*)
//         } else {
//             ::core::hint::unreachable_unchecked()
//         }
//     };
// }
// pub(crate) use unreachable_unsafe;

// macro_rules! display {
//     ($lit:literal $($tt:tt)*) => {
//         $crate::utils::Display(move |f| {
//             use core::fmt::Write as _;
//             write!(f, $lit $($tt)*)
//         })
//     };
// }
// pub(crate) use display;

// #[allow(unstable_name_collisions)]
// pub(crate) fn locate(outer: &str, inner: &str) -> Range<usize> {
//     let offset =
//         (inner as *const str as *const u8).addr() - (outer as *const str as *const u8).addr();
//     offset..offset + inner.len()
// }

pub(crate) fn unescape(src: &str) -> impl '_ + fmt::Display {
    Fmt(move |f| {
        let mut src = src;
        loop {
            match src.find('\\') {
                None => {
                    f.write_str(src)?;
                    break;
                }
                Some(i) => {
                    f.write_str(&src[..i])?;
                    src = &src[i + 1..];
                    match src.as_bytes().first().ok_or(fmt::Error)? {
                        b'n' => f.write_char('\n')?,
                        b'r' => f.write_char('\r')?,
                        b't' => f.write_char('\t')?,
                        b'\\' => f.write_char('\\')?,
                        b'"' => f.write_char('"')?,
                        b'b' => f.write_char('\u{8}')?,
                        b'f' => f.write_char('\u{C}')?,
                        b'u' if src.as_bytes().get(1) == Some(&b'{') => match src.find('}') {
                            None => return Err(fmt::Error),
                            Some(ix) => {
                                let bracketed = &src[2..ix];
                                src = &src[ix..];
                                let codepoint =
                                    u32::from_str_radix(bracketed, 16).map_err(|_| fmt::Error)?;
                                let ch = char::from_u32(codepoint).ok_or(fmt::Error)?;
                                f.write_char(ch)?;
                            }
                        },
                        _ => return Err(fmt::Error),
                    }
                    src = &src[1..];
                }
            }
        }
        Ok(())
    })
}

// pub(crate) fn valid_bareword(s: &str) -> bool {
//     fn looks_like_number(s: &str) -> bool {
//         s.starts_with(|c: char| c.is_ascii_digit())
//             || (s.starts_with(['+', '-'])
//                 && s.chars().nth(1).map_or(false, |c: char| c.is_ascii_digit()))
//     }

//     fn looks_like_string(s: &str) -> bool {
//         s.starts_with(r#"r""#) || s.starts_with(r#"r#"#)
//     }

//     fn is_keyword(s: &str) -> bool {
//         matches!(s, "true" | "false" | "null")
//     }

//     !(s.contains([
//         '\\', '/', '(', ')', '{', '}', '<', '>', ';', '[', ']', '=', ',', '"',
//     ]) || is_keyword(s)
//         || looks_like_string(s)
//         || looks_like_number(s))
// }

// pub(crate) fn bareword(s: &str) -> impl '_ + fmt::Display {
//     display!(|f| {
//         if valid_bareword(s) {
//             write!(f, "{s}")
//         } else {
//             write!(f, "{}", escape(s))
//         }
//     })
// }

// pub(crate) fn escape(src: &str) -> impl '_ + fmt::Display {
//     #[allow(unreachable_patterns)] // clarity
//     fn should_escape(c: char) -> bool {
//         match c {
//             // KDL Escape Sequences should always be escaped
//             '\n' | '\r' | '\t' | '\\' | '/' | '"' | '\u{8}' | '\u{C}' => true,
//             // ASCII printable characters should never be escaped
//             '\u{20}'..='\u{7e}' => false,
//             // Other KDL Whitespace should never be escaped
//             // '\t' => true (above)
//             ' '
//             | '\u{A0}'
//             | '\u{1600}'
//             | '\u{2000}'..='\u{200A}'
//             | '\u{202F}'
//             | '\u{205F}'
//             | '\u{3000}' => false,
//             // KDL Newline should always be escaped
//             '\r' | '\n' | '\u{85}' | '\u{C}' | '\u{2028}' | '\u{2029}' => true,
//             // Aproximate "printable" unicode characters to avoid overescaping
//             c if c.is_alphanumeric() => false,
//             _ => true,
//         }
//     }

//     display!(|f| {
//         let mut src = src;
//         write!(f, "\"")?;
//         while let Some(next_escaped) = src.find(should_escape) {
//             write!(f, "{}", &src[..next_escaped])?;
//             src = &src[next_escaped..];
//             let escaped = src.chars().next().unwrap();
//             match escaped {
//                 '\r' => write!(f, r"\r")?,
//                 '\n' => write!(f, r"\n")?,
//                 '\t' => write!(f, r"\t")?,
//                 '\\' => write!(f, r"\\")?,
//                 '/' => write!(f, r"\/")?,
//                 '"' => write!(f, r#"\""#)?,
//                 '\u{8}' => write!(f, r"\b")?,
//                 '\u{C}' => write!(f, r"\f")?,
//                 c => write!(f, r"\u{:04X}", c as u32)?,
//             }
//             src = &src[escaped.len_utf8()..];
//         }
//         write!(f, "{}\"", src)
//     })
// }

// pub(crate) fn escape_raw(src: &str) -> impl '_ + fmt::Display {
//     let needs_hashes = src.contains('"');
//     let hash_count = if needs_hashes {
//         // FIXME: there's got to be a better way to do this;
//         // this is also a conservative sufficient but not minimal estimate.
//         src.split(|c| c != '#')
//             .filter(|run| !run.is_empty()) // try to hint the optimizer
//             .map(|run| run.len() + 1)
//             .max()
//             .unwrap_or(0)
//     } else {
//         0
//     };

//     display!(
//         r##"r{hashes}"{src}"{hashes}"##,
//         hashes = format_args!("{:#<hash_count$}", "")
//     )
// }
