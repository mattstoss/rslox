use std::fs;

use anyhow::Result;

mod scanner;
mod token;

fn main() -> Result<()> {
    let input = fs::read_to_string("examples/simple.lox")?;

    let tokens = scanner::scan(&input)?;
    for t in tokens {
        println!("{:?}", t);
    }

    Ok(())
}
