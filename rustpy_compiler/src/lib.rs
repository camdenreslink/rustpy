extern crate unicode_xid;

pub mod ast;
pub mod parser;
pub mod tokenizer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
