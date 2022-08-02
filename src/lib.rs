mod scene;
mod utils;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PANIC_HOOK: () = {
        utils::set_panic_hook();
    };
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
