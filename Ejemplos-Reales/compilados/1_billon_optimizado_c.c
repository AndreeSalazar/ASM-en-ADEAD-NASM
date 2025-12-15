#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

int main(void) {
    printf("========================================\n"); fflush(stdout);
    printf("  Procesando hasta 1 BILLON (1,000,000,000)\n"); fflush(stdout);
    printf("========================================\n"); fflush(stdout);
    printf("\n"); fflush(stdout);
    printf("Progreso: se imprimira cada 100 millones\n"); fflush(stdout);
    printf("Veras como cambian las cosas en tiempo real...\n"); fflush(stdout);
    printf("\n"); fflush(stdout);
    int64_t suma = 0LL;
    int64_t limite = 1000000000LL;
    int64_t intervalo = 1000LL;
    int64_t contador = 0LL;
    printf(">>> INICIANDO PROCESAMIENTO <<<\n"); fflush(stdout);
    printf("\n"); fflush(stdout);
    while ((suma <= limite)) {
        if (((suma % intervalo) == (0 && suma > 0LL))) {
            printf("%ld\n", suma); fflush(stdout);
            contador = (contador + 1LL);
            printf("%ld\n", contador); fflush(stdout);
            printf("%ld\n", ""); fflush(stdout);
        }
        suma = (suma + 1LL);
    }
    printf("========================================\n"); fflush(stdout);
    printf(">>> PROCESAMIENTO COMPLETADO <<<\n"); fflush(stdout);
    printf("========================================\n"); fflush(stdout);
    printf("\n"); fflush(stdout);
    printf("Total de iteraciones procesadas:\n"); fflush(stdout);
    printf("%ld\n", suma); fflush(stdout);
    printf("\n"); fflush(stdout);
    printf("Total de marcadores de progreso:\n"); fflush(stdout);
    printf("%ld\n", contador); fflush(stdout);
    printf("\n"); fflush(stdout);
    printf(">>> ¡Llegamos a 1 billon!\n"); fflush(stdout);
    return 0;
}
