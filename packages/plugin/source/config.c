#include "config.h"
#include "ini.h"
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/stat.h>
#include <fcntl.h>
#include "plugin_common.h"

#define DEFAULT_CONFIG_DATA \
    "; Global settings\n" \
    "[global]\n" \
    ";target_address = 192.168.1.133\n" \
    ";target_port = 9090\n" \
    "\n" \
    "; Game-specific settings\n" \
    ";[CUSA08010]\n" \
    ";original_tls_size = 0\n"

void create_default_config(void) {
    char dir_path[256];
    strncpy(dir_path, CONFIG_FILE_PATH, sizeof(dir_path));
    char* last_slash = strrchr(dir_path, '/');
    if (last_slash) {
        *last_slash = '\0';
        mkdir(dir_path, 0777);
    }

    int fd = open(CONFIG_FILE_PATH, O_WRONLY | O_CREAT | O_TRUNC, 0666);
    if (fd < 0) {
        final_printf("Failed to create default config file: %s\n", CONFIG_FILE_PATH);
        return;
    }

    write(fd, DEFAULT_CONFIG_DATA, strlen(DEFAULT_CONFIG_DATA));
    close(fd);
}

bool load_config(const char* titleid, PluginConfig* config) {
    if (access(CONFIG_FILE_PATH, F_OK) != 0) {
        create_default_config();
        final_printf("Created default config file: %s\n", CONFIG_FILE_PATH);
        return false;
    }

    ini_table_s *ini = ini_table_create();
    if (!ini_table_read_from_file(ini, CONFIG_FILE_PATH)) {
        final_printf("Failed to read config file: %s\n", CONFIG_FILE_PATH);
        ini_table_destroy(ini);
        return false;
    }

    // Read global settings
    const char *address = ini_table_get_entry(ini, "global", "target_address");
    const char *port = ini_table_get_entry(ini, "global", "target_port");

    if (address) strncpy(config->target_address, address, sizeof(config->target_address) - 1);
    if (port) config->target_port = atoi(port);

    // Read game-specific settings
    const char *tls_size = ini_table_get_entry(ini, titleid, "original_tls_size");
    if (tls_size) {
        config->original_tls_size = atoi(tls_size);
    } else {
        final_printf("No configuration found for title ID: %s\n", titleid);
        ini_table_destroy(ini);
        return false;
    }

    ini_table_destroy(ini);
    return true;
}