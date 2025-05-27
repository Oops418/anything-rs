// quicklook_shim.h
#pragma once
#ifdef __cplusplus
extern "C" {
#endif

void ql_preview(const char *path);

void ql_close(void);

#ifdef __cplusplus
}
#endif
