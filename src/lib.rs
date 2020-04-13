#![cfg_attr(all(not(test), not(feature = "std")), no_std)]

mod builder;

pub use builder::CommandBuilder;
