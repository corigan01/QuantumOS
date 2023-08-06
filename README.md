![Black Blue and Neon Pink Modern Tech Electronics and Technology X-Frame Banner (1)](https://user-images.githubusercontent.com/33582457/172414092-1c3fb73c-51e2-43f0-8e68-b1de9848fd1f.png)

---

# What is QuantumOS?
Quantum OS is a continuation project of my ever loving joy for operating system development. I started with a project called FluxedOS; but due to the limitations with `c` and `c++`, I found myself doing much more work than what should be necessary to program basic Kernel features. This made programming FluxedOS feel like a chore sometimes, and I would spend months working on concepts just because of memory safety issues. This is why I started QuantumOS.


QuantumOS has the idea of being memory safe, and well documented. I want to make an operating system that has an ever dying care for every line of code written. I would like the Kernel to have a whole new feel then anything seen before.


I feel like Operating Systems have become very boring, and progression has stopped trying to bring a new architecture to the realm of System Development.


# What makes QuantumOS different?
QuantumOS is one of the few Operating System projects written in the Rust Programming Language. Rust is a programming language that is supposed to solve the aforementioned issues encountered with `c` and `c++`. This makes development time much faster and easier to maintain, plus having the idea of compiled code is working code. Every time you compile rust, it is guaranteed that you have a memory safe and “working” program. This ensures that almost all problems that were causing my Kernel to crash in `c++` would be caught by the compiler.

This is not the only reason that QuantumOS is different. QuantumOS is not going to be based on an existing Operating System. This means that how the file system interacts with the Kernel, and how the system is laid out as a whole is going to be slightly if not wildly different then Linux or Posix


# Building QuantumOS

Since the Kernel is small, its super easy to build and get running. Follow the few basic steps to get running. 

### Packages
```
* Qemu
* Rust (rustup, cargo)
* llvm-tools

(Auto install script coming soon)
```

### Compiling and Running

```bash
# There is a cargo project dedicated to building and running QuantumOS
&> cd Meta/
&/Meta/> cargo run -- help
Meta QuantumOS Compile Script

Usage: meta [OPTIONS] <COMMAND>

Commands:
  build  Build QuantumOS and all of its dependencies
  run    Run QuantumOS
  test   Test QuantumOS
  clean  Delete Build artifacts
  help   Print this message or the help of the given subcommand(s)

Options:
  -b, --bootloader <BOOTLOADER>
          Which bootloader to use (bios / uefi)
          
          [default: bios]

          Possible values:
          - bios: Use bios booting
          - uefi: use uefi booting

      --kvm
          Enable KVM

  -d, --debug-compile
          Debug Compile Mode

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
&/Meta/> cargo run -- run
...

# This will build and run QuantumOS with normal bulid options!
# There should be no extra work to get Quantum Booting :)


```
---
![how to contribute](https://user-images.githubusercontent.com/33582457/172416262-3bb764bd-2aba-4b94-a41f-dd8c0acb4501.png)


* Make a fork of the project
* Commit fairly (use details of what you changed in each commit)
* Submit a pull request for addition to the main branch
* Check changes

## How to fork
* Click the fork button at the top right corner
![](https://docs.github.com/assets/images/help/repository/fork_button.jpg)
* If you need more help, click [here](https://docs.github.com/en/github/getting-started-with-github/fork-a-repo)


## What is a fair commit
After you make a change, make a small explanation of the changes you made. An example is 
```bash
&> git add *
&> git commit -a

Added 'MyNewFile.txt'. This file helps explain the contributing. 

# Please enter the commit message for your changes. Lines starting
# with '#' will be ignored, and an empty message aborts the commit.
#
# On branch main
# Your branch is up to date with 'origin/main'.
#
# Changes to be committed:
#       new file:   MyNewFile.txt
#
# Untracked files:
#       .vscode/
#
# Press ctrl+X then Y then enter to exit and save

&> git push
```

## Make a pull request
Go to the main project source (corigan01/QuantumOS). Then click on pull requests. From here you pull in your repo into the main branch. 

## Check changes
When you make a pull-request github will check to see if the code works. If github fails to build the code, then you will need to fix and commit those changes. 
If github is able to build the file, then we will pull it into the main branch!



