#ifndef LIB_SHIM_V2_H
#define LIB_SHIM_V2_H

#include <stdint.h>

int shim_v2_new(const char *container_id, const char *addr);
int shim_v2_create(const char *container_id, const char *bundle, bool terminal,
                   const char *stdin, const char *stdout, const char *stderr, int *pid);
int shim_v2_start(const char *container_id, const char *exec_id, int *pid);
#endif /* LIB_SHIM_V2_H */