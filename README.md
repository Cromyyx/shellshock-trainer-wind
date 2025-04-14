# ShellShock Live Trainer
A simple (non intrusive) trainer for [ShellShock Live](http://www.shellshocklive.com)

**The project is only for exercise purposes and should not be used for cheating.**

My main interest is not the game itself. The goal of this project was to improve my knowledge of the Rust programming language and the Windows-API.

# Usage (Windows only)

1. Execute the trainer (installation see below).
2. Start "Shellshock Live" (the trainer automatically detects a running instance of "Shellshock Live").
3. There are four hardcoded keys
    * Key 1 (Set Source Position (Your Tank))
    * Key 2 (Set Target Position (Enemy Tank))
    * Key 3 *Optional*: (Set Wind Strength (via console input))
    * Key 4 (Calculate Hits (using stored wind & dimensions))
    * Key 5 (Clear Positions and Wind)
    * Key 6 (Switch Mode (Angle/Velocity))
    * Key 7 (Cache Game Window Dimensions (Press while game is active)

Example:
1. Press '7' to cach window size (only have to do once per session or if game window size is changed).
2. Move the mouse over your tank and press '1'.
3. Move the mouse over the enemy tank and press '2'.
4. *Optional* Press '3' and input current wind and press 'Enter'.
5. Press '4' to start caclulation.

# Installation (Windows only)

## Install Rust (must support 2021 edition)
https://www.rust-lang.org/tools/install

## Download sources

Clone this repository with [Git for Windows](https://git-scm.com)
```
git clone [REPO]
```
    
Alternatively download this repository
```
Button "Clone or download" -> "Download ZIP"
```

## Build
1. Open a command line window and change to the directory (cloned or downloaded).
2. Build (output folder "target\release\shellshock-trainer.exe")
```
cargo build --release
```

3. Run
```
cargo run --release
```

or

```
cd target\release
shellshock-trainer.exe
```

# License
MIT
