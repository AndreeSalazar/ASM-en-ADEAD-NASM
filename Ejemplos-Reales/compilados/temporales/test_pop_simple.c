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

// Eliminar y retornar último elemento (pop)
int64_t array_pop(Array* arr) {
    if (arr->length == 0) {
        fprintf(stderr, "Error: pop de array vacío\n");
        exit(1);
    }
    return arr->data[--arr->length];
}

// Eliminar y retornar elemento en índice específico
int64_t array_pop_at(Array* arr, size_t index) {
    if (index >= arr->length) {
        fprintf(stderr, "Error: índice fuera de rango\n");
        exit(1);
    }
    int64_t value = arr->data[index];
    // Mover elementos hacia la izquierda
    for (size_t i = index; i < arr->length - 1; i++) {
        arr->data[i] = arr->data[i + 1];
    }
    arr->length--;
    return value;
}

// Insertar elemento en posición específica
void array_insert(Array* arr, size_t index, int64_t value) {
    if (index > arr->length) {
        fprintf(stderr, "Error: índice fuera de rango\n");
        exit(1);
    }
    // Redimensionar si es necesario
    if (arr->length >= arr->capacity) {
        arr->capacity *= 2;
        arr->data = (int64_t*)realloc(arr->data, arr->capacity * sizeof(int64_t));
    }
    // Mover elementos hacia la derecha
    for (size_t i = arr->length; i > index; i--) {
        arr->data[i] = arr->data[i - 1];
    }
    arr->data[index] = value;
    arr->length++;
}

// Eliminar primera ocurrencia de valor
void array_remove(Array* arr, int64_t value) {
    for (size_t i = 0; i < arr->length; i++) {
        if (arr->data[i] == value) {
            // Mover elementos hacia la izquierda
            for (size_t j = i; j < arr->length - 1; j++) {
                arr->data[j] = arr->data[j + 1];
            }
            arr->length--;
            return;
        }
    }
    fprintf(stderr, "Error: valor no encontrado en array\n");
    exit(1);
}

// Encontrar índice de valor
size_t array_index(Array* arr, int64_t value) {
    for (size_t i = 0; i < arr->length; i++) {
        if (arr->data[i] == value) {
            return i;
        }
    }
    fprintf(stderr, "Error: valor no encontrado en array\n");
    exit(1);
}

// Contar ocurrencias de valor
size_t array_count(Array* arr, int64_t value) {
    size_t count = 0;
    for (size_t i = 0; i < arr->length; i++) {
        if (arr->data[i] == value) {
            count++;
        }
    }
    return count;
}

// Ordenar array (bubble sort simple)
void array_sort(Array* arr) {
    for (size_t i = 0; i < arr->length; i++) {
        for (size_t j = 0; j < arr->length - i - 1; j++) {
            if (arr->data[j] > arr->data[j + 1]) {
                int64_t temp = arr->data[j];
                arr->data[j] = arr->data[j + 1];
                arr->data[j + 1] = temp;
            }
        }
    }
}

// Invertir orden del array
void array_reverse(Array* arr) {
    for (size_t i = 0; i < arr->length / 2; i++) {
        int64_t temp = arr->data[i];
        arr->data[i] = arr->data[arr->length - 1 - i];
        arr->data[arr->length - 1 - i] = temp;
    }
}

int main(void) {
    int64_t _init_arr_0[] = { 1LL, 2LL, 3LL };
    Array arr = array_from_values(3, _init_arr_0);
    array_append(&arr, 4LL);
    int64_t last = array_pop(&arr);
    printf("%ld\n", last); fflush(stdout);
    return 0;
}
