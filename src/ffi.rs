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
    // SAFETY: this exported entry delegates the same caller memory contract.
    unsafe { invoke(input, input_len, output, capacity, crate::verify_json) }
}

// A private verifier parameter permits real unwind-path tests without adding a
// panic trigger or a test-only operation to the production ABI.
unsafe fn invoke(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    capacity: usize,
    verify: impl FnOnce(&[u8]) -> serde_json::Value + std::panic::UnwindSafe,
) -> isize {
    if input.is_null()
        || input_len > 131072
        || capacity > 262144
        || output.is_null() && capacity != 0
    {
        return 0;
    }
    // Reject arithmetic overflow and overlapping caller ranges before creating
    // a slice or writing. This does not establish validity of arbitrary pointers.
    let Some(input_end) = input.addr().checked_add(input_len) else {
        return 0;
    };
    let Some(output_end) = output.addr().checked_add(capacity) else {
        return 0;
    };
    if input_len != 0 && capacity != 0 && input.addr() < output_end && output.addr() < input_end {
        return 0;
    }
    let r = std::panic::catch_unwind(|| {
        // SAFETY: caller provides readable input under the documented ABI contract.
        let bytes = unsafe { slice::from_raw_parts(input, input_len) };
        let result = verify(bytes);
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

#[cfg(test)]
mod safety_tests {
    use super::*;
    #[test]
    fn panic_is_contained_and_output_untouched() {
        let input = b"{}";
        let mut output = [0xA5; 1024];
        // SAFETY: valid non-overlapping live buffers. Panic injected only in
        // this private helper test; exported production ABI has no panic hook.
        let n = unsafe {
            invoke(
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output.len(),
                |_| panic!("synthetic panic"),
            )
        };
        assert_eq!(n, 0);
        assert!(output.iter().all(|b| *b == 0xA5));
        assert_eq!(input, b"{}");
    }
    #[test]
    fn overlap_and_limits_reject_before_access() {
        let mut buffer = [b' '; 128];
        buffer[..2].copy_from_slice(b"{}");
        let ptr = buffer.as_mut_ptr();
        // SAFETY: allocated pointers; ranges overlap but the ABI rejects before
        // constructing any aliased references or executing a copy.
        assert_eq!(unsafe { ap2_verify_v1(ptr, 2, ptr, 128) }, 0);
        assert_eq!(unsafe { ap2_verify_v1(ptr, 64, ptr.add(32), 64) }, 0);
        assert_eq!(unsafe { ap2_verify_v1(ptr.add(32), 64, ptr, 64) }, 0);
        assert_eq!(unsafe { ap2_verify_v1(ptr, 2, ptr, 262145) }, 0);
        assert_eq!(&buffer[..2], b"{}");
    }
    #[test]
    fn capacity_sweep_and_repeated_ownership() {
        let input = b"{}";
        let expected = serde_json::to_vec(&crate::verify_json(input)).unwrap();
        for capacity in 0..=expected.len() + 2 {
            let mut output = vec![0xC7; capacity + 2];
            // SAFETY: distinct live buffers; guard bytes lie outside capacity.
            let n = unsafe {
                ap2_verify_v1(
                    input.as_ptr(),
                    input.len(),
                    output.as_mut_ptr().add(1),
                    capacity,
                )
            };
            assert_eq!(output[0], 0xC7);
            assert_eq!(output[capacity + 1], 0xC7);
            if capacity < expected.len() {
                assert_eq!(n, -(expected.len() as isize));
                assert!(output.iter().all(|b| *b == 0xC7));
            } else {
                assert_eq!(n, expected.len() as isize);
                assert_eq!(&output[1..expected.len() + 1], expected);
                assert!(output[expected.len() + 1..].iter().all(|b| *b == 0xC7));
            }
        }
        for _ in 0..128 {
            let mut out = vec![0; 262144];
            assert!(
                unsafe { ap2_verify_v1(input.as_ptr(), input.len(), out.as_mut_ptr(), out.len()) }
                    > 0
            );
            // Drop the caller-owned buffer immediately, then allocate another.
        }
        assert_eq!(input, b"{}");
    }
    #[test]
    fn request_at_byte_limit_uses_only_declared_input() {
        let mut input = vec![b' '; 131074];
        input[..2].copy_from_slice(b"{}");
        input[131072] = 0xFF;
        let mut output = vec![0; 1024];
        assert!(
            unsafe { ap2_verify_v1(input.as_ptr(), 131072, output.as_mut_ptr(), output.len()) } > 0
        );
        assert_eq!(
            unsafe { ap2_verify_v1(input.as_ptr(), 131073, output.as_mut_ptr(), output.len()) },
            0
        );
        assert_eq!(input[131072], 0xFF);
    }
}
