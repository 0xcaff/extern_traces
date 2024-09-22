#ifndef CONFIG_H
#define CONFIG_H

#include <stdbool.h>
#include <stdint.h>

#define CONFIG_FILE_PATH "/data/GoldHEN/plugins/extern_traces.ini"

typedef struct {
    char target_address[256];
    uint16_t target_port;
    int original_tls_size;
} PluginConfig;

bool load_config(const char* titleid, PluginConfig* config);
void create_default_config(void);

#endif // CONFIG_H