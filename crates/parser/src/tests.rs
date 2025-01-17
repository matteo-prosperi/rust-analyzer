mod sourcegen_inline_tests;

use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use expect_test::expect_file;

use crate::LexedStr;

#[test]
fn lex_ok() {
    for case in TestCase::list("lexer/ok") {
        let actual = lex(&case.text);
        expect_file![case.txt].assert_eq(&actual)
    }
}

#[test]
fn lex_err() {
    for case in TestCase::list("lexer/err") {
        let actual = lex(&case.text);
        expect_file![case.txt].assert_eq(&actual)
    }
}

fn lex(text: &str) -> String {
    let lexed = LexedStr::new(text);

    let mut res = String::new();
    for i in 0..lexed.len() {
        let kind = lexed.kind(i);
        let text = lexed.text(i);
        let error = lexed.error(i);

        let error = error.map(|err| format!(" error: {}", err)).unwrap_or_default();
        writeln!(res, "{:?} {:?}{}", kind, text, error).unwrap();
    }
    res
}

#[test]
fn parse_ok() {
    for case in TestCase::list("parser/ok") {
        let (actual, errors) = parse(&case.text);
        assert!(!errors, "errors in an OK file {}:\n{}", case.rs.display(), actual);
        expect_file![case.txt].assert_eq(&actual);
    }
}

#[test]
fn parse_inline_ok() {
    for case in TestCase::list("parser/inline/ok") {
        let (actual, errors) = parse(&case.text);
        assert!(!errors, "errors in an OK file {}:\n{}", case.rs.display(), actual);
        expect_file![case.txt].assert_eq(&actual);
    }
}

#[test]
fn parse_err() {
    for case in TestCase::list("parser/err") {
        let (actual, errors) = parse(&case.text);
        assert!(errors, "no errors in an ERR file {}:\n{}", case.rs.display(), actual);
        expect_file![case.txt].assert_eq(&actual)
    }
}

#[test]
fn parse_inline_err() {
    for case in TestCase::list("parser/inline/err") {
        let (actual, errors) = parse(&case.text);
        assert!(errors, "no errors in an ERR file {}:\n{}", case.rs.display(), actual);
        expect_file![case.txt].assert_eq(&actual)
    }
}

fn parse(text: &str) -> (String, bool) {
    let lexed = LexedStr::new(text);
    let input = lexed.to_input();
    let output = crate::TopEntryPoint::SourceFile.parse(&input);

    let mut buf = String::new();
    let mut errors = Vec::new();
    let mut indent = String::new();
    lexed.intersperse_trivia(&output, &mut |step| match step {
        crate::StrStep::Token { kind, text } => {
            write!(buf, "{}", indent).unwrap();
            write!(buf, "{:?} {:?}\n", kind, text).unwrap();
        }
        crate::StrStep::Enter { kind } => {
            write!(buf, "{}", indent).unwrap();
            write!(buf, "{:?}\n", kind).unwrap();
            indent.push_str("  ");
        }
        crate::StrStep::Exit => {
            indent.pop();
            indent.pop();
        }
        crate::StrStep::Error { msg, pos } => errors.push(format!("error {}: {}\n", pos, msg)),
    });

    for (token, msg) in lexed.errors() {
        let pos = lexed.text_start(token);
        errors.push(format!("error {}: {}\n", pos, msg));
    }

    let has_errors = !errors.is_empty();
    for e in errors {
        buf.push_str(&e);
    }
    (buf, has_errors)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct TestCase {
    rs: PathBuf,
    txt: PathBuf,
    text: String,
}

impl TestCase {
    fn list(path: &'static str) -> Vec<TestCase> {
        let crate_root_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let test_data_dir = crate_root_dir.join("test_data");
        let dir = test_data_dir.join(path);

        let mut res = Vec::new();
        let read_dir = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("can't `read_dir` {}: {}", dir.display(), err));
        for file in read_dir {
            let file = file.unwrap();
            let path = file.path();
            if path.extension().unwrap_or_default() == "rs" {
                let rs = path;
                let txt = rs.with_extension("txt");
                let text = fs::read_to_string(&rs).unwrap();
                res.push(TestCase { rs, txt, text });
            }
        }
        res.sort();
        res
    }
}
