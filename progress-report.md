# Quantum OS Progress Report - Dec 2024

This report details the recent progress made on the Quantum OS project. This month has been highly productive, despite a necessary rewrite that temporarily set back the boot process.

## Project Overview

Quantum OS is a microkernel-based operating system designed with a strong emphasis on security and fine-grained permission management. The core principle revolves around a "pledge" system, where processes declare all required permissions upfront (and before they can do anything else). This prevents privilege escalation and enhances overall system security. 

The architecture focuses on minimizing kernel responsibilities, delegating most functionalities to user-space services accessed via IPC. The kernel handles core functions like memory allocation, scheduling, and IPC, while services manage tasks like file system access, networking, and configuration management.

A key service is the Authentication Daemon, responsible for authorizing and managing process permissions. This daemon implements a sophisticated permission handling system, including the ability to emulate denied permissions by providing isolated resources to processes. For instance, if a process requests access to the user's "Documents" folder but is denied, the Authentication Daemon can create a separate, isolated "Documents" folder for that process, preventing access to the user's actual data.

The intention is that many services can be hooked together to decrease complexity on the programmer, and increase overall system integration. I intend to use Rust's type system and more proc macros to ensure the programmer **must** follow the IPC system exactly as the other process describes to work. 

## Recent Progress

### Bootloader Rewrite

A recent rewrite of the bootloader was undertaken. While this temporarily regressed the boot process, it provided an opportunity to improve the underlying structure and prepare for future development.

### FAT Filesystem Driver Improvements

Prior to this semester, significant progress was made on the FAT filesystem driver. A crucial bug was fixed that prevented correct loading of files larger than one block (1024 bytes). The driver now correctly reads and assembles multi-block files. 

Previously, the driver would randomly shuffle blocks around towards the end of the executable. 

### Procedural Macros (Proc Macros)

Two custom procedural macros have significantly accelerated development.

#### What are Proc Macros?

In Rust, procedural macros (proc macros) are functions that operate on Rust code at compile time. They receive a stream of tokens representing the code and output a new stream of tokens, effectively transforming the code. This allows for powerful code generation and metaprogramming. There are three types of procedural macros:

*   **Function-like macros:** These are invoked like regular functions, e.g., `my_macro!(...)`.
*   **Derive macros:** These are used with the `#[derive]` attribute to automatically implement traits for structs, enums, etc., e.g., `#[derive(Debug)]`.
*   **Attribute macros:** These are used as attributes on items, e.g., `#[my_attribute]`.

The macros described below are attribute macros.

#### 1. Debugging Macro

This proc macro simplifies the process of sending debug information to the serial port (and potentially other destinations like the virtual console). It drastically reduces boilerplate code and allows for easy routing of debug output.

#### 2. Hardware/Register Access Macro (`#[make_hw]`)

This macro generates getter and setter functions for hardware registers, significantly simplifying hardware interaction. It allows me to define bit fields and bit ranges within a register, and the macro automatically generates the necessary bit shifting and masking logic, often performing these operations at compile time for optimal performance (really nice for paging and other control registers).

**Example Usage:**

```rust
#[make_hw(
    // Generates two functions: set_enable_flag(bool) and get_enable_flag() -> bool
    field(RW, 10, enable),
    // Generates one function: get_started_flag() -> bool
    field(RO, 2, started)
)]
struct ExampleStruct(u32);

fn main() {
    let mut example = ExampleStruct(0);
    example.set_enable_flag(true);
    println!("Enable flag: {}", example.get_enable_flag());
    println!("Started flag: {}", example.get_started_flag());
}
```

## Future Work
 - Continued development of the microkernel architecture.
 - Implementation of core services (kernel services), including memory management and scheduling.
 - Begin development of the Authentication Daemon.
 - UEFI Booting support.
 - Boot2 Booting support.
 - Improved ELF parser.

## Security Comparisons and Further Considerations

Quantum OS's security model, centered on least privilege, the "pledge" system, and the Authentication Daemon, offers a different environment to common operating systems. 

FreeBSD uses robust Mandatory Access Control (MAC) frameworks like MAC Framework and TrustedBSD for fine-grained resource control. These are powerful but complex to configure and to use for both the User and the developer. Quantum OS's "pledge" system simplifies security management by always requiring upfront process permission declarations. The Authentication Daemon further improves this by emulating the denied permissions to such process, balancing security and usability for the user and application. FreeBSD's MAC is often system-wide, while Quantum OS's pledge system is application-by-application specific.

This approach addresses a key frustration with existing permission models, such as those found in Android, where denying permissions often causes applications to be unusable. Quantum OS ensures that users retain control over permissions without sacrificing the application's functionality.

Furthermore, Quantum OS draws inspiration from the containerization concepts used in modern operating systems. By incorporating similar ideas into the core IPC model, Quantum OS provides inherent security and isolation, eliminating the need for external tools or complex configurations. This approach enhances system stability and security by preventing unintended interactions between applications.

Unlike FreeBSD and other OSes, Quantum OS intends to not allow the programmer to Opt-Out of these features. For example, in OpenBSD and commonly in other OSes too, the user is only encouraged to take advantage of these features and not required. 

### IPC Prototype


#### Example Server
```rust
/// Hello World Service
/// (This is a doc comment, and can be seen by the macro to provide docs)
#[qos::server(
  // The socket name `HelloWorld://`
  service_name = "HelloWorld",
  // Wake this process up immediately when called
  service_kind = qos::SyncServiceByCall,
  // Does this service apply to other users
  sys_bind = false
)]
mod hello_world_service {
  // Endpoints define a URI interaction point:
  // So, this would be `HelloWorld://TalkToMe{name="{name}"}` <- CLIENT
  #[endpoint(
    // What to repond to the program
    reponse = HelloReponse
  )]
  pub struct TalkToMe {
    name: String
  }

  // Reponse Macro
  // `HelloWorld://HelloReponse{reponse_string="{reponse}"}` -> CLIENT
  #[reponse]
  pub struct HelloReponse {
    reponse_string: String
  }
}

impl qos::IpcReponse for HelloWorldService::TalkToMe {
  fn respond(self) -> Result<HelloReponse, ServiceError> {

    Ok(HelloReponse {
      reponse_string: format!("Hello {}! Welcome to QuantumOS!!", self.name)
    })
  }
}

// Our pledge
#[qos::permissions(
  serve("HelloWorld")
)]
fn main() {
  let hello_world = HelloWorld::bind()
    .expect("Unable to bind to 'HelloWorld' endpoint!");

  // This will spawn a new thread to provide the IP endpoint
  hello_world.make_available();
}
```

#### Example Client
```rust

// Once all the verification is done, the Kernel puts these requests
// into custom pages where the memory is shared and thus does not need
// to be serde.
//
// This is a binary to binary transfer (mostly).
//
// Negotiation looks like this
// 1. Client makes IPC Syscall: `HelloWorld://TalkToMe{ptr=0x12345,len=24}`
// 2. Kernel moves `TalkToMe` to shared page (including the str)
// 3. Server gets woken up, and gets a `HelloWorld://TalkToMe{ptr=0x24567,len=24}` request
// 4. Server makes IPC Syscall: `Reponse://HelloReponse{ptr=0x34567,len=24)}`
// 5. Kernel verifies output struct and copies struct into clients memory
// 6. Kernel wakes the client up and sends them a `HelloWorld://HelloReponse{ptr=0x34567,len=24}`

#[qos::client(
  client_of = "HelloWorld",
)]
mod hello_world_service {
  #[endpoint(
    reponse = HelloReponse
  )]
  pub struct TalkToMe {
    name: String
  }

  #[reponse]
  pub struct HelloReponse {
    reponse_string: String
  }
}

#[qos::permissions(
  stdout,
  bind("HelloWorld")
)]
fn main() {
  // Make the request
  let request = TalkToMe {
    name: "Bob"
  };

  let reponse = request.call()
    .expect("Unable to call 'HelloWorld' service!");

  println!("{}", reponse.response_string);
}
```
