#![no_std]

// Compile-only: requests are never passed to a loader in this probe.
use limine::{request::{FramebufferRequest, MemoryMapRequest, RsdpRequest}, BaseRevision};

pub static BASE: BaseRevision = BaseRevision::with_revision(3);
pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();
pub static MEMORY: MemoryMapRequest = MemoryMapRequest::new();
pub static RSDP: RsdpRequest = RsdpRequest::new();

// x86-64 request header in the pinned C protocol: four ID words, revision,
// response pointer. These assertions only check the host-compiled type shape.
const _: () = assert!(core::mem::size_of::<FramebufferRequest>() == 48);
const _: () = assert!(core::mem::size_of::<MemoryMapRequest>() == 48);
const _: () = assert!(core::mem::size_of::<RsdpRequest>() == 48);
const _: () = assert!(core::mem::align_of::<RsdpRequest>() == 8);
