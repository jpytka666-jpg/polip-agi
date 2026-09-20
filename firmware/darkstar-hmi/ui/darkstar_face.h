#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "lvgl.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    DARKSTAR_STATUS_OFFLINE = 0,
    DARKSTAR_STATUS_STANDBY,
    DARKSTAR_STATUS_ONLINE,
    DARKSTAR_STATUS_WARNING,
    DARKSTAR_STATUS_ALARM
} darkstar_status_t;

typedef struct {
    darkstar_status_t host;
    darkstar_status_t aions;
    darkstar_status_t net;
    darkstar_status_t gpu;
    darkstar_status_t thermal;
    const char *node_name;       /* e.g. "R730 / NODE 01" */
    const char *host_text;       /* e.g. "ONLINE" */
    const char *aions_text;      /* e.g. "STANDBY" */
    const char *net_text;        /* e.g. "LINK" */
    const char *gpu_text;        /* e.g. "READY" */
    const char *thermal_text;    /* e.g. "NOMINAL" */
    bool host_lost;
    uint32_t seconds_since_contact;
} darkstar_face_state_t;

void darkstar_face_init(lv_obj_t *root);
void darkstar_face_set_status(const darkstar_face_state_t *state);

#ifdef __cplusplus
}
#endif
