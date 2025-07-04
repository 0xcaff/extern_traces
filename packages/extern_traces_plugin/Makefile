DEBUG_FLAGS = -D__FINAL__=1
LOG_TYPE = -D__USE_PRINTF__
BUILD_TYPE = _final

ifeq ($(DEBUG),1)
    DEBUG_FLAGS = -D__FINAL__=0
    BUILD_TYPE = _debug
endif

TYPE         := $(BUILD_TYPE)
FINAL        := $(DEBUG_FLAGS)
BUILD_FOLDER := $(shell pwd)/target
OUTPUT_PRX   := extern_traces
TARGET       := $(BUILD_FOLDER)/prx$(TYPE)/$(OUTPUT_PRX)
TARGET_ELF   := $(BUILD_FOLDER)/elf$(TYPE)/$(OUTPUT_PRX)
TARGETSTUB   := $(OUTPUT_PRX).so

# Libraries linked into the ELF.
LIBS := -lSceLibcInternal -lGoldHEN_Hook -lkernel


EXTRAFLAGS := $(DEBUG_FLAGS) $(LOG_TYPE) -fcolor-diagnostics -Wall -Wno-implicit-function-declaration

# You likely won't need to touch anything below this point.
# Root vars
TOOLCHAIN     := $(OO_PS4_TOOLCHAIN)
GH_SDK        := $(GOLDHEN_SDK)
PROJDIR       := $(shell pwd)/source
INTDIR        := $(shell pwd)/build
INCLUDEDIR    := $(shell pwd)/include
COMMON_DIR    := ./common

# Define objects to build
CFILES      := $(wildcard $(PROJDIR)/*.c)
CPPFILES    := $(wildcard $(PROJDIR)/*.cpp)
COMMONFILES := $(wildcard $(COMMONDIR)/*.cpp)
OBJS        := $(patsubst $(PROJDIR)/%.c, $(INTDIR)/%.o, $(CFILES)) $(patsubst $(PROJDIR)/%.cpp, $(INTDIR)/%.o, $(CPPFILES)) $(patsubst $(COMMONDIR)/%.cpp, $(INTDIR)/%.o, $(COMMONFILES))
STUBOBJS    := $(patsubst $(PROJDIR)/%.c, $(INTDIR)/%.o, $(CFILES)) $(patsubst $(PROJDIR)/%.cpp, $(INTDIR)/%.o.stub, $(CPPFILES)) $(patsubst $(COMMONDIR)/%.cpp, $(INTDIR)/%.o.stub, $(COMMONFILES))

# Define final C/C++ flags
CFLAGS      := $(FINAL) --target=x86_64-pc-freebsd12-elf -fPIC -funwind-tables -c $(EXTRAFLAGS) -isysroot $(TOOLCHAIN) -isystem $(TOOLCHAIN)/include -I$(GH_SDK)/include -I$(INCLUDEDIR) -I$(COMMON_DIR) $(O_FLAG)
CXXFLAGS    := $(CFLAGS) -isystem $(TOOLCHAIN)/include/c++/v1
LDFLAGS     := -m elf_x86_64 -pie --script $(TOOLCHAIN)/link.x -e _init --eh-frame-hdr -L$(TOOLCHAIN)/lib -L$(GH_SDK) $(LIBS)

# Create the intermediate directory incase it doesn't already exist.
_unused     := $(shell mkdir -p $(INTDIR))

# Check for linux vs macOS and account for clang/ld path
UNAME_S     := $(shell uname -s)

ifeq ($(UNAME_S),Linux)
        CC      := clang
        CCX     := clang++
        LD      := ld.lld
        CDIR    := linux
endif
ifeq ($(UNAME_S),Darwin)
        CC      := /usr/local/opt/llvm/bin/clang
        CCX     := /usr/local/opt/llvm/bin/clang++
        LD      := /usr/local/opt/llvm/bin/ld.lld
        CDIR    := macos
endif

./plugin_support/target/x86_64-unknown-freebsd/release/libplugin_support.a:
	cargo -Z unstable-options -C plugin_support build --target x86_64-unknown-freebsd --release

$(TARGET): $(INTDIR) $(OBJS) ./plugin_support/target/x86_64-unknown-freebsd/release/libplugin_support.a
	$(LD) $(GH_SDK)/build/crtprx.o $(INTDIR)/*.o ./plugin_support/target/x86_64-unknown-freebsd/release/libplugin_support.a -o $(TARGET_ELF).elf $(LDFLAGS)
	$(TOOLCHAIN)/bin/$(CDIR)/create-fself -in=$(TARGET_ELF).elf -out=$(TARGET_ELF).oelf --lib=$(TARGET).prx --paid 0x3800000000000011

$(TARGETSTUB): $(INTDIR) $(STUBOBJS)
	$(CC) $(INTDIR)/*.o.stub -o $(TARGETSTUB) -target x86_64-pc-linux-gnu -shared -fuse-ld=lld -ffreestanding -nostdlib -fno-builtin -L$(TOOLCHAIN)/lib $(LIBS)

$(INTDIR)/%.o: $(PROJDIR)/%.c
	$(CC) $(CFLAGS) -o $@ $<

$(INTDIR)/%.o: $(PROJDIR)/%.cpp
	$(CCX) $(CXXFLAGS) -o $@ $<

$(INTDIR)/%.o.stub: $(PROJDIR)/%.c
	$(CC) -target x86_64-pc-linux-gnu -ffreestanding -nostdlib -fno-builtin -fPIC $(O_FLAG) -s -c -o $@ $<

$(INTDIR)/%.o.stub: $(PROJDIR)/%.cpp
	$(CCX) -target x86_64-pc-linux-gnu -ffreestanding -nostdlib -fno-builtin -fPIC $(O_FLAG) -s -c -o $@ $<

plugin_common:
	$(CC) $(CFLAGS) -o $(INTDIR)/plugin_common.o $(COMMON_DIR)/plugin_common.c

config:
	curl -T config.ini ftp://192.168.1.123:2121/data/extern_traces.ini

.PHONY: clean config
.DEFAULT_GOAL := all

all: plugin_common $(TARGET)

deploy: all config
	curl -T target/prx_final/extern_traces.prx ftp://192.168.1.123:2121/data/GoldHEN/plugins/extern_traces.prx

clean:
	rm -rf $(TARGET) $(TARGETSTUB) $(INTDIR) $(OBJS)
