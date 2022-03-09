#include "api.h"
#include "syscall.h"
#include <sys/stat.h>
#include <errno.h>
#undef errno
extern int errno;

char *__env[1] = { 0 };
char **environ = __env;

void _exit(int status) {
}

int _close(int fd) {
    return -1;
}

int _execve(char *name, char **argv, char **env) {
    errno = ENOMEM;
    return -1;
}

int _fork(void) {
    errno = EAGAIN;
    return -1;
}

int _fstat(int file, struct stat *st) {
    st->st_mode = 1;
    return 0;
}

int _getpid(void) {
    return 1;
}

int _isatty(int file) {
    return 1;
}

int _kill(int pid, int sig) {
    errno = EINVAL;
    return -1;
}

int _link(char *old, char *new) {
    errno = EMLINK;
    return -1;
}

int _lseek(int file, int ptr, int dir) {
    return 0;
}

int _open(const char *name, int flags, int mode) {
    return -1;
}

int _read(int file, char *ptr, int len) {
    return 0;
}

caddr_t _sbrk(int incr) {
    caddr_t stack_ptr = (caddr_t) &incr;
    extern char _end;		/* Defined by the linker */

    static char *heap_limit;
    static char *heap_end = &_end;
    char *prev_heap_end = heap_end;
    if (heap_end + incr > stack_ptr) {
        _write(1, "Heap and stack collision\n", 25);
    }

    if (heap_limit == 0) {
        _init_heap(&heap_limit);
    }
    if (heap_end + incr > heap_limit) {
        return (caddr_t) -1;
    }

    heap_end += incr;
    return (caddr_t) prev_heap_end;
}

void _init_heap(char** heap_limit) {
    call_syscall(SYS_BRK, heap_limit);
}

int _stat(const char *file, struct stat *st) {
    st->st_mode = S_IFCHR;
    return 0;
}

int _times(struct tms *buf) {
    return -1;
}

int _unlink(char *name) {
    errno = ENOENT;
    return -1;
}

int _wait(int *status) {
    errno = ECHILD;
    return -1;
}

int _write(int file, char *buf, int len) {
    call_syscall(SYS_WRITE, file, buf, len);
    return len;
}
