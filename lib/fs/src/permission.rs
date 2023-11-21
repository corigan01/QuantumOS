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
*
*/

pub struct GroupId(usize);
pub struct UserId(usize);

pub const SET_UID_PERM_BIT: usize = 0x800;
pub const SET_GID_PERM_BIT: usize = 0x400;
pub const SICKY_PERM_BIT: usize = 0x200;

pub const R_OWNER_PERM_BIT: usize = 0x100;
pub const W_OWNER_PERM_BIT: usize = 0x080;
pub const X_OWNER_PERM_BIT: usize = 0x040;

pub const R_GROUP_PERM_BIT: usize = 0x020;
pub const W_GROUP_PERM_BIT: usize = 0x010;
pub const X_GROUP_PERM_BIT: usize = 0x008;

pub const R_OTHER_PERM_BIT: usize = 0x004;
pub const W_OTHER_PERM_BIT: usize = 0x002;
pub const X_OTHER_PERM_BIT: usize = 0x001;

pub struct Permissions {
    permission: u16,
    uid: UserId,
    gid: GroupId,
}
