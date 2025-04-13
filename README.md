# ShellShock Live Trainer
A simple (non intrusive) trainer for [ShellShock Live](http://www.shellshocklive.com)

**The project is only for exercise purposes and should not be used for cheating.**

My main interest is not the game itself. The goal of this project was to improve my knowledge of the Rust programming language and the Windows-API.

# Usage (Windows only)

1. Execute the trainer (installation see below).
2. Start "Shellshock Live" (the trainer automatically detects a running instance of "Shellshock Live").
3. There are four hardcoded keys
    * Key 1 (save current mouse position as position 1)
    * Key 2 (save current mouse position as position 2)
    * Key 3 (calculate different angle/speed combinations to hit the target (position 2)) (now also must input Wind (-100 - 100)
    * Key 4 (clear positions)
    * Key 5 (switch calculation mode) #don't know what this does tbh

Example:
1. Move the mouse over your tank and press '1'.
2. Move the mouse over the enemy tank and press '2'.
3. Press '3'.
4. Input -30
5. Enter

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
