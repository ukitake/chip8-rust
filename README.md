# chip8-rust

This is a CHIP8 emulator written in Rust (first time using it).  It was developed in VS Code on Windows 11 using WSL.  

## Types

#### Rom
The bytecode for a chip8 rom.  This is just an array of unsigned bytes.

#### Cpu
This is the actual chip8 cpu emulator.  It mirrors the hardware of the actual chip8:
- 4096 bytes of RAM
- 16 general purpose 8-bit registers
- 1 16-bit memory register for referring to addresses in RAM
- 1 8-bit delay timer
- 1 8-bit sound timer

I keep the current keyboard state and screen state outside of the actual emulated hardware.

#### Program
A composition of a `Cpu`, `Rom`, program counter, and stack pointer.  Also has a frequency parameter that controls how many 
opcodes are executed per second. 

#### Platform
The Chip8 has a 16 key keyboard and a 64x32 pixel screen.  The Platform is the implementation 
of these 2 pieces of hardware.  
It could theoretically be implemented using any number of windowing libraries, but I supplied 1 implementation using SDL 2.  

#### PlatformContext & CpuContext
The Emulator and the `Platform` are run concurrently in 2 threads.  The Emulator runs in the main 
thread of the program while the `Platform` is responsible for spawning a separate thread for managing 
keyboard input and display rendering.  The Emulator and `Platform` communicate via a set of 4 `Channel`s.
- Keyboard Channel
- Display Channel
- Sound Channel
- Single-Key Channel

The `PlatformContext` needs:
- Keyboard Sender
- Display Receiver
- Sound Receiver
- Single-Key Sender

While the `CpuContext` needs:
- Keyboard Receiver
- Display Sender
- Sound Sender
- Single-Key Receiver

`Platform` implementations should use the non-blocking `try_send` and `try_recv`.

