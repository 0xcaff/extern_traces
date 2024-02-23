#include "logging.h"
#include <string.h>

int printf(const char*, ...);

typedef unsigned int uint32_t;
typedef unsigned long uint64_t;

#define HIGH_WATER_MARK_BYTES (16 * 1024)
#define MAX_BUFFERED_BYTES (64 * 1024)

_Static_assert(
    HIGH_WATER_MARK_BYTES < MAX_BUFFERED_BYTES,
    "high water mark must be smaller than max buffer size");

// Not sure of the exact size of the mutex, so blocking off more bytes than we
// probably need.
typedef char ScePthreadMutex[1024];

#define MAX_FLUSH_QUEUE_ENTRIES 1024

typedef struct FlushQueueEntry
{
    char buffer[MAX_BUFFERED_BYTES];
    uint32_t buffer_len;
} FlushQueueEntry;

typedef struct FlushQueue
{
    FlushQueueEntry entries[MAX_FLUSH_QUEUE_ENTRIES];
    uint32_t read_head_idx;
    uint32_t write_head_idx;
} FlushQueue;

static FlushQueue flush_queue;
static ScePthreadMutex flush_queue_mutex;

typedef struct ThreadLoggingState
{
    char buffer[MAX_BUFFERED_BYTES];
    uint32_t buffer_idx;
    uint64_t start_time;
} ThreadLoggingState;

void flush_queue_add(ThreadLoggingState *input, FlushQueue *queue)
{
    uint32_t write_head = queue->write_head_idx;

    // add to flush queue
    FlushQueueEntry *entry = &queue->entries[write_head];
    strncpy(entry->buffer, input->buffer, input->buffer_idx);
    entry->buffer_len = input->buffer_idx;

    uint32_t next_write_head = (write_head + 1) % MAX_FLUSH_QUEUE_ENTRIES;
    if (next_write_head == queue->read_head_idx)
    {
        printf("extern_traces: flush_queue_add: full, dropping logs\n");
        return;
    }

    queue->write_head_idx = next_write_head;

    // reset input
    input->buffer_idx = 0;
    input->start_time = 0;
}

__thread ThreadLoggingState logging_state;

int sceRtcGetCurrentTick(uint64_t *tick);
int scePthreadMutexTrylock(ScePthreadMutex *mutex);
int scePthreadMutexLock(ScePthreadMutex *mutex);
int scePthreadMutexUnlock(ScePthreadMutex *mutex);
unsigned int sceKernelSleep(unsigned int seconds);
int sceKernelOpen(const char *, int, uint64_t);
long sceKernelWrite(int, const void *, unsigned long);

void extern_logf(const char *msg)
{
    int len = strlen(msg);
    strncpy(&logging_state.buffer[logging_state.buffer_idx], msg, len);
    logging_state.buffer_idx += len;

    uint64_t current_time;
    int ret = sceRtcGetCurrentTick(&current_time);
    if (ret != 0)
    {
        printf("extern_traces: sceRtcGetCurrentTick failed\n");
        return;
    }

    if (!logging_state.start_time)
    {
        logging_state.start_time = current_time;
    }

    if (logging_state.buffer_idx >= HIGH_WATER_MARK_BYTES || (current_time - logging_state.start_time) >= 1 * 1000 * 1000)
    {
        // if past the high water mark or more than 1 second since the buffer init, attempt to flush
        int ret = scePthreadMutexTrylock(&flush_queue_mutex);
        if (!ret)
        {
            // locked, add to flush queue
            flush_queue_add(&logging_state, &flush_queue);
            int ret = scePthreadMutexUnlock(&flush_queue_mutex);
            if (ret)
            {
                // error occured
                printf("extern_traces: scePThreadMutexUnlock non-ok value %x\n", ret);
            }
        }
        else
        {
            // failed to acquire lock, in this case wait for future logs
            // invocations to handle this
        }
    }
}

void *flush_thread_start_routine(void *args)
{
    printf("extern_traces: flush thread: waiting\n");

    sceKernelSleep(5);

    printf("extern_traces: flush thread: starting thread\n");

    int fd = sceKernelOpen(
        "/data/extern.log",
        0x0001 | 0x0200 | 0x0400 | 0x0080,
        0666);
    if (fd < 0)
    {
        printf("extern_traces: failed to open file /data/extern.log %x\n", fd);
        return 0;
    }

    while (1)
    {
        printf("extern_traces: flush thread: flushing\n");
        int ret = scePthreadMutexLock(&flush_queue_mutex);
        if (ret)
        {
            printf("extern_traces: flush thread: failed to acquire lock %x\n", ret);
            continue;
        }

        printf("extern_traces: flush thread: locked\n");

        for (; flush_queue.read_head_idx != flush_queue.write_head_idx; flush_queue.read_head_idx = (flush_queue.read_head_idx + 1) % MAX_FLUSH_QUEUE_ENTRIES)
        {
            FlushQueueEntry *entry = &flush_queue.entries[flush_queue.read_head_idx];

            int ret = sceKernelWrite(fd, entry->buffer, entry->buffer_len);
            if (ret < 0)
            {
                printf("extern_traces: flush thread: write failed %x\n", ret);
                continue;
            }

            printf("extern_traces: flush thread: wrote entry %d\n", flush_queue.read_head_idx);
        }

        ret = scePthreadMutexUnlock(&flush_queue_mutex);
        if (ret)
        {
            printf("extern_traces: flush thread: failed to release lock %x\n", ret);
            continue;
        }
        printf("extern_traces: flush thread: unlocked\n");

        sceKernelSleep(1);
    };
}
