#include <metal_stdlib>
using namespace metal;

// Keccak256 constants
constant uint64_t RC[24] = {
    0x0000000000000001, 0x0000000000008082, 0x800000000000808a,
    0x8000000080008000, 0x000000000000808b, 0x0000000080000001,
    0x8000000080008081, 0x8000000000008009, 0x000000000000008a,
    0x0000000000000088, 0x0000000080008009, 0x000000008000000a,
    0x000000008000808b, 0x800000000000008b, 0x8000000000008089,
    0x8000000000008003, 0x8000000000008002, 0x8000000000000080,
    0x000000000000800a, 0x800000008000000a, 0x8000000080008081,
    0x8000000000008080, 0x0000000080000001, 0x8000000080008008
};

constant int r[24] = {
    1,  3,  6,  10, 15, 21, 28, 36, 45, 55, 2,  14,
    27, 41, 56, 8,  25, 43, 62, 18, 39, 61, 20, 44
};

// Optimized Keccak-f[1600] permutation with unrolled loops
void keccak_f(thread uint64_t state[25]) {
    uint64_t C[5], D[5], B[25];
    
    #pragma unroll 24
    for (int round = 0; round < 24; round++) {
        // Theta - unrolled
        C[0] = state[0] ^ state[5] ^ state[10] ^ state[15] ^ state[20];
        C[1] = state[1] ^ state[6] ^ state[11] ^ state[16] ^ state[21];
        C[2] = state[2] ^ state[7] ^ state[12] ^ state[17] ^ state[22];
        C[3] = state[3] ^ state[8] ^ state[13] ^ state[18] ^ state[23];
        C[4] = state[4] ^ state[9] ^ state[14] ^ state[19] ^ state[24];
        
        D[0] = C[4] ^ ((C[1] << 1) | (C[1] >> 63));
        D[1] = C[0] ^ ((C[2] << 1) | (C[2] >> 63));
        D[2] = C[1] ^ ((C[3] << 1) | (C[3] >> 63));
        D[3] = C[2] ^ ((C[4] << 1) | (C[4] >> 63));
        D[4] = C[3] ^ ((C[0] << 1) | (C[0] >> 63));
        
        #pragma unroll 5
        for (int i = 0; i < 25; i += 5) {
            state[i] ^= D[0];
            state[i+1] ^= D[1];
            state[i+2] ^= D[2];
            state[i+3] ^= D[3];
            state[i+4] ^= D[4];
        }
        
        // Rho and Pi
        B[0] = state[0];
        int x = 1, y = 0;
        for (int t = 0; t < 24; t++) {
            int index = x + 5 * y;
            B[y + 5 * ((2 * x + 3 * y) % 5)] = ((state[index] << r[t]) | (state[index] >> (64 - r[t])));
            int temp = x;
            x = y;
            y = (2 * temp + 3 * y) % 5;
        }
        
        // Chi - unrolled
        #pragma unroll 5
        for (int j = 0; j < 25; j += 5) {
            uint64_t T0 = B[j], T1 = B[j+1], T2 = B[j+2], T3 = B[j+3], T4 = B[j+4];
            state[j] = T0 ^ ((~T1) & T2);
            state[j+1] = T1 ^ ((~T2) & T3);
            state[j+2] = T2 ^ ((~T3) & T4);
            state[j+3] = T3 ^ ((~T4) & T0);
            state[j+4] = T4 ^ ((~T0) & T1);
        }
        
        // Iota
        state[0] ^= RC[round];
    }
}

// Optimized Keccak256 hash function with simd hints
inline void keccak256_thread(const thread uchar* input, uint32_t input_len, thread uchar* output) {
    uint64_t state[25] = {0};
    thread uchar* state_bytes = (thread uchar*)state;
    
    // Absorption phase
    uint32_t rate = 136; // For Keccak256
    uint32_t offset = 0;
    
    while (offset < input_len) {
        uint32_t block_size = min(rate, input_len - offset);
        
        for (uint32_t i = 0; i < block_size; i++) {
            state_bytes[i] ^= input[offset + i];
        }
        
        if (block_size == rate) {
            keccak_f(state);
            offset += rate;
        } else {
            break;
        }
    }
    
    // Padding
    uint32_t padding_offset = input_len % rate;
    state_bytes[padding_offset] ^= 0x01;
    state_bytes[rate - 1] ^= 0x80;
    keccak_f(state);
    
    // Squeeze phase
    for (int i = 0; i < 32; i++) {
        output[i] = state_bytes[i];
    }
}

// Optimized hex character to value conversion
inline uchar hex_to_value(uchar c) {
    return (c <= '9') ? (c - '0') : ((c & 0xDF) - 'A' + 10);
}

// Optimized hex decode with vectorization hints
inline void hex_decode_device(const device uchar* hex, thread uchar* bytes, uint32_t len) {
    #pragma unroll 4
    for (uint32_t i = 0; i < len; i++) {
        uint32_t idx = i * 2;
        bytes[i] = (hex_to_value(hex[idx]) << 4) | hex_to_value(hex[idx + 1]);
    }
}

// Helper function to decode hex string from thread memory
void hex_decode_thread(const thread uchar* hex, thread uchar* bytes, uint32_t len) {
    for (uint32_t i = 0; i < len; i++) {
        bytes[i] = (hex_to_value(hex[i * 2]) << 4) | hex_to_value(hex[i * 2 + 1]);
    }
}

// Optimized hex encode with inline conversion
inline void hex_encode(const thread uchar* bytes, thread uchar* hex, uint32_t len) {
    #pragma unroll 4
    for (uint32_t i = 0; i < len; i++) {
        uchar b = bytes[i];
        uint32_t idx = i * 2;
        uchar high = b >> 4;
        uchar low = b & 0x0f;
        hex[idx] = (high < 10) ? ('0' + high) : ('a' + high - 10);
        hex[idx + 1] = (low < 10) ? ('0' + low) : ('a' + low - 10);
    }
}

// PCG32 Random Number Generator for GPU
// Simple, fast, and good quality random numbers
struct PCGState {
    uint64_t state;
    uint64_t inc;
};

inline uint32_t pcg32_random(thread PCGState* rng) {
    uint64_t oldstate = rng->state;
    rng->state = oldstate * 6364136223846793005ULL + rng->inc;
    uint32_t xorshifted = ((oldstate >> 18u) ^ oldstate) >> 27u;
    uint32_t rot = oldstate >> 59u;
    return (xorshifted >> rot) | (xorshifted << ((-rot) & 31));
}

inline void pcg32_init(thread PCGState* rng, uint64_t seed, uint64_t stream) {
    rng->state = 0U;
    rng->inc = (stream << 1u) | 1u;
    pcg32_random(rng);
    rng->state += seed;
    pcg32_random(rng);
}

// Generate random hex string using GPU RNG
inline void generate_random_salt(thread PCGState* rng, thread uchar* salt) {
    const uchar hex_chars[16] = {'0','1','2','3','4','5','6','7','8','9','a','b','c','d','e','f'};
    
    // Generate 32 hex characters (16 bytes)
    for (int i = 0; i < 8; i++) {
        uint32_t rand = pcg32_random(rng);
        // Extract 4 bytes from the random number
        salt[i*4] = hex_chars[(rand >> 28) & 0xF];
        salt[i*4+1] = hex_chars[(rand >> 24) & 0xF];
        salt[i*4+2] = hex_chars[(rand >> 20) & 0xF];
        salt[i*4+3] = hex_chars[(rand >> 16) & 0xF];
    }
}

struct Create2Params {
    uchar implementation[40];  // hex string without 0x
    uchar deployer[40];        // hex string without 0x
    uint32_t batch_size;       // number of addresses to compute
    uint32_t addresses_per_thread; // number of addresses each thread processes
    uint32_t random_seed;      // seed for GPU random number generation
    uint32_t use_gpu_random;   // 1 to use GPU random, 0 to use provided salts
};

struct Create2Result {
    uchar address[40];         // resulting address in hex
    uint32_t salt_index;       // which salt produced this address
};

kernel void compute_create2_batch(
    device const Create2Params* params [[buffer(0)]],
    device const uchar* salts [[buffer(1)]],  // Array of salts (32 bytes each)
    device Create2Result* results [[buffer(2)]],
    uint gid [[thread_position_in_grid]],
    uint tid [[thread_index_in_threadgroup]]
) {
    // Thread coarsening: each thread processes multiple addresses
    uint32_t addresses_per_thread = params->addresses_per_thread;
    uint32_t start_idx = gid * addresses_per_thread;
    uint32_t end_idx = min(start_idx + addresses_per_thread, params->batch_size);
    
    if (start_idx >= params->batch_size) return;
    
    // Initialize GPU RNG if needed
    PCGState rng;
    if (params->use_gpu_random == 1) {
        // Use global thread ID and seed to create unique RNG per thread
        uint64_t unique_seed = params->random_seed + gid;
        uint64_t stream = (uint64_t)tid * 1099511628211ULL; // Large prime for stream separation
        pcg32_init(&rng, unique_seed, stream);
    }
    
    // Constants - shared across all iterations
    const uchar PREFIX[20] = {
        0x3d, 0x60, 0x2d, 0x80, 0x60, 0x0a, 0x3d, 0x39, 0x81, 0xf3,
        0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73
    };
    const uchar SUFFIX[16] = {
        0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d,
        0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b, 0xf3, 0xff
    };
    
    // Pre-decode addresses once (reuse across iterations)
    uchar impl_bytes[20];
    hex_decode_device(params->implementation, impl_bytes, 20);
    
    uchar depl_bytes[20];
    hex_decode_device(params->deployer, depl_bytes, 20);
    
    // Pre-build common bytecode parts
    uchar bytecode_template[76];  // Without salt: 20 + 20 + 16 + 20 = 76
    uint32_t pos = 0;
    
    // Add PREFIX
    for (int i = 0; i < 20; i++) {
        bytecode_template[pos++] = PREFIX[i];
    }
    
    // Add implementation
    for (int i = 0; i < 20; i++) {
        bytecode_template[pos++] = impl_bytes[i];
    }
    
    // Add SUFFIX
    for (int i = 0; i < 16; i++) {
        bytecode_template[pos++] = SUFFIX[i];
    }
    
    // Add deployer
    for (int i = 0; i < 20; i++) {
        bytecode_template[pos++] = depl_bytes[i];
    }
    
    // Process multiple addresses per thread
    for (uint32_t idx = start_idx; idx < end_idx; idx++) {
        // Get salt for this iteration
        uchar salt_str[32];
        
        if (params->use_gpu_random == 1) {
            // Generate random salt on GPU
            generate_random_salt(&rng, salt_str);
        } else {
            // Use provided salt
            device const uchar* salt_ptr = salts + (idx * 32);
            
            // Vectorized salt copy
            #pragma unroll 8
            for (int i = 0; i < 32; i++) {
                salt_str[i] = salt_ptr[i];
            }
        }
        
        // Build complete bytecode by adding salt to template
        uchar bytecode[108];
        
        // Copy template
        #pragma unroll 8
        for (int i = 0; i < 76; i++) {
            bytecode[i] = bytecode_template[i];
        }
        
        // Add salt
        #pragma unroll 8
        for (int i = 0; i < 32; i++) {
            bytecode[76 + i] = salt_str[i];
        }
        
        // First hash - compute directly from bytecode (first 55 bytes)
        uchar first_hash[32];
        keccak256_thread(bytecode, 55, first_hash);
        
        // Build second part for hashing
        uchar second_part[85];
        
        // Copy remaining bytecode
        #pragma unroll 8
        for (int i = 0; i < 53; i++) {
            second_part[i] = bytecode[55 + i];
        }
        
        // Add first hash
        #pragma unroll 8
        for (int i = 0; i < 32; i++) {
            second_part[53 + i] = first_hash[i];
        }
        
        uchar second_hash[32];
        keccak256_thread(second_part, 85, second_hash);
        
        // Take last 20 bytes as address
        uchar address_bytes[20];
        #pragma unroll 4
        for (int i = 0; i < 20; i++) {
            address_bytes[i] = second_hash[12 + i];
        }
        
        // Convert to checksum address
        uchar address_hex[40];
        hex_encode(address_bytes, address_hex, 20);
        
        // Compute checksum
        uchar address_hash[32];
        keccak256_thread(address_hex, 40, address_hash);
        
        // Apply checksum
        #pragma unroll 8
        for (int i = 0; i < 40; i++) {
            uchar c = address_hex[i];
            if (c >= 'a' && c <= 'f') {
                uint32_t byte_index = i / 2;
                uint32_t nibble_index = i % 2;
                uchar byte_value = address_hash[byte_index];
                uchar nibble_value = (nibble_index == 0) ? (byte_value >> 4) : (byte_value & 0x0f);
                
                if (nibble_value >= 8) {
                    address_hex[i] = c - 32; // Convert to uppercase
                }
            }
        }
        
        // Store result
        #pragma unroll 8
        for (int i = 0; i < 40; i++) {
            results[idx].address[i] = address_hex[i];
        }
        results[idx].salt_index = idx;
    }
}