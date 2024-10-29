```rust
hw_device! {
	#[provider]
	pub mod DingusArea {
		// `const` is optional
		// this function will be set in `::DingusArea::read()`
		// Optional if no RO,RW
		pub const fn read() -> u64 {
			todo!()
		}

		// Optional if no RW/WO
		pub const unsafe fn write(dingus: u64) {
			todo!()
		}
	}

	#[field(RO, 16, DingusArea)]
	pub register_enable,

	#[field(RW, 0..12, DingusArea)]
	pub virt_address,

	// ------------------------------------------------------------------

	// You can define fields to be re-useable
	ReuseThis = generic! {
		#[field(RO, 0)]
		dingus
	}

	// ------------------------------------------------------------------

	// Structs are possible instead of modules for when you want instances
	// if registers.
	#[new = 0]
	#[impl(ReuseThis)]
	pub struct Dingus(u64);

	// This will be inside a impl block for `Dingus`
	#[field(RW, 0..32, Dingus)]
	pub something,

	// This will be inside a impl block for `Dingus`
	#[custom(Dingus)]
	pub fn custom_function(&mut self, idk: u32) {
		self.0 &= idk;
	}

}
```

Should expand to:

```rust

pub mod DingusArea {
	pub const fn read() -> u64 {
		todo!()
	}

	pub const fn write(dingus: u64) {
		todo!()
	}
}

pub const fn is_register_enabled() -> bool {
	DingusArea::read() & 16 != 0
}

// virt_address ... 

pub struct Dingus(u64);

impl Dingus {
	pub const fn new() -> Self {
		Self (0)
	}

	pub const fn is_dingus_set(&self) -> bool {
		self.0 & 1 != 0
	}

	pub const fn set_something(&mut self, value: u32) -> Self {
		self.0 = (self.0 & (u32::MAX as u64) | (value as u64);
		*self.0
	}

	pub const fn get_something(&self) -> u32 {
		*self.0 as u32
	}

	pub fn custom_function(&mut self, idk: u32) {
		self.0 &= idk;
	}
}

```
