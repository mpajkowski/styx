#![no_std]

pub mod framebuffer;
pub mod memmap;
pub mod terminal;

#[macro_export]
macro_rules! make_struct {
    (
        $(#[$meta:meta])*
        struct $name:ident: [$id1:expr, $id2:expr] => $response_ty:ty {
            $($(#[$field_meta:meta])* $field_name:ident : $field_ty:ty = $field_default:expr),*
        };
    ) => {
        $(#[$meta])*
        #[repr(C)]
        #[derive(Debug)]
        pub struct $name {
            id: [u64; 4],
            revision: u64,

            // NOTE: The response is required to be wrapped inside an unsafe cell, since
            // by default the response is set to NULL and when the compiler does not see
            // any writes to the field, it is free to assume that the response is NULL. In
            // our situation the bootloader mutates the field and we need to ensure that
            // the compiler does not optimize the read away.
            response: core::cell::UnsafeCell<*const $response_ty>,
            $(pub $field_name: $field_ty),*
        }

        impl $name {
            // NOTE: The request ID is composed of 4 64-bit wide unsigned integers but the first
            // two remain constant. This is refered as `LIMINE_COMMON_MAGIC` in the limine protocol
            // header.
            pub const ID: [u64; 4] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, $id1, $id2];


            pub const fn new(revision: u64) -> Self {
                Self {
                    id: Self::ID,
                    revision,


                    response: core::cell::UnsafeCell::new(core::ptr::null()),
                    $($field_name: $field_default),*
                }
            }

            pub fn response(&self) -> Option<&'static $response_ty> {
                unsafe {
                    let ptr = self.response.get();

                    if !core::ptr::eq(ptr, core::ptr::null()) {
                        Some(&*core::ptr::read_volatile(self.response.get()) as &'static $response_ty)
                    } else {
                        None
                    }
                }
            }
        }

        // maker trait implementations for limine request struct:
        unsafe impl Sync for $name {}
    };
}
