#include <stdbool.h>

#include "plugin_common.h"

attr_public const char *g_pluginName = "patch_resolver";
attr_public const char *g_pluginDesc = "Patch dns forward resolution";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

unsigned int pack(unsigned char octet1, unsigned char octet2, unsigned char octet3, unsigned char octet4) {
	return octet1 | (octet2 << 8) | (octet3 << 16) | (octet4 << 24);
}

typedef uint32_t SceNetInAddr_t;
typedef struct SceNetInAddr {
	SceNetInAddr_t s_addr;
} SceNetInAddr;

typedef uint32_t SceNetInAddr_t;
typedef int SceNetId;

extern SceNetId sceNetResolverStartNtoa(
	SceNetId rid,
	const char *hostname,
	SceNetInAddr *addr,
	int timeout,
	int retry,
	int flags
);

HOOK_INIT(sceNetResolverStartNtoa);

SceNetId resolve(
	SceNetId rid,
	const char *hostname,
	SceNetInAddr *addr,
	int timeout,
	int retry,
	int flags
) {
	if (strcmp(hostname, "zhaxxy.buzz") == 0) {
		addr->s_addr = pack(192, 168, 2, 1);
		return 0;
	}

	SceNetId result = HOOK_CONTINUE(
		sceNetResolverStartNtoa,
		SceNetId (*)(SceNetId, const char *, SceNetInAddr *, int, int, int),
		rid, hostname, addr, timeout, retry, flags
	);

	return result;
}

SceNetId sceNetResolverStartNtoa_hook(
	SceNetId rid,
	const char *hostname,
	SceNetInAddr *addr,
	int timeout,
	int retry,
	int flags
) {
	SceNetId result = resolve(rid, hostname, addr, timeout, retry, flags);

    unsigned char byte1 = (addr->s_addr) & 0xFF;
    unsigned char byte2 = (addr->s_addr >> 8) & 0xFF;
    unsigned char byte3 = (addr->s_addr >> 16) & 0xFF;
	unsigned char byte4 = (addr->s_addr >> 24) & 0xFF;

    final_printf("[sceNetResolverStartNtoa] hostname: \"%s\", address: \"%u.%u.%u.%u\"\n", hostname, byte1, byte2, byte3, byte4);

	return result;
}

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    HOOK32(sceNetResolverStartNtoa);

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);
    return 0;
}

