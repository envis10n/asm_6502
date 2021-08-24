# asm_6502_cli

The library's companion CLI tool for assembly and disassembly of 6502 ASM and raw data.

## Build

In order to build a functional version of this binary, it requires the `cli` feature to be enabled.

This is to make sure the argument parsing crate isn't pulled in when it's being used for only the library.

Build command: `cargo build --bin asm_6502 --features "cli"`

## Usage

Call the binary with `--help` or `-h` to see the available options and arguments.

Example help output:
```
A rusty 6502 assembler/disassembler.

Positional arguments:
  input                 Direct source input.

Optional arguments:
  -h,--help             Show this help message and exit
  -v,--version          Display the version.
  -o,--output OUTPUT    Path to write the output to. If missing, will write to
                        stdout.
  -f,--file FILE        Path to a source file to compile / decompile.
  -O,--offset OFFSET    The memory offset to start the program at.
  -d,--disassemble      Disassemble the input or file, instead of assembling.
  -a,--assemble         Assemble the input or file. (Default)
```