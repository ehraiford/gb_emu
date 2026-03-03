A GameBoy emulator written in Rust. 


# Project Goals

This was primarily written in January and the 1st 1/2 of February 2026 during some down time while my job was waiting for a contract to come through. This project is still in development but the time I can dedicate to it is now much less.

## Reasons for development

There were a number of reasons for the creation of this project. 

    1. I've been pretty interested in constant evaluation recently and I wanted to explore how it could benefit statically defined systems like emulators.

    2. While I've written a fair number of emulators in Rust at this point, all of them since college were written for work so I don't own their code and can't show them off.

    3. The emulators I have worked on focus on data-level accuracy over wall-clock accuracy. I wanted to work on a system that I could reasonably achieve both.

    4. The emulators I generally work on are embedded devices. Syncing graphics, input, and sound across threads was a challenge I wanted to try my hand at.

    5. On-the-job often isn't the best place to try out potentially bad ideas.

## Development Priorities

As such, this was the ordering of my priorities with the project. 

    1. Accuracy: There are a number of wonky things the GameBoy does to eke out a bit more performance (particularly the PPU since it's got an integrated LCD). The eccentricities of the hardware should be captured in the code. I've tried to abstract the design when appropriate but, if an abstraction could potentially impact the accuracy of the system, I've tried to avoid them except as stopgaps to more permanent solutions.
    
    2. Readability, Extensability, and Adaptability: I find writing code that affords its intention while still being easy to rework for new problems or as your understanding of your requirements evolves is one of the most interesting parts of writing software.This generally amounted to recursive composition of increasingly focused scope for modules, structs, and methods, verbose but explicit naming, and a reliance on enums.

    3. Performance: Admittedly, maintaining wall-clock accuracy of a 1989 handlheld in a modern, performant language isn't the toughest challenge, I still took care to not waste performance. This meant avoiding unnecessary heap allocations, shifting static calculations to compile-time, taking advantage of Rust's zero-cost abstractions, and taking advantage of flamegraphs and simple wall-clock comparisons between commits to track performance changes over time.
    
    4. Experimentation: When it wouldn't impact any of the above priorities, I used this model as an opportunity to try out different approaches to common emulation pain points such as tightly coupled devices, execution synchronization, memory interfaces, and state management. 


# State of the Project

While the project is not *complete*, most major features are at a point that they are usable. Additionally, the model passes a partial collection of the Blargg Test ROM Suite. 

## The Model
Below is a rundown of the state of various core components of the Device:

### CPU

All instructions are implemented. 
With the most recent rewrite, the CPU ticks individual M-cycles instead of whole instructions. This facilitates timing quirks at the sub-instruction level for accesses on the bus.

### PPU

All PPU stages are implemented. 
The different PPU stages take the proper amount of time to execute (with some exceptions on the Drawing Pixels stage), disable CPU access to memory regions when appropriate, and raise interrupts when appropriate.
The OAM Fetcher takes the appropriate amount of time to fetch objects.
There are discrete Background and Object Fetcher queues. These should be cycle accurate.
The big challenge for a cycle-accurate PPU is that the Drawing Pixels stage takes a variable number of P-Cycles. While, I've modeled some of those in my current implementation of the PPU, I have not made the full pass on it to implement all of the quirks that can delay the mode from ending. 

### Memory-Mapped Devices

The only memory mapped device that is not implemented should be Sound. 
This means the list of functional devices are:
    * LCD registers,
    * Joypad Input
    * Serial Transfer
    * Interrupts
    * OAM DMA
    * Timer and Divider
Serial cannot currently accept input and shouts its output into the void (This will likely always be the case. I don't plan to implement communication between multiple instances). However, I'm using it for automated testing of Blargg's test suite.

### Bus and Memory

ROM and all of the various versions of RAM are implemented. 
Memory regions can have their accessibility on the Bus toggled when appropriate. 
Echo RAM masks requests to route them back to Work RAM


## The Interface

Beyond the model itself, there are a few extra layers to let the device run. 

### The Emulator

A minimalist wrapper around the model. It's responsible for managing any of the usability features of the model that way the model can stay a "purer" representation of the hardware.

### The Runtime User Interface

The interface that you use while playing a game. When the model is built with the `headless` feature flag, these are left out. These systems run in a dedicated thread and use thread primitives to communicate directly with the required emulated component.

#### The Game Window

A window acting as the system's LCD display. It uses a triple buffer to avoid thread contentions and a dirty flag to skip draws when there's nothing new to display. 

#### Tile Viewer

A window displaying VRAM's two tilemaps. 

#### Button Input

Input is polled on this thread, too. The mapping is WASD for the D-Pad, K for A, J for B, Enter for Select, and SpaceBar for Start. The input is packed into an `AtomicU8` and sent directly to the Joypad device where it is unpacked in its tick cycle.

### The Commandline Interface

This is primarily to allow the user to supply the path to their ROMs, however there are a few options you can enable here. An early measure I used during the process was to use my system as a disassembler and I've left that as an option here. Other features are enabling the tile-viewer, running for a number of cycles, and executing as fast as possible. 


# Running the project

To compile the project, you need Cargo. This project has primarily been written on Rust v1.92.0 but I don't believe I used any specific cutting edge features so it should work on at least a couple releases older than that. 

The fastest way to run everything is `cargo run --release -- run --tile-map-viewer {path_to_rom}`. However, if you want to separately build and then run the project, you can do as follows:

From the top-level directory, run `cargo build --release`. That will create the executable. You can omit the `--release` arg if you want a debug build instead.
Also optionally, you can add `--features headless` to the end to build the headless version that omits the GUI and controller input systems.

Once you've got the executable, you need to supply it with the run mode and the rom path. Since it's built using Clap for the command line, just running `{your_executable} --help` will provide a breakdown of how to structure your arguments. 
However, the fully featured way to run is as follows: 
    `{your_executable} run --tile-map-viewer {path_to_rom}`
