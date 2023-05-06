/*
  ____                 __               __   _ __
 / __ \__ _____ ____  / /___ ____ _    / /  (_) /
/ /_/ / // / _ `/ _ \/ __/ // /  ' \  / /__/ / _ \
\___\_\_,_/\_,_/_//_/\__/\_,_/_/_/_/ /____/_/_.__/
  Part of the Quantum OS Project

Copyright 2023 Gavin Kellam

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

use crate::Nothing;
use core::marker::PhantomData;

pub struct SendingPortal;
pub struct ReceivingPortal;
pub struct BothPortal;
pub struct PortalUnknown;

pub struct Portal<FunctionType, PortalType = PortalUnknown, Bi = Nothing, Other = Nothing> {
    stream: FunctionType,
    inlet_data: PhantomData<Bi>,
    outlet_data: PhantomData<Other>,
    portal_type: PhantomData<PortalType>,
}

impl<FunctionType, Inlet> Portal<FunctionType, PortalUnknown, Inlet>
where
    FunctionType: FnMut(Inlet),
{
    pub fn new_sending_portal(func: FunctionType) -> Portal<FunctionType, SendingPortal, Inlet> {
        Portal {
            stream: func,
            inlet_data: Default::default(),
            outlet_data: Default::default(),
            portal_type: Default::default(),
        }
    }
}

impl<FunctionType, Outlet> Portal<FunctionType, PortalUnknown, Outlet>
where
    FunctionType: FnMut() -> Outlet,
{
    pub fn new_recv_portal(func: FunctionType) -> Portal<FunctionType, ReceivingPortal, Outlet> {
        Portal {
            stream: func,
            inlet_data: Default::default(),
            outlet_data: Default::default(),
            portal_type: Default::default(),
        }
    }
}

impl<FunctionType, Inlet, Outlet> Portal<FunctionType, PortalUnknown, Inlet, Outlet>
where
    FunctionType: FnMut(Inlet) -> Outlet,
{
    pub fn new_both_portal(func: FunctionType) -> Portal<FunctionType, BothPortal, Inlet, Outlet> {
        Portal {
            stream: func,
            inlet_data: Default::default(),
            outlet_data: Default::default(),
            portal_type: Default::default(),
        }
    }
}

impl<FunctionType, Inlet> Portal<FunctionType, SendingPortal, Inlet>
where
    FunctionType: FnMut(Inlet),
{
    pub fn send_data(&mut self, data: Inlet) {
        (self.stream)(data);
    }
}

impl<FunctionType, Outlet> Portal<FunctionType, ReceivingPortal, Outlet>
where
    FunctionType: FnMut() -> Outlet,
{
    pub fn recv_data(&mut self) -> Outlet {
        (self.stream)()
    }
}

impl<FunctionType, Inlet, Outlet> Portal<FunctionType, BothPortal, Inlet, Outlet>
where
    FunctionType: FnMut(Inlet) -> Outlet,
{
    pub fn both_data(&mut self, data: Inlet) -> Outlet {
        (self.stream)(data)
    }
}
