//! Pseudo-assembler and disassembler

use std::collections::HashMap;

use anyhow::Context;

use crate::vm::*;

/// Single assembly instruction with optional label and operand to assemble.
pub struct Insn {
    label: Option<&'static str>,
    opcode: Opcode,
    operand: Operand,
}

/// Instruction operand.
pub enum Operand {
    None,
    Target(&'static str),
    Value(u32),
}

impl Insn {
    pub const fn new(opcode: Opcode) -> Insn {
        Insn {
            label: None,
            opcode,
            operand: Operand::None,
        }
    }

    pub const fn set_label(self, label: &'static str) -> Insn {
        Insn {
            label: Some(label),
            opcode: self.opcode,
            operand: self.operand,
        }
    }

    pub const fn set_value(self, value: u32) -> Insn {
        Insn {
            label: self.label,
            opcode: self.opcode,
            operand: Operand::Value(value),
        }
    }

    pub const fn set_target(self, label: &'static str) -> Insn {
        Insn {
            label: self.label,
            opcode: self.opcode,
            operand: Operand::Target(label),
        }
    }
}

/// Assemble a sequence of instructions into a sequence of bytecodes.
pub fn assemble(source: &[Insn]) -> anyhow::Result<Vec<u8>> {
    let mut labels = HashMap::new();
    let mut relocations = Vec::new();
    let mut bytecodes = Vec::new();
    for insn in source.iter() {
        if let Some(label) = insn.label {
            labels.insert(label, bytecodes.len());
        }
        bytecodes.push(insn.opcode as u8);
        match insn.operand {
            Operand::None => (),
            Operand::Target(label) => {
                relocations.push((label, bytecodes.len()));
                bytecodes.push(0)
            }
            Operand::Value(value) => bytecodes.push(value as u8),
        }
    }

    for (label, offset) in relocations {
        bytecodes[offset] = *labels.get(label).context("look up label")? as u8;
    }

    Ok(bytecodes)
}

pub fn pretty_print(source: &[Insn]) -> anyhow::Result<String> {
    let mut output = String::new();
    for insn in source {
        if let Some(label) = insn.label {
            output.push_str(&format!("{}:\t", label))
        } else {
            output.push('\t')
        };
        output.push_str(&format!("{:?}", insn.opcode));
        match insn.operand {
            Operand::None => (),
            Operand::Target(label) => output.push_str(&format!(" {}", label)),
            Operand::Value(n) => output.push_str(&format!(" {}", n)),
        };
        output.push('\n');
    }
    Ok(output)
}
