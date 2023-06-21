mod cpu;
mod disassembler;
mod platform;
mod rom;
mod sdl_platform;
mod keyboard;
use cpu::{init_program, Runnable};
use platform::{create_contexts, Platform};
use sdl_platform::SdlPlatform;
use std::env;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        eprintln!("Missing file arg!");
        return Ok(());
    }

    let file_name = "/home/ori/chip8-rust/test/Breakout.ch8";

    let program_res = init_program(&file_name);
    if program_res.is_err() {
        eprint!("Error loading program!");
        return Ok(());
    }

    let mut program = program_res.unwrap();
    let (platform_context, cpu_context) = create_contexts();
    let platform_thread = std::thread::spawn(move || {
        let mut platform = SdlPlatform::default();
        platform.start(&platform_context);
    });

    program.run(&cpu_context);

    match platform_thread.join() {
        Ok(_) => (),
        Err(_) => (),
    }
    return Ok(());
}
