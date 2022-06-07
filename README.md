# QuantumOS
The Operating System built for the modern human!

---

# What is QuantumOS?
Quantum OS is a continuation project of my ever loving joy for operating system development. I started with a project called FluxedOS; but due to the limitations with `c` and `c++`, I found myself doing much more work than what should be necessary to program basic kernel features. This made programming FluxedOS feel like a chore sometimes, and I would spend months working on concepts just because of memory safety issues. This is why I started QuantumOS.


QuantumOS has the idea of being memory safe, and well documented. I want to make an operating system that has an ever dying care for every line of code written. I would like the kernel to have a whole new feel then anything seen before.


I feel like Operating Systems have become very boring, and progression has stopped trying to bring a new architecture to the realm of System Development.


# What makes QuantumOS different?
QuantumOS is one of the few Operating System projects written in the Rust Programming Language. Rust is a programming language that is supposed to solve the aforementioned issues encountered with `c` and `c++`. This makes development time much faster and easier to maintain, plus having the idea of compiled code is working code. Every time you compile rust, it is guaranteed that you have a memory safe and “working” program. This ensures that almost all problems that were causing my kernel to crash in `c++` would be caught by the compiler.

This is not the only reason that QuantumOS is different. QuantumOS is not going to be based on an existing Operating System. This means that how the file system interacts with the kernel, and how the system is laid out as a whole is going to be slightly if not wildly different then Linux or Posix


