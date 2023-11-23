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

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct GroupId(usize);

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct UserId(usize);

impl GroupId {
    pub fn root() -> Self {
        Self(0)
    }
}

impl UserId {
    pub fn root() -> Self {
        Self(0)
    }
}

pub const SET_UID_PERM_BIT: u16 = 0x800;
pub const SET_GID_PERM_BIT: u16 = 0x400;
pub const SICKY_PERM_BIT: u16 = 0x200;

pub const R_OWNER_PERM_BIT: u16 = 0x100;
pub const W_OWNER_PERM_BIT: u16 = 0x080;
pub const X_OWNER_PERM_BIT: u16 = 0x040;

pub const R_GROUP_PERM_BIT: u16 = 0x020;
pub const W_GROUP_PERM_BIT: u16 = 0x010;
pub const X_GROUP_PERM_BIT: u16 = 0x008;

pub const R_OTHER_PERM_BIT: u16 = 0x004;
pub const W_OTHER_PERM_BIT: u16 = 0x002;
pub const X_OTHER_PERM_BIT: u16 = 0x001;

pub const fn permission_from_octal(val: u16) -> u16 {
    let owner_group = (val / 100) % 7;
    let group_group = (val / 10) % 7;
    let other_group = val % 7;

    let mut pc = 0_u16;

    pc |= match owner_group {
        0 => 0,
        1 => X_OWNER_PERM_BIT,
        2 => W_OWNER_PERM_BIT,
        3 => W_OWNER_PERM_BIT | X_OWNER_PERM_BIT,
        4 => R_OWNER_PERM_BIT,
        5 => R_OWNER_PERM_BIT | X_OWNER_PERM_BIT,
        6 => R_OWNER_PERM_BIT | W_OWNER_PERM_BIT,
        7 => R_OWNER_PERM_BIT | W_OWNER_PERM_BIT | X_OWNER_PERM_BIT,
        _ => panic!("Cannot have permission over 7"),
    };

    pc |= match group_group {
        0 => 0,
        1 => X_GROUP_PERM_BIT,
        2 => W_GROUP_PERM_BIT,
        3 => W_GROUP_PERM_BIT | X_GROUP_PERM_BIT,
        4 => R_GROUP_PERM_BIT,
        5 => R_GROUP_PERM_BIT | X_GROUP_PERM_BIT,
        6 => R_GROUP_PERM_BIT | W_GROUP_PERM_BIT,
        7 => R_GROUP_PERM_BIT | W_GROUP_PERM_BIT | X_GROUP_PERM_BIT,
        _ => panic!("Cannot have permission over 7"),
    };

    pc |= match other_group {
        0 => 0,
        1 => X_OTHER_PERM_BIT,
        2 => W_OTHER_PERM_BIT,
        3 => W_OTHER_PERM_BIT | X_OTHER_PERM_BIT,
        4 => R_OTHER_PERM_BIT,
        5 => R_OTHER_PERM_BIT | X_OTHER_PERM_BIT,
        6 => R_OTHER_PERM_BIT | W_OTHER_PERM_BIT,
        7 => R_OTHER_PERM_BIT | W_OTHER_PERM_BIT | X_OTHER_PERM_BIT,
        _ => panic!("Cannot have permission over 7"),
    };

    pc
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Permissions {
    pub permission: u16,
    pub uid: UserId,
    pub gid: GroupId,
}

impl Permissions {
    pub fn none() -> Self {
        Self {
            permission: permission_from_octal(000),
            uid: UserId::root(),
            gid: GroupId::root(),
        }
    }

    pub fn all() -> Self {
        Self {
            permission: permission_from_octal(777),
            uid: UserId::root(),
            gid: GroupId::root(),
        }
    }

    pub fn root_rw() -> Self {
        Self {
            permission: permission_from_octal(600),
            uid: UserId::root(),
            gid: GroupId::root(),
        }
    }

    pub fn root_rwx() -> Self {
        Self {
            permission: permission_from_octal(700),
            uid: UserId::root(),
            gid: GroupId::root(),
        }
    }
}
