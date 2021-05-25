// Copyright (c) 2020 Huawei Technologies Co.,Ltd. All rights reserved.
//
// lib-shim-v2 is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan
// PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//         http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY
// KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

#ifndef LIB_SHIM_V2_H
#define LIB_SHIM_V2_H

#include <stdint.h>

struct DeleteResponse {
    unsigned int exit_status;
    unsigned int pid;
};

enum Status {
    UnknownStatus = 0,
    CreatedStatus,
    RunningStatus,
    StoppedStatus,
    DeletedStatus,
    PauseStatus,
    PausingStatus,
};

struct State {
    const char *id;
    unsigned int pid;
    enum Status status;
    const char *stdin;
    const char *stdout;
    const char *stderr;
    bool terminal;
    unsigned int exit_status;
};

int shim_v2_new(const char *container_id, const char *addr);
int shim_v2_close(const char *container_id);

int shim_v2_create(const char *container_id, const char *bundle, bool terminal,
                   const char *stdin, const char *stdout, const char *stderr, int *pid);
int shim_v2_start(const char *container_id, const char *exec_id, int *pid);
int shim_v2_kill(const char *container_id, const char *exec_id, unsigned int signal, bool all);
int shim_v2_delete(const char *container_id, const char *exec_id, const struct DeleteResponse *resp);
int shim_v2_shutdown(const char *container_id);

int shim_v2_exec(const char *container_id, const char *exec_id, bool terminal,
                 const char *stdin, const char *stdout, const char *stderr, const char *spec);
int shim_v2_resize_pty(const char *container_id, const char *exec_id, unsigned int height, unsigned int width);

int shim_v2_pause(const char *container_id);
int shim_v2_resume(const char *container_id);

int shim_v2_state(const char *container_id, const struct State *state);

int shim_v2_pids(const char *container_id, int *pid);
#endif /* LIB_SHIM_V2_H */
