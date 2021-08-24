#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod ops;

use error::CompileError;
use std::{collections::HashMap, fmt::Display};

use ops::{AddressingMode, OpCode, OPCODES_MAP, OPCODES_OP_MAP};

pub type Result<T> = std::result::Result<T, CompileError>;

#[derive(Clone)]
pub enum InstructionAddress {
    None,
    Address(u16),
    Label(String),
}

impl Display for InstructionAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.clone() {
            InstructionAddress::None => write!(f, "    "),
            InstructionAddress::Label(addr) => write!(f, "{}", addr),
            InstructionAddress::Address(addr) => write!(f, "{:04X}", addr),
        }
    }
}

#[derive(Clone)]
pub struct Instruction {
    pub mnemonic: String,
    pub mode: AddressingMode,
    pub code: u8,
    pub operands: Vec<u8>,
    pub address: InstructionAddress,
}

impl Instruction {
    pub fn new(
        mnemonic: String,
        mode: AddressingMode,
        code: u8,
        operands: Vec<u8>,
        address: InstructionAddress,
    ) -> Self {
        Instruction {
            mnemonic,
            mode,
            code,
            operands,
            address,
        }
    }
    pub fn from_source_line(
        labels: &HashMap<String, u16>,
        input: &str,
    ) -> std::result::Result<Self, &'static str> {
        let ref opcodes = *OPCODES_MAP;

        let temp1: Vec<&str> = input.split("\t").filter(|s| s.len() > 0).collect();
        let temp2: Vec<&str>;

        let address: InstructionAddress = if temp1.len() > 1 {
            temp2 = temp1[1].split(" ").filter(|s| s.len() > 0).collect();
            // Included label OR address
            let address_temp = temp1[0];
            if let Ok(addr) = u16::from_str_radix(address_temp, 16) {
                // Absolute Address
                InstructionAddress::Address(addr)
            } else {
                // Label
                InstructionAddress::Label(address_temp.to_string())
            }
        } else {
            temp2 = temp1[0].split(" ").collect();
            InstructionAddress::None
        };

        let mnemonic = temp2[0];
        let operand = if temp2.len() == 1 {
            ""
        } else {
            if temp2[1].len() > 0 {
                temp2[1]
            } else {
                ""
            }
        };

        let (operands, mode) = if operand.len() > 0 {
            if &operand[0..1] == "(" {
                // Indirect Addressing
                if &operand[(operand.len() - 1)..] == "Y" {
                    // Indirect Y
                    let val_t = &operand[1..(operand.len() - 3)];
                    let (val, _) = get_bytes_from_asm(labels, val_t)?;
                    (val, AddressingMode::IndirectY)
                } else if &operand[(operand.len() - 2)..] == "X)" {
                    // Indirect X
                    let val_t = &operand[1..(operand.len() - 3)];
                    let (val, _) = get_bytes_from_asm(labels, val_t)?;
                    (val, AddressingMode::IndirectX)
                } else {
                    // Indirect
                    let val_t = &operand[1..(operand.len() - 1)];
                    let (val, _) = get_bytes_from_asm(labels, val_t)?;
                    (val, AddressingMode::Indirect)
                }
            } else {
                let val_t = &operand[0..];
                let last_t = &val_t[(val_t.len() - 1)..];
                match last_t {
                    "X" | "Y" => {
                        // Y
                        let is_x = last_t == "X";
                        let (val, _) = get_bytes_from_asm(labels, &val_t[..(val_t.len() - 2)])?;
                        if val.len() == 2 {
                            // Absolute X/Y
                            (
                                val,
                                if is_x {
                                    AddressingMode::AbsoluteX
                                } else {
                                    AddressingMode::AbsoluteY
                                },
                            )
                        } else {
                            // ZeroPage X/Y
                            (
                                val,
                                if is_x {
                                    AddressingMode::ZeroPageX
                                } else {
                                    AddressingMode::ZeroPageY
                                },
                            )
                        }
                    }
                    _ => {
                        let (val, absolute) = get_bytes_from_asm(labels, val_t)?;
                        if absolute {
                            if val.len() == 2 {
                                // Absolute
                                (val, AddressingMode::Absolute)
                            } else {
                                (val, AddressingMode::ZeroPage)
                            }
                        } else {
                            (val, AddressingMode::Immediate)
                        }
                    }
                }
            }
        } else {
            (vec![], AddressingMode::Implied)
        };
        let code: Option<u8> = if let Some(codes) = opcodes.get(mnemonic) {
            let mut c: Option<u8> = None;
            for opcode in codes {
                if opcode.mode == mode {
                    c = Some(opcode.code);
                    break;
                } else if opcode.mode == AddressingMode::Relative
                    && mode == AddressingMode::ZeroPage
                {
                    c = Some(opcode.code);
                    break;
                }
            }
            c
        } else {
            None
        };
        if let Some(op) = code {
            Ok(Instruction::new(
                mnemonic.to_string(),
                mode,
                op,
                operands,
                address,
            ))
        } else {
            Err("no opcode found")
        }
    }
}

impl From<OpCode> for Instruction {
    /// Create an Instruction from an OpCode with empty operands.
    fn from(opcode: OpCode) -> Self {
        Instruction {
            mnemonic: opcode.mnemonic.to_string(),
            code: opcode.code,
            mode: opcode.mode,
            operands: Vec::with_capacity(opcode.len as usize - 1),
            address: InstructionAddress::None,
        }
    }
}

impl Into<(InstructionAddress, Vec<u8>)> for Instruction {
    fn into(mut self) -> (InstructionAddress, Vec<u8>) {
        let mut result = vec![self.code];
        result.append(&mut self.operands);
        (self.address, result)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = self.into();
        write!(f, "{}\t{}", self.address, s)
    }
}

fn get_bytes_from_asm(
    labels: &HashMap<String, u16>,
    input: &str,
) -> std::result::Result<(Vec<u8>, bool), &'static str> {
    if &input[0..1] == "$" {
        // Address
        let val_t = &input[1..];
        if val_t.len() == 4 {
            // u16
            if let Ok(val) = u16::from_str_radix(val_t, 16) {
                Ok((val.to_le_bytes().to_vec(), true))
            } else {
                Err("error converting value to u16")
            }
        } else {
            if let Ok(val) = u8::from_str_radix(val_t, 16) {
                Ok((vec![val], true))
            } else {
                Err("error converting value to u8")
            }
        }
    } else if &input[0..1] == "#" {
        // Numeric Value
        match &input[1..2] {
            "$" => {
                // Hex
                let val_t = &input[2..];
                if val_t.len() == 2 {
                    // u8
                    if let Ok(val) = u8::from_str_radix(val_t, 16) {
                        Ok((vec![val], false))
                    } else {
                        Err("error converting value to u8")
                    }
                } else {
                    if let Ok(val) = u16::from_str_radix(val_t, 16) {
                        Ok((val.to_le_bytes().to_vec(), false))
                    } else {
                        Err("error converting value to u16")
                    }
                }
            }
            "%" => {
                // Binary
                let val_t = &input[2..];
                if val_t.len() == 8 {
                    // u16
                    if let Ok(val) = u16::from_str_radix(val_t, 2) {
                        Ok((val.to_le_bytes().to_vec(), false))
                    } else {
                        Err("error converting value to u16")
                    }
                } else {
                    // u8
                    if let Ok(val) = u8::from_str_radix(val_t, 2) {
                        Ok((vec![val], false))
                    } else {
                        Err("error converting value to u8")
                    }
                }
            }
            _ => {
                // Decimal
                let val_t = &input[1..];
                if let Ok(byte) = u8::from_str_radix(val_t, 10) {
                    Ok((vec![byte], false))
                } else {
                    if let Ok(val) = u16::from_str_radix(val_t, 10) {
                        Ok((val.to_le_bytes().to_vec(), false))
                    } else {
                        Err("error converting value to u16")
                    }
                }
            }
        }
    } else {
        // Label?
        if let Some(label) = labels.get(&input.to_string()) {
            Ok((label.to_le_bytes().to_vec(), true))
        } else {
            if cfg!(debug_assertions) {
                println!("DEBUG ON ERROR PREFIX: {}", input);
            }
            Err("invalid value prefix")
        }
    }
}

impl Into<String> for Instruction {
    fn into(self) -> String {
        match self.operands.len() {
            2 => {
                // u16
                let value = u16::from_le_bytes([self.operands[0], self.operands[1]]);
                format!(
                    "{} {}",
                    self.mnemonic,
                    match self.mode {
                        AddressingMode::Absolute => format!("${:04X}", value),
                        AddressingMode::AbsoluteX => format!("${:04X},X", value),
                        AddressingMode::AbsoluteY => format!("${:04X},Y", value),
                        AddressingMode::Indirect => format!("(${:04X})", value),
                        _ => panic!("mismatched addressing mode and operand length"),
                    }
                )
            }
            1 => {
                let value = self.operands[0];
                format!(
                    "{} {}",
                    self.mnemonic,
                    match self.mode {
                        AddressingMode::Immediate => format!("#${:02X}", value),
                        AddressingMode::ZeroPage => format!("${:02X}", value),
                        AddressingMode::ZeroPageX => format!("${:02X},X", value),
                        AddressingMode::ZeroPageY => format!("${:02X},Y", value),
                        AddressingMode::IndirectX => format!("(${:02X},X)", value),
                        AddressingMode::IndirectY => format!("(${:02X}),Y", value),
                        _ => panic!("mismatched addressing mode and operand length"),
                    }
                )
            }
            0 => format!("{}", self.mnemonic),
            _ => panic!("invalid operand count"),
        }
    }
}

impl Into<String> for &Instruction {
    fn into(self) -> String {
        match self.operands.len() {
            2 => {
                // u16
                let value = u16::from_le_bytes([self.operands[0], self.operands[1]]);
                format!(
                    "{} {}",
                    self.mnemonic,
                    match self.mode {
                        AddressingMode::Absolute => format!("${:04X}", value),
                        AddressingMode::AbsoluteX => format!("${:04X},X", value),
                        AddressingMode::AbsoluteY => format!("${:04X},Y", value),
                        AddressingMode::Indirect => format!("(${:04X})", value),
                        _ => panic!(
                            "0x{:02X} mismatched addressing mode and operand length 2: {:?} - {:?}",
                            self.code,
                            self.mode,
                            self.operands.clone()
                        ),
                    }
                )
            }
            1 => {
                let value = self.operands[0];
                format!(
                    "{} {}",
                    self.mnemonic,
                    match self.mode {
                        AddressingMode::Immediate => format!("#${:02X}", value),
                        AddressingMode::ZeroPage => format!("${:02X}", value),
                        AddressingMode::ZeroPageX => format!("${:02X},X", value),
                        AddressingMode::ZeroPageY => format!("${:02X},Y", value),
                        AddressingMode::IndirectX => format!("(${:02X},X)", value),
                        AddressingMode::IndirectY => format!("(${:02X}),Y", value),
                        AddressingMode::Relative => format!("${:02X}", value),
                        _ => panic!(
                            "mismatched addressing mode and operand length 1: {:?} - {:?}",
                            self.mode,
                            self.operands.clone()
                        ),
                    }
                )
            }
            0 => format!("{}", self.mnemonic),
            _ => panic!("invalid operand count"),
        }
    }
}

pub struct Asm6502 {
    pub input: String,
    pub instructions: Vec<Instruction>,
    memory_start: u16,
}

impl Asm6502 {
    pub fn new(data: String, memory_start: u16) -> Self {
        Asm6502 {
            input: data.replace("\r\n", "\n").trim().to_string(),
            instructions: vec![],
            memory_start,
        }
    }
    pub fn decompile(input: Vec<u8>, memory_start: u16) -> Vec<String> {
        let ref opcodes = *OPCODES_OP_MAP;
        let mut result = vec![];
        let mut i: usize = 0;
        loop {
            if i >= input.len() {
                break;
            }
            let b = input[i];
            if let Some(opcode) = opcodes.get(&b) {
                let code = opcode.code;
                let mnemonic = opcode.mnemonic.to_string();
                let address = InstructionAddress::Address(memory_start + i as u16);
                let operands: Vec<u8> = input[i + 1..opcode.len as usize + i].to_vec();
                let instruction =
                    Instruction::new(mnemonic, opcode.mode.clone(), code, operands, address);
                result.push(format!("{}", instruction.to_string()));
                if i + opcode.len as usize > input.len() - 1 {
                    break;
                }
                i += opcode.len as usize;
            } else {
                i += 1;
            }
        }
        result
    }
    pub fn compile(&mut self) -> Result<Vec<Instruction>> {
        let mut result = vec![];
        let mut labels: HashMap<String, u16> = HashMap::new();
        let mut line_number: usize = 1;
        let mut first_addr: u16 = self.memory_start;
        let mut current_addr: u16 = first_addr;
        for line in self.input.split("\n") {
            match Instruction::from_source_line(&labels, line) {
                Ok(mut instruction) => {
                    match instruction.address.clone() {
                        InstructionAddress::Label(label) => {
                            labels.insert(label.clone(), current_addr);
                            instruction.address = InstructionAddress::Address(current_addr);
                        }
                        InstructionAddress::None => {
                            instruction.address = InstructionAddress::Address(current_addr);
                        }
                        InstructionAddress::Address(adr) => {
                            if line_number == 1 {
                                first_addr = adr;
                            }
                        }
                    }
                    current_addr += instruction.operands.len() as u16 + 1;
                    result.push(instruction);
                }
                Err(err) => return Err(CompileError::new(line_number, err)),
            }
            line_number += 1;
        }
        self.instructions = result.clone();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn general_parse() {
        let mut asm = Asm6502::new(
            "LDA #%0101\nSTA ($15,X)\nEOR ($2A),Y\nTAX\nJMP (label_a)\nADC $C001,X\nINC $F001,X\nLDA $01,X\nLDA ($01),Y\nBPL $2D\nLDY $02\nLDX label_a".to_string(),
        0x8000);
        match asm.compile() {
            Ok(instructions) => {
                let mut bytes: Vec<u8> = vec![];
                for instruction in instructions {
                    let (_, mut buffer) = instruction.clone().into();
                    bytes.append(&mut buffer);
                    println!("{}", instruction);
                }
                let decomp = Asm6502::decompile(bytes.clone(), 0x8000);
                println!("Bytecode: {:?}\n", bytes);
                println!("Decompiled:\n");
                for line in decomp {
                    println!("{}", line);
                }
            }
            Err(err) => panic!("{}", err),
        }
    }
}
