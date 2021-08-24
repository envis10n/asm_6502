#[cfg(feature = "cli")]
use argparse::{ArgumentParser, Collect, Print, Store, StoreFalse, StoreOption, StoreTrue};

use asm_6502::{Asm6502, Result};

fn assemble(input: String, offset: u16) -> Result<Vec<u8>> {
    let mut result = vec![];
    let mut asm = Asm6502::new(input, offset);
    asm.compile()?;
    for instruction in asm.instructions {
        let (_, mut bytes) = instruction.clone().into();
        result.append(&mut bytes);
    }
    Ok(result)
}

fn main() {
    let mut filepath: Option<String> = None;
    let mut output_filepath: Option<String> = None;
    let mut decompile = false;
    let mut memory_offset: String = "8000".to_string();
    let mut input: Vec<String> = vec![];
    #[cfg(feature = "cli")]
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("A rusty 6502 assembler/disassembler.");
        ap.add_option(
            &["-v", "--version"],
            Print(format!(
                "ASM 6502\n{}",
                env!("CARGO_PKG_VERSION").to_string()
            )),
            "Display the version.",
        );
        ap.refer(&mut output_filepath).add_option(
            &["-o", "--output"],
            StoreOption,
            "Path to write the output to. If missing, will write to stdout.",
        );
        ap.refer(&mut filepath).add_option(
            &["-f", "--file"],
            StoreOption,
            "Path to a source file to compile / decompile.",
        );
        ap.refer(&mut memory_offset).add_option(
            &["-O", "--offset"],
            Store,
            "The memory offset to start the program at.",
        );
        ap.refer(&mut decompile)
            .add_option(
                &["-d", "--disassemble"],
                StoreTrue,
                "Disassemble the input or file, instead of assembling.",
            )
            .add_option(
                &["-a", "--assemble"],
                StoreFalse,
                "Assemble the input or file. (Default)",
            );
        ap.refer(&mut input)
            .add_argument("input", Collect, "Direct source input.");
        ap.parse_args_or_exit();
    }
    let offset = u16::from_str_radix(&memory_offset, 16).unwrap();
    if filepath.is_some() {
        // Ignore input and load file.
        input.clear();
        if decompile {
            let filedata = std::fs::read(filepath.unwrap()).unwrap();
            let result = Asm6502::decompile(filedata, offset).join("\n");
            if let Some(out_file) = output_filepath {
                std::fs::write(out_file, result).unwrap();
            } else {
                println!("{}", result)
            }
        } else {
            let filedata = std::fs::read_to_string(filepath.unwrap()).unwrap();
            match assemble(filedata.clone(), offset) {
                Ok(output) => {
                    if let Some(out_file) = output_filepath {
                        std::fs::write(out_file, output).unwrap();
                    } else {
                        println!(
                            "{}",
                            &output[..]
                                .iter()
                                .map(|v| format!("{:02X}", v))
                                .collect::<Vec<String>>()
                                .join("\n")
                        );
                    }
                }
                Err(err) => {
                    if cfg!(debug_assertions) {
                        println!("FILEDATA:\n{}", filedata);
                    }
                    panic!("{}", err);
                }
            }
        }
    } else if input.len() > 0 {
        // Input is available to compile.
        todo!("Handle raw input here");
    } else {
        std::process::exit(0);
    }
}
