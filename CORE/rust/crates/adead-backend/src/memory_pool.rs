// Memory Pool para optimizar alocaciones pequeñas
// Estrategia: Pool de bloques pre-allocados para arrays pequeños (< 16 elementos)

pub struct MemoryPool {
    small_arrays: Vec<usize>,  // Pools para arrays pequeños (4, 8, 16 elementos)
    pool_size: usize,           // Tamaño de cada pool
}

impl MemoryPool {
    pub fn new() -> Self {
        Self {
            small_arrays: vec![0; 3],  // 3 pools: 4, 8, 16 elementos
            pool_size: 10,              // 10 bloques por pool
        }
    }

    /// Determinar si un array es "pequeño" y puede usar pooling
    pub fn is_small_array(capacity: usize) -> bool {
        capacity <= 16
    }

    /// Obtener índice del pool basado en capacity
    pub fn get_pool_index(capacity: usize) -> Option<usize> {
        match capacity {
            1..=4 => Some(0),   // Pool para arrays de 4 elementos
            5..=8 => Some(1),   // Pool para arrays de 8 elementos
            9..=16 => Some(2),  // Pool para arrays de 16 elementos
            _ => None,          // Arrays grandes no usan pooling
        }
    }

    /// Calcular capacity del pool (redondeado hacia arriba)
    pub fn get_pool_capacity(requested: usize) -> usize {
        if requested <= 4 {
            4
        } else if requested <= 8 {
            8
        } else if requested <= 16 {
            16
        } else {
            requested  // No pooling para arrays grandes
        }
    }
}

