use std::ffi::CString;

pub fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));

    return unsafe { CString::from_vec_unchecked(buffer) };
}


