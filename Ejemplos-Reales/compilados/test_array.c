#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

// Estructura Array dinámica
typedef struct {
    int64_t* data;
    size_t length;
    size_t capacity;
} Array;

// Crear array vacío
Array array_new(void) {
    Array arr;
    arr.length = 0;
    arr.capacity = 4;
    arr.data = (int64_t*)malloc(arr.capacity * sizeof(int64_t));
    return arr;
}

// Crear array desde valores iniciales
Array array_from_values(size_t count, int64_t* values) {
    Array arr;
    arr.length = count;
    arr.capacity = count > 4 ? count * 2 : 4;
    arr.data = (int64_t*)malloc(arr.capacity * sizeof(int64_t));
    memcpy(arr.data, values, count * sizeof(int64_t));
    return arr;
}

// Agregar elemento al array
void array_append(Array* arr, int64_t value) {
    if (arr->length >= arr->capacity) {
        arr->capacity *= 2;
        arr->data = (int64_t*)realloc(arr->data, arr->capacity * sizeof(int64_t));
    }
    arr->data[arr->length++] = value;
}

// Obtener elemento por índice
int64_t array_get(Array* arr, size_t index) {
    if (index >= arr->length) {
        fprintf(stderr, "Error: índice fuera de rango\n");
        exit(1);
    }
    return arr->data[index];
}

// Establecer elemento por índice
void array_set(Array* arr, size_t index, int64_t value) {
    if (index >= arr->length) {
        fprintf(stderr, "Error: índice fuera de rango\n");
        exit(1);
    }
    arr->data[index] = value;
}

// Obtener longitud del array
size_t array_len(Array* arr) {
    return arr->length;
}

int main(void) {
    int64_t _init_arr_0[] = { 1LL, 2LL, 3LL };
    Array arr = array_from_values(3, _init_arr_0);
    printf("%ld\n", array_get(&arr, (size_t)(0LL))); fflush(stdout);
    printf("%ld\n", array_get(&arr, (size_t)(1LL))); fflush(stdout);
    printf("%ld\n", array_get(&arr, (size_t)(2LL))); fflush(stdout);
    return 0;
}
