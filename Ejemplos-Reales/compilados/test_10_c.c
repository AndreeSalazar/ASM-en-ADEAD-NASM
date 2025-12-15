#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

int main(void) {
    printf("Iniciando test hasta 10...\n");
    printf("Progreso: se imprimira cada 5\n");
    printf("\n");
    int64_t suma = 1LL;
    int64_t limite = 10LL;
    int64_t intervalo = 5LL;
    while ((suma <= limite)) {
        if (((suma % intervalo) == 0LL)) {
            printf("%ld\n", suma);
        }
        suma = (suma + 1LL);
    }
    printf("\n");
    printf("Llegamos a 10!\n");
    return 0;
}
