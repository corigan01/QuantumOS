/*
  ____                 __               __  __
 / __ \__ _____ ____  / /___ ____ _    / / / /__ ___ ____
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /_/ (_-</ -_) __/
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/  \____/___/\__/_/
  Part of the Quantum OS Kernel

Copyright 2025 Gavin Kellam

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#![no_std]
#![no_main]
tiny_std!();

use core::marker::PhantomData;

use libq::{RecvHandleError, close, connect, dbugln, recv, send, tiny_std, yield_now};

trait HelloClient {
    fn get_hello() -> u32;
}

trait HelloServer: Sized {
    fn parse_request(&mut self) -> Result<HelloRequest<Self>, ()>;
}

struct IpcResponder<'a, C: HelloServer, T> {
    serv: &'a C,
    ty: PhantomData<T>,
}

enum HelloRequest<'a, C: HelloServer> {
    GetHello { reponse: IpcResponder<'a, C, u32> },
}

struct Dingus(u64);

impl HelloServer for Dingus {
    fn parse_request<'a>(&'a mut self) -> Result<HelloRequest<'a, Self>, ()> {
        Ok(HelloRequest::GetHello {
            reponse: IpcResponder {
                serv: self,
                ty: PhantomData,
            },
        })
    }
}

impl<'a, C: HelloServer, T> IpcResponder<'a, C, T> {
    fn respond_with(self, data: T) -> Result<(), ()> {
        todo!()
    }
}

fn test() {
    let mut dingus = Dingus(123);

    let result = dingus.parse_request().unwrap();
    match result {
        HelloRequest::GetHello { reponse } => {
            reponse.respond_with(123).unwrap();
        }
    }
}

fn main() {
    dbugln!("Starting Dummy");

    let handle = loop {
        match connect("hello-server") {
            Ok(okay) => {
                dbugln!("Connected to `hello-server` at handle {okay}!");
                break okay;
            }
            Err(e) => {
                dbugln!("Error connecting to `hello-server` {e:?}...");
                yield_now();
            }
        }
    };

    send(handle, &[0xde, 0xad, 0xbe, 0xef]).unwrap();

    let mut data_buf = [0; 16];
    loop {
        match recv(handle, &mut data_buf) {
            Ok(b) => {
                dbugln!("Got {b} bytes! {:02x?}", &data_buf[..b]);
                break;
            }
            Err(RecvHandleError::WouldBlock) => yield_now(),
            Err(err) => {
                dbugln!("Error {err:?}!");
                break;
            }
        }
    }

    close(handle);
}
