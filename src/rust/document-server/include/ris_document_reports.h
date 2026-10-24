#ifndef RIS_DOCUMENT_REPORTS_H
#define RIS_DOCUMENT_REPORTS_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Status: 0 success, 1 invalid input, 2 render/transport failure.
 * Release BOTH pdf and error with doc_reports_free_buffer. */
int32_t doc_reports_generate_pdf(const char *type, const uint8_t *json, int32_t json_len,
                               void **pdf, int32_t *pdf_len, void **error);
int32_t doc_reports_print_receipt(const char *type, const uint8_t *json, int32_t json_len,
                                const char *host, int32_t port, void **error);
void doc_reports_free_buffer(void *buffer);
#ifdef __cplusplus
}
#endif
#endif
