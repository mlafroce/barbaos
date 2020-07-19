pub mod error;
pub mod handlers;
pub mod macros;

pub struct NullTerminatedStr;

impl NullTerminatedStr {
    pub fn len(address: *const u8) -> usize {
        let mut char_ptr = address;
        let mut size = 0;
        while unsafe { *char_ptr } != 0 {
            size += 1;
            char_ptr = (char_ptr as usize + 1) as *const u8;
        }
        size
    }

    pub unsafe fn as_bytes(address: *const u8) -> &'static [u8] {
        core::slice::from_raw_parts(address, NullTerminatedStr::len(address))
    }

    pub unsafe fn as_str(address: *const u8) -> &'static str {
        let bytes = NullTerminatedStr::as_bytes(address);
        core::str::from_utf8_unchecked(bytes)
    }
}
