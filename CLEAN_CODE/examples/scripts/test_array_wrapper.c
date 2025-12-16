#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

// Declaraciones de funciones del ASM
typedef struct {
    void* data;
    size_t length;
    size_t capacity;
} Array;

extern Array array_new(void);
extern Array array_from_values(size_t count, int64_t* values);
extern void array_append(Array* arr, int64_t value);
extern int64_t array_get(Array* arr, size_t index);
extern void array_set(Array* arr, size_t index, int64_t value);
extern size_t array_len(Array* arr);

int main(void) {
    printf("=== Test Array Functions ===\n");
    
    // Test 1: Crear array desde valores
    int64_t vals[] = {1, 2, 3};
    Array arr = array_from_values(3, vals);
    
    printf("Array length: %zu\n", array_len(&arr));
    printf("Array[0]: %lld\n", array_get(&arr, 0));
    printf("Array[1]: %lld\n", array_get(&arr, 1));
    printf("Array[2]: %lld\n", array_get(&arr, 2));
    
    printf("\n=== Test completado ===\n");
    return 0;
}
