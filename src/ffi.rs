use std::{ptr, slice};
/// C ABI v1: no allocation crosses the ABI. Caller owns both buffers.
/// Returns written length, negative required capacity, or zero for ABI misuse.
///
/// # Safety
/// Input must point to `input_len` initialized readable bytes (<=131072).
/// Output must point to `capacity` writable bytes (<=262144), or be null at
/// capacity zero. Buffers must not overlap and remain valid for this call.
/// Invalid/dangling pointers are caller UB and cannot be caught as Rust panics.
#[no_mangle]
pub unsafe extern "C" fn ap2_verify_v1(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    capacity: usize,
) -> isize {
    if input.is_null()
        || input_len > 131072
        || capacity > 262144
        || output.is_null() && capacity != 0
    {
        return 0;
    }
    let r = std::panic::catch_unwind(|| {
        // SAFETY: caller provides readable input under the documented ABI contract.
        let bytes = unsafe { slice::from_raw_parts(input, input_len) };
        let result = crate::verify_json(bytes);
        let encoded = serde_json::to_vec(&result).ok()?;
        if encoded.len() > 262144 {
            return None;
        }
        if capacity < encoded.len() {
            return Some(-(encoded.len() as isize));
        }
        // SAFETY: caller provides non-overlapping writable output of capacity bytes.
        unsafe { ptr::copy_nonoverlapping(encoded.as_ptr(), output, encoded.len()) };
        Some(encoded.len() as isize)
    });
    r.ok().flatten().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caller_owned_buffers_and_guards() {
        let input = b"{}";
        // SAFETY: valid initialized input; null output with zero capacity.
        let required = unsafe { ap2_verify_v1(input.as_ptr(), input.len(), ptr::null_mut(), 0) };
        assert!(required < 0);
        let n = (-required) as usize;
        let mut out = vec![0xAA; n + 2];
        // SAFETY: output region starts after first guard and has n bytes.
        let written =
            unsafe { ap2_verify_v1(input.as_ptr(), input.len(), out.as_mut_ptr().add(1), n) };
        assert_eq!(written, n as isize);
        assert_eq!(out[0], 0xAA);
        assert_eq!(out[n + 1], 0xAA);
        let result: serde_json::Value = serde_json::from_slice(&out[1..n + 1]).unwrap();
        assert_eq!(result["verified"], false);
        // SAFETY: valid pointers, insufficient capacity; no output written.
        let small = unsafe { ap2_verify_v1(input.as_ptr(), input.len(), out.as_mut_ptr(), 1) };
        assert_eq!(small, required);
        assert_eq!(out[0], 0xAA);
        // Invalid API shape returns before pointer dereference.
        assert_eq!(
            unsafe { ap2_verify_v1(ptr::null(), 0, ptr::null_mut(), 0) },
            0
        );
        assert_eq!(
            unsafe { ap2_verify_v1(input.as_ptr(), 131073, ptr::null_mut(), 0) },
            0
        );
        assert_eq!(
            unsafe { ap2_verify_v1(input.as_ptr(), 2, ptr::null_mut(), 5) },
            0
        );
    }
}
