#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct {
    void* data;
    size_t length;
    size_t capacity;
} Array;

extern Array array_new(void);

int main(void) {
    printf("Test: Array functions\n");
    Array arr = array_new();
    printf("Array created successfully\n");
    return 0;
}
