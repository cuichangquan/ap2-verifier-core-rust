#ifndef AP2_VERIFIER_H
#define AP2_VERIFIER_H
#include <stddef.h>
#include <stdint.h>
/* Caller-owned, non-overlapping buffers. Input <=131072 bytes; capacity <=262144.
 * Returns written length; negative required capacity (no output written); 0 for
 * ABI/internal failure. No NUL terminator. Never authorize on missing results.
 * Valid pointers and lifetimes are caller responsibility; no retained pointers.
 */
#ifdef __cplusplus
extern "C" {
#endif
intptr_t ap2_verify_v1(const uint8_t *input, size_t input_len, uint8_t *output, size_t capacity);
#ifdef __cplusplus
}
#endif
/* Overlapping address ranges and size arithmetic overflow reject with 0 before
 * access. This does not validate allocation extents or dangling pointers. */
#endif
