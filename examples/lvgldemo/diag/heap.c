#include <execinfo.h>
#include <stdlib.h>

int get_backtrace(char ***out_frames)
{
    void *array[0x200];
    int size = backtrace(array, 0x200);
    char **strings = backtrace_symbols(array, size);
    if (!strings)
        return 0;

    *out_frames = strings;
    return size;
}
