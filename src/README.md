# Quantum Build
This is the `build script` that makes it easy to get up and running with the Quantum project.

# How do I use?
This `build script` is designed to be as easy to use as possible, below are some basic commands with thier discription.
```bash
# This will build and run the entire project, its what
# is recommended to be used during development. 
&> cargo run 

# This will test all the Libraries in the project. 
# Tests are done in userspace on the devlopment 
# machine's enviroment. This is done for easy
# debugging.
&> cargo run test-libs

# This will only build the project, but will skip
# running qemu
&> cargo run noqemu
```

# Why do all this in a crate?
I used to have a nice little build script in a little `build.sh`, 
but as the project grew, it was very slow and hard to keep up with 
the changes. Once the bootloader was at `stage-2`, I needed a better
option to create a disk image and compile all the different modules. 
So this little script was made to speed up the process. This crate
makes disk images 10x faster then `dd` and `mkfs`, so its really nice
for rebuilding the disk image every build. 
