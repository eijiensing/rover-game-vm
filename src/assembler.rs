pub struct Assembler {}

enum AssemblyPhase {
    FirstPass,
    SecondPass,
}

impl Assembler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn assemble(text: &str) {}
}
