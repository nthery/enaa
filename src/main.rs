//! VM CLI

use anyhow::Context;
use clap::{Parser, Subcommand};
use std::fs;

use enaa::asm::*;
use enaa::vm::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Dis,
    Decrypt { path: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let bytecode = assemble(DECRYPTER)?;
    match cli.command {
        Commands::Dis => println!("{}", pretty_print(DECRYPTER)?),
        Commands::Decrypt { path } => {
            let cipher = fs::read_to_string(path).context("reading cipher")?;
            println!("{}", run(&bytecode, &cipher)?);
        }
    }
    Ok(())
}
const DECRYPTER: &[Insn] = &[
    Insn::new(Opcode::Push).set_value(4),
    Insn::new(Opcode::Popa),
    Insn::new(Opcode::In).set_label("loop"),
    Insn::new(Opcode::Dup),
    Insn::new(Opcode::Bne).set_target("decode"),
    Insn::new(Opcode::Exit),
    Insn::new(Opcode::Pusha).set_label("decode"),
    Insn::new(Opcode::Add),
    Insn::new(Opcode::Dup),
    Insn::new(Opcode::Push).set_value('z' as u32),
    Insn::new(Opcode::Ble).set_target("out"),
    Insn::new(Opcode::Push).set_value(26),
    Insn::new(Opcode::Sub),
    Insn::new(Opcode::Out).set_label("out"),
    Insn::new(Opcode::Pusha),
    Insn::new(Opcode::Push).set_value(1),
    Insn::new(Opcode::Add),
    Insn::new(Opcode::Dup),
    Insn::new(Opcode::Push).set_value(25),
    Insn::new(Opcode::Bgt).set_target("wrap"),
    Insn::new(Opcode::Popa),
    Insn::new(Opcode::Jmp).set_target("loop"),
    Insn::new(Opcode::Push).set_value(0).set_label("wrap"),
    Insn::new(Opcode::Popa),
    Insn::new(Opcode::Jmp).set_target("loop"),
];
