#include "rade.h"
#include <stdio.h>
#include <string.h>

int main(void) {
    char *err = NULL;
    RadeEngine *engine = rade_engine_new_from_yaml_dir("rade/test_data/rules", &err);
    if (!engine) {
        fprintf(stderr, "new failed: %s\n", err ? err : "?");
        rade_string_free(err);
        return 1;
    }

    char *out = NULL;
    int32_t rc = rade_engine_eval_json(
        engine,
        "{\"process_name\":\"powershell\","
        "\"content\":\"powershell -EncodedCommand SGVsbG8=\"}",
        &out, &err);

    if (rc != 0) {
        fprintf(stderr, "eval failed: %s\n", err ? err : "?");
        rade_string_free(err);
        rade_engine_free(engine);
        return 1;
    }

    printf("%s\n", out);

    int ok = out != NULL && strstr(out, "\"matched\":true") != NULL;
    rade_string_free(out);
    rade_string_free(err);
    rade_engine_free(engine);
    return ok ? 0 : 1;
}
