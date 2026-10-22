#ifndef QUESTPDF_ANDROID_H
#define QUESTPDF_ANDROID_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* ABI 1. Status: 0 success, 1 caught managed error, 2 invalid output parameters.
 * On status 0, generate returns PDF bytes and run_tests returns UTF-8 JSON.
 * On status 1, generate returns UTF-8 diagnostics. run_tests returns JSON if the
 * suite ran, or UTF-8 diagnostics if suite initialization itself failed.
 * Free every non-null output with questpdf_free, even on failure. No NUL byte is
 * appended; use the returned length. The cache path must be writable UTF-8.
 */
int32_t questpdf_abi_version(void);
int32_t questpdf_generate(int32_t feature, const char *cache, uint8_t **output, int32_t *length);
int32_t questpdf_run_tests(const char *cache, uint8_t **output, int32_t *length);
void questpdf_free(void *buffer);
#ifdef __cplusplus
}
#endif
#endif
