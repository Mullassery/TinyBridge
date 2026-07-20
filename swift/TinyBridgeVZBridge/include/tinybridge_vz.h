#ifndef TINYBRIDGE_VZ_H
#define TINYBRIDGE_VZ_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef struct TBVirtualMachine TBVirtualMachine;

typedef enum {
    TB_VM_STATE_STOPPED = 0,
    TB_VM_STATE_STARTING = 1,
    TB_VM_STATE_RUNNING = 2,
    TB_VM_STATE_STOPPING = 3,
    TB_VM_STATE_ERROR = 4,
} TBVMState;

typedef void (*TBVMStateCallback)(
    TBVirtualMachine *vm,
    TBVMState new_state,
    const char *error_message,
    void *user_data
);

typedef struct {
    const char *kernel_path;
    const char *initrd_path;
    const char *cmdline;
    const char *disk_image_path;
    uint32_t cpu_count;
    uint64_t memory_bytes;
    bool enable_rosetta;
    TBVMStateCallback state_callback;
    void *user_data;
} TBVMConfig;

typedef struct {
    const char *host_path;
    const char *mount_tag;
    bool read_only;
} TBVirtioFSConfig;

typedef struct {
    TBVMState state;
    double cpu_usage_pct;
    uint64_t memory_used_bytes;
    uint64_t memory_total_bytes;
    char ip_address[46];
} TBVMStatus;

TBVirtualMachine *tb_vm_create(const TBVMConfig *config);
int tb_vm_start(TBVirtualMachine *vm);
int tb_vm_stop(TBVirtualMachine *vm);
int tb_vm_force_stop(TBVirtualMachine *vm);
void tb_vm_destroy(TBVirtualMachine *vm);

int tb_vm_add_virtiofs(TBVirtualMachine *vm, const TBVirtioFSConfig *config);

int tb_vm_get_status(TBVirtualMachine *vm, TBVMStatus *out_status);
int tb_vm_get_ip(TBVirtualMachine *vm, char *buf, size_t buf_len);

const char *tb_version(void);
bool tb_is_available(void);

#endif
