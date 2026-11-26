#ifndef RADE_H
#define RADE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RADE_ABI_VERSION 1u

typedef struct RadeEngine RadeEngine;

uint32_t rade_abi_version(void);

RadeEngine *rade_engine_new_from_ruleset(const uint8_t *data, size_t len, char **err);
RadeEngine *rade_engine_new_from_yaml_dir(const char *path, char **err);

int32_t rade_engine_eval_json(RadeEngine *engine,
                              const char *event_json,
                              char **out_matches_json,
                              char **err);

int32_t rade_engine_eval_events_json(RadeEngine *engine,
                                     const char *events_json,
                                     char **out_matches_json,
                                     char **err);

void rade_engine_free(RadeEngine *engine);

void rade_string_free(char *s);

#ifdef __cplusplus
}
#endif

#endif
