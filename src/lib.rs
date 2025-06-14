pub mod assembler;
pub mod inst;
pub mod instruction_parser;
pub mod vm;

#[cfg(test)]
mod tests {
    use super::*;

    fn get_assembly_string<'a>() -> &'a str {
        r#"
    li 
    "#
    }

    #[test]
    fn run_assembly() {
        let asm = get_assembly_string();
        assert_eq!(4, 4);
    }
}
