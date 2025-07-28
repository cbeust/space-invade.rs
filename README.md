# Space Invade.rs
## An 8080 Space Invaders emulator written in Rust

<p align="center">
  
![space-invaders-1](https://user-images.githubusercontent.com/92322/132575917-6cbf246b-7acd-4126-9e60-2710216d0109.gif)

</p>

This is an 8080 emulator running the 1978 Space Invaders game by Taito, written in Rust, including sound.

`space-invade.rs` is my third
emulator (the first one was [Chip-8](https://github.com/cbeust/chip-8) and then an
[Apple \]\[ emulator](https://github.com/cbeust/sixty), both written in Kotlin). I chose Rust for this project to
experiment with a different design and see how far I could push the performance.

## Running the emulator

`cargo run --release`

`cargo test` will run the `cpudiag` emulator test.

## Playing it

- `c` to insert a coin.
- `1` or `2` to select the number of players.
- Player 1: left and right arrows to move, space to shoot.
- Player 2: `a` and `d` to move, `s` to shoot.
- `p` will pause the game, any key will resume.
- `t` to cause a tilt.
- `ESC` to close the window.

## The 8080 processor

The 8080 turned out to be quite an advanced processor and even though it came out before the 6502 (1974 and 1975
respectively), I grew to appreciate its rich set of instructions (the 6502 only has 56, the 8080 has over 200) and
how elegantly you can write some algorithms.

The emulator in this project is close to complete, including:

- All the instructions
- Correct cycle count
- Implementation of binary coded decimal (`DAA`)
- Interruptions (`EI` and `SI` and some `RST` but not all since Space Invaders only uses a few)

The emulator also passes the `cpudiag` diagnostic application, which you can run with `cargo test`.

## Space Invaders

The arcade game had quite an interesting architecture, please see the [resources section](#resources) if you
need in-depth details. In a nutshell:

- The resolution is 224x256 with one bit of depth, so just black and white. The arcade used tranparent green and
red tapes on top of the screen to simulate additional colors, which this emulator does as well.
- The graphic memory starts at 0x2400 with each byte representing eight bits (note: the screen is flipped 90 degrees).
- The video generates two interrupts: one when the beam is halfway through the screen and one at the end
of the screen (VBL). Each of these interrupts calls a `RST` which tells the CPU to jump at $08 and $10 respectively.
These two addresses are in charge of updating the half of the screen that the beam just finished drawing (quite a clever hack). My emulator
doesn't go to that level of details and simply refreshes the graphic during the VBL.
- While the 8080 knows how to shift bits, it only has instructions to shift by one bit, and these are pretty slow,
which makes shifting variable numbers of bits costly. This is unfortunate since it's the only way that this hardware
can animate its graphics, so the designers created a multiple bit shifter outside in the game hardware itself
and hooked it up with its `input` and `output` instructions. Refer to the detailed architecture (or the code) to
understand how this works, but I thought this was another very clever design around hardware limitations of the time.

The emulator is calibrated to run at 2Mhz with the following logic, which you will find in the [`run_one_frame()`](https://github.com/cbeust/space-invade.rs/blob/main/emulator/src/emulator.rs#L143-L157) function:

- Run as many cycles as necessary to reach the first half of the screen (about 16,500)
- Generate the first interrupt
- Run as many cycles as necessary to reach the end of the screen (33,000)
- Generate the second interrupt

This function is called for each frame but the caller will sleep if needed so that the next frame isn't drawn until
16ms have elapsed (60Hz). You can find this code in [`sdl2.rs`](https://github.com/cbeust/space-invade.rs/blob/main/src/sdl2.rs#L42-L49).
Without this delay, this is what the game at normal speed will look like, running at about 70Mhz instead of 2Mhz:

![space-invaders-2](https://user-images.githubusercontent.com/92322/132596321-788b99db-c765-4ddb-bca3-4288fb8b3e35.gif)

Note that letting the game run like this for a little while is a good way to test that your code doesn't have
any bugs that only show up on the long run...

Despite this, I can't shake the feeling that my Space Invaders is running a bit faster than the original,
based on watching some footage of the actual arcade game, but I'm not 100% sure. Feel free to let me know
if you find a bug in my timing routine that might explain my impression.

## Lessons learned

### `cpudiag` is not enough.

When I wrote my Apple ][ emulator, I first developed the 6502 emulator with a diagnostic file that was extremely
thorough, and which gave me a pretty much 100% guarantee that once all these tests pass, my 6502 is flawless
and I can now focus on the Apple ][ part without worrying about CPU bugs. I started this project with the
same assumption and it was a mistake. Deep into the development, I was displaying graphics that made no sense
and after hours of disassembling and understanding what the 8080 code actually does, I ended up finding
two CPU bugs which `cpudiag` had not found.

And one of them was really trivial: I hadn't implemented the `MOV E,A` instruction correctly. This is a very simple
instruction which moves the content of `A` into `E`, but I made a typo in my code and moved it into
`D` instead... The other bug was in my implementation of `DAD`.

Moral of the story: you might have to add tests of your own, and at any rate, you cannot embark on such a project
without developing a fluent ability to understand the assembly you are emulating.

### Graphics in Rust are still not quite there.

It's becoming a meme now that Rust is struggling in the graphics library department
([exhibit A](https://www.areweguiyet.com/)) and in the game department
([exhibit B](https://arewegameyet.rs/)). Finding a library that would make it easy for me to implement
this project with very simple requirements (basically just need a frame buffer, possibly some simple
GUI widgets) took me way too long. I ended up settling on SDL 2, but the bottom line is that developing
GUI's and graphics in Rust on Windows is not exactly a walk in the park.

### Next steps

I'm probably done with this project but if I were to come back to it to improve it, I would probably:

- Make it run in the browser with WASM
- Show the controls so the user doesn't have to guess
- Add a debugger, breakpoints, etc...

### Resources

The following documentation was invaluable to pull off this fun project:

- [8080 Reference Manual](https://altairclone.com/downloads/manuals/8080%20Programmers%20Manual.pdf)
- [Emulator 101](http://www.emulator101.com/)  (note: not `https`)
- [Space Invaders assembly listing](https://computerarcheology.com/Arcade/SpaceInvaders/Code.html)
- [cpudiag assembly listing](https://github.com/cbeust/space-invade.rs/blob/main/emulator/cpudiag.lst)
 
