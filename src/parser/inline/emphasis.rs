use crate::ast::Inline;
use crate::parser::MarkdownParserState;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::anychar,
    combinator::{map, map_opt, not, peek, recognize, value, verify},
    multi::many1,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use std::rc::Rc;

pub(crate) fn emphasis(
    state: Rc<MarkdownParserState>,
) -> impl FnMut(&str) -> IResult<&str, Inline> {
    move |input: &str| {
        alt((
            map(
                alt((
                    delimited(
                        open_tag("***"),
                        emphasis_content(state.clone(), close_tag("***")),
                        close_tag("***"),
                    ),
                    delimited(
                        open_tag("___"),
                        emphasis_content(state.clone(), close_tag("___")),
                        close_tag("___"),
                    ),
                )),
                |inner| Inline::Strong(vec![Inline::Emphasis(inner)]),
            ),
            map(
                alt((
                    delimited(
                        open_tag("**"),
                        emphasis_content(state.clone(), close_tag("**")),
                        close_tag("**"),
                    ),
                    delimited(
                        open_tag("__"),
                        emphasis_content(state.clone(), close_tag("__")),
                        close_tag("__"),
                    ),
                )),
                Inline::Strong,
            ),
            map(
                alt((
                    delimited(
                        open_tag("*"),
                        emphasis_content(state.clone(), close_tag("*")),
                        close_tag("*"),
                    ),
                    delimited(
                        open_tag("_"),
                        emphasis_content(state.clone(), close_tag("_")),
                        close_tag("_"),
                    ),
                )),
                Inline::Emphasis,
            ),
        ))
        .parse(input)
    }
}

fn emphasis_content<'a, P>(
    state: Rc<MarkdownParserState>,
    mut close_tag: P,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<Inline>>
where
    P: Parser<&'a str, Output = (), Error = nom::error::Error<&'a str>>,
{
    move |input: &str| {
        let not_end = |i: &'a str| close_tag.parse(i);
        map_opt(
            recognize(many1(preceded(
                peek(not(not_end)),
                alt((value((), tag("\\*")), value((), anychar))),
            ))),
            |content: &str| {
                crate::parser::inline::inline_many1(state.clone())
                    .parse(content)
                    .map(|(_, content)| content)
                    .ok()
            },
        )
        .parse(input)
    }
}

fn open_tag(tag_value: &'static str) -> impl FnMut(&str) -> IResult<&str, ()> {
    move |input: &str| {
        value(
            (),
            verify(tag(tag_value), |v: &str| {
                can_open(v.chars().next().unwrap(), input.chars().nth(v.len()))
            }),
        )
        .parse(input)
    }
}

fn can_open(marker: char, next: Option<char>) -> bool {
    let left_flanking = next.is_some_and(|c| !c.is_whitespace())
        && (next.is_some_and(|c| !is_punctuation(c)) || (next.is_some_and(is_punctuation)));
    if !left_flanking {
        return false;
    }
    if marker == '_' {
        let right_flanking = next.is_none_or(|c| c.is_whitespace() || is_punctuation(c));
        return !right_flanking;
    }
    true
}

fn close_tag(tag_value: &'static str) -> impl FnMut(&str) -> IResult<&str, ()> {
    move |input: &str| {
        value(
            (),
            verify(tag(tag_value), |v: &str| {
                can_close(v.chars().next().unwrap(), input.chars().nth(v.len()))
            }),
        )
        .parse(input)
    }
}

fn can_close(marker: char, next: Option<char>) -> bool {
    if marker == '_' {
        let right_flanking = next.is_none_or(|c| c.is_whitespace() || is_punctuation(c));
        if !right_flanking {
            return false;
        }
        let left_flanking = next.is_some_and(|c| !c.is_whitespace())
            && (next.is_some_and(|c| !is_punctuation(c)))
            || (next.is_some_and(is_punctuation));
        return !left_flanking || next.is_some_and(is_punctuation);
    }
    true
}

fn is_punctuation(c: char) -> bool {
    use unicode_categories::UnicodeCategories;
    c.is_ascii_punctuation() || c.is_punctuation()
}
