#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

typedef struct {
    int64_t* data;
    size_t length;
} Array;

Array array_from_values(size_t count, int64_t* values) {
    Array arr;
    arr.length = count;
    arr.data = (int64_t*)malloc(count * sizeof(int64_t));
    for (size_t i = 0; i < count; i++) {
        arr.data[i] = values[i];
    }
    return arr;
}

int64_t array_get(Array* arr, size_t index) {
    return arr->data[index];
}

int main(void) {
    int64_t vals[] = { 1LL, 2LL, 3LL };
    Array arr = array_from_values(3, vals);
    printf("%ld\n", array_get(&arr, 0LL));
    printf("%ld\n", array_get(&arr, 1LL));
    printf("%ld\n", array_get(&arr, 2LL));
    return 0;
}

