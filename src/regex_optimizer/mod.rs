use std::string::String;
use regex_syntax::Parser;
use regex_syntax::hir::{self, Hir, HirKind};
use regex_syntax::hir::literal::Literals;

pub fn optimize(regex: &str) {
    let hir = Parser::new().parse(regex).unwrap();
    println!("{:?}", Literals::prefixes(&hir));

    match hir.kind() {
        HirKind::Concat(field) => field.into_iter().for_each(|h| println!("{}", h) ),
        _ => (),
    };
}