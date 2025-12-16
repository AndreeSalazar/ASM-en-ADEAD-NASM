#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

int main(void) {
    printf("Iniciando suma hasta 100,000...\n"); fflush(stdout);
    printf("Progreso: se imprimira cada 10,000\n"); fflush(stdout);
    printf("\n"); fflush(stdout);
    int64_t suma = 0LL;
    int64_t limite = 100000LL;
    int64_t intervalo = 10000LL;
    while ((suma <= limite)) {
        if (((suma % intervalo) == (0 && suma > 0LL))) {
            printf("%ld\n", suma); fflush(stdout);
        }
        suma = (suma + 100LL);
    }
    printf("\n"); fflush(stdout);
    printf("Llegamos a 100,000!\n"); fflush(stdout);
    return 0;
}
