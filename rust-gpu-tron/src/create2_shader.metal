#include <metal_stdlib>
using namespace metal;

// ==================== SHA256 Implementation ====================
constant uint32_t K256[64] = {
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
};

inline uint32_t rotr(uint32_t x, uint32_t n) {
    return (x >> n) | (x << (32 - n));
}

inline uint32_t ch(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) ^ (~x & z);
}

inline uint32_t maj(uint32_t x, uint32_t y, uint32_t z) {
    return (x & y) ^ (x & z) ^ (y & z);
}

inline uint32_t sigma0(uint32_t x) {
    return rotr(x, 2) ^ rotr(x, 13) ^ rotr(x, 22);
}

inline uint32_t sigma1(uint32_t x) {
    return rotr(x, 6) ^ rotr(x, 11) ^ rotr(x, 25);
}

inline uint32_t gamma0(uint32_t x) {
    return rotr(x, 7) ^ rotr(x, 18) ^ (x >> 3);
}

inline uint32_t gamma1(uint32_t x) {
    return rotr(x, 17) ^ rotr(x, 19) ^ (x >> 10);
}

void sha256(const thread uchar* input, uint32_t input_len, thread uchar* output) {
    uint32_t h[8] = {
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
    };
    
    uint32_t total_len = input_len + 1 + 8;
    uint32_t padding_len = (64 - ((input_len + 9) % 64)) % 64;
    total_len += padding_len;
    
    for (uint32_t chunk_start = 0; chunk_start < total_len; chunk_start += 64) {
        uint32_t w[64];
        
        for (int i = 0; i < 16; i++) {
            w[i] = 0;
            for (int j = 0; j < 4; j++) {
                uint32_t byte_idx = chunk_start + i * 4 + j;
                uchar byte_val = 0;
                
                if (byte_idx < input_len) {
                    byte_val = input[byte_idx];
                } else if (byte_idx == input_len) {
                    byte_val = 0x80;
                } else if (byte_idx >= total_len - 8) {
                    uint64_t bit_len = (uint64_t)input_len * 8;
                    int len_byte_idx = byte_idx - (total_len - 8);
                    byte_val = (bit_len >> (56 - len_byte_idx * 8)) & 0xff;
                }
                
                w[i] |= ((uint32_t)byte_val) << (24 - j * 8);
            }
        }
        
        for (int i = 16; i < 64; i++) {
            w[i] = gamma1(w[i-2]) + w[i-7] + gamma0(w[i-15]) + w[i-16];
        }
        
        uint32_t a = h[0], b = h[1], c = h[2], d = h[3];
        uint32_t e = h[4], f = h[5], g = h[6], hh = h[7];
        
        for (int i = 0; i < 64; i++) {
            uint32_t t1 = hh + sigma1(e) + ch(e, f, g) + K256[i] + w[i];
            uint32_t t2 = sigma0(a) + maj(a, b, c);
            hh = g; g = f; f = e; e = d + t1;
            d = c; c = b; b = a; a = t1 + t2;
        }
        
        h[0] += a; h[1] += b; h[2] += c; h[3] += d;
        h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
    }
    
    for (int i = 0; i < 8; i++) {
        output[i*4] = (h[i] >> 24) & 0xff;
        output[i*4+1] = (h[i] >> 16) & 0xff;
        output[i*4+2] = (h[i] >> 8) & 0xff;
        output[i*4+3] = h[i] & 0xff;
    }
}

// ==================== Keccak256 Implementation ====================
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

void keccak_f(thread uint64_t state[25]) {
    uint64_t C[5], D[5], B[25];
    
    #pragma unroll 24
    for (int round = 0; round < 24; round++) {
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
        
        B[0] = state[0];
        int x = 1, y = 0;
        for (int t = 0; t < 24; t++) {
            int index = x + 5 * y;
            B[y + 5 * ((2 * x + 3 * y) % 5)] = ((state[index] << r[t]) | (state[index] >> (64 - r[t])));
            int temp = x;
            x = y;
            y = (2 * temp + 3 * y) % 5;
        }
        
        #pragma unroll 5
        for (int j = 0; j < 25; j += 5) {
            uint64_t T0 = B[j], T1 = B[j+1], T2 = B[j+2], T3 = B[j+3], T4 = B[j+4];
            state[j] = T0 ^ ((~T1) & T2);
            state[j+1] = T1 ^ ((~T2) & T3);
            state[j+2] = T2 ^ ((~T3) & T4);
            state[j+3] = T3 ^ ((~T4) & T0);
            state[j+4] = T4 ^ ((~T0) & T1);
        }
        
        state[0] ^= RC[round];
    }
}

void keccak256_thread(const thread uchar* input, uint32_t input_len, thread uchar* output) {
    uint64_t state[25] = {0};
    thread uchar* state_bytes = (thread uchar*)state;
    
    uint32_t rate = 136;
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
    
    uint32_t padding_offset = input_len % rate;
    state_bytes[padding_offset] ^= 0x01;
    state_bytes[rate - 1] ^= 0x80;
    keccak_f(state);
    
    for (int i = 0; i < 32; i++) {
        output[i] = state_bytes[i];
    }
}

// ==================== Base58 Encoding ====================
constant uchar BASE58_ALPHABET[58] = {
    '1','2','3','4','5','6','7','8','9',
    'A','B','C','D','E','F','G','H','J','K','L','M','N','P','Q','R','S','T','U','V','W','X','Y','Z',
    'a','b','c','d','e','f','g','h','i','j','k','m','n','o','p','q','r','s','t','u','v','w','x','y','z'
};

uint32_t base58_encode(const thread uchar* input, uint32_t input_len, device uchar* output) {
    uchar temp[50] = {0};
    uint32_t temp_len = 0;
    
    for (uint32_t i = 0; i < input_len; i++) {
        temp[i] = input[i];
    }
    temp_len = input_len;
    
    uchar result[40];
    uint32_t result_pos = 0;
    
    while (temp_len > 0) {
        uint32_t remainder = 0;
        uint32_t new_len = 0;
        
        for (uint32_t i = 0; i < temp_len; i++) {
            uint32_t value = remainder * 256 + temp[i];
            uint32_t quotient = value / 58;
            remainder = value % 58;
            
            if (quotient > 0 || new_len > 0) {
                temp[new_len++] = quotient;
            }
        }
        
        result[result_pos++] = BASE58_ALPHABET[remainder];
        temp_len = new_len;
    }
    
    for (uint32_t i = 0; i < input_len && input[i] == 0; i++) {
        result[result_pos++] = '1';
    }
    
    uint32_t output_len = result_pos;
    for (uint32_t i = 0; i < output_len; i++) {
        output[i] = result[output_len - 1 - i];
    }
    
    return output_len;
}

// ==================== Utility Functions ====================
inline uchar hex_to_value(uchar c) {
    return (c <= '9') ? (c - '0') : ((c & 0xDF) - 'A' + 10);
}

inline void hex_decode_device(const device uchar* hex, thread uchar* bytes, uint32_t len) {
    #pragma unroll 4
    for (uint32_t i = 0; i < len; i++) {
        uint32_t idx = i * 2;
        bytes[i] = (hex_to_value(hex[idx]) << 4) | hex_to_value(hex[idx + 1]);
    }
}

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

// ==================== Random Number Generation ====================
struct PCGState {
    uint64_t state;
    uint64_t inc;
};

uint32_t pcg32_random(thread PCGState* rng) {
    uint64_t oldstate = rng->state;
    rng->state = oldstate * 6364136223846793005ULL + rng->inc;
    uint32_t xorshifted = ((oldstate >> 18u) ^ oldstate) >> 27u;
    uint32_t rot = oldstate >> 59u;
    return (xorshifted >> rot) | (xorshifted << ((-rot) & 31));
}

void pcg32_init(thread PCGState* rng, uint64_t seed, uint64_t stream) {
    rng->state = 0U;
    rng->inc = (stream << 1u) | 1u;
    pcg32_random(rng);
    rng->state += seed;
    pcg32_random(rng);
}

void generate_random_salt(thread PCGState* rng, thread uchar* salt) {
    const uchar hex_chars[16] = {'0','1','2','3','4','5','6','7','8','9','a','b','c','d','e','f'};
    
    for (int i = 0; i < 8; i++) {
        uint32_t rand = pcg32_random(rng);
        salt[i*4] = hex_chars[(rand >> 28) & 0xF];
        salt[i*4+1] = hex_chars[(rand >> 24) & 0xF];
        salt[i*4+2] = hex_chars[(rand >> 20) & 0xF];
        salt[i*4+3] = hex_chars[(rand >> 16) & 0xF];
    }
}

// ==================== Create2 Parameters ====================
struct Create2TronParams {
    uchar implementation[40];  // Hex address without 0x
    uchar deployer[40];        // Hex address without 0x
    uint32_t batch_size;
    uint32_t addresses_per_thread;
    uint32_t random_seed;
    uint32_t use_gpu_random;
};

struct Create2TronResult {
    uchar address[64];         // Base58 encoded Tron address
    uint32_t salt_index;
    uint32_t address_len;
};

// ==================== Main Kernel ====================
kernel void compute_create2_tron_batch(
    device const Create2TronParams* params [[buffer(0)]],
    device const uchar* salts [[buffer(1)]],
    device Create2TronResult* results [[buffer(2)]],
    uint gid [[thread_position_in_grid]],
    uint tid [[thread_index_in_threadgroup]]
) {
    uint32_t addresses_per_thread = params->addresses_per_thread;
    uint32_t start_idx = gid * addresses_per_thread;
    uint32_t end_idx = min(start_idx + addresses_per_thread, params->batch_size);
    
    if (start_idx >= params->batch_size) return;
    
    PCGState rng;
    if (params->use_gpu_random == 1) {
        uint64_t unique_seed = params->random_seed + gid;
        uint64_t stream = (uint64_t)tid * 1099511628211ULL;
        pcg32_init(&rng, unique_seed, stream);
    }
    
    // Decode hex addresses to bytes
    uchar impl_bytes[20];
    uchar depl_bytes[20];
    hex_decode_device(params->implementation, impl_bytes, 20);
    hex_decode_device(params->deployer, depl_bytes, 20);
    
    // Constants
    const uchar PREFIX[20] = {
        0x3d, 0x60, 0x2d, 0x80, 0x60, 0x0a, 0x3d, 0x39, 0x81, 0xf3,
        0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73
    };
    const uchar TRON_SUFFIX[16] = {
        0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d,
        0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b, 0xf3, 0x41
    };
    
    // Build bytecode template
    uchar bytecode_template[76];
    uint32_t pos = 0;
    
    for (int i = 0; i < 20; i++) bytecode_template[pos++] = PREFIX[i];
    for (int i = 0; i < 20; i++) bytecode_template[pos++] = impl_bytes[i];
    for (int i = 0; i < 16; i++) bytecode_template[pos++] = TRON_SUFFIX[i];
    for (int i = 0; i < 20; i++) bytecode_template[pos++] = depl_bytes[i];
    
    // Process addresses
    for (uint32_t idx = start_idx; idx < end_idx; idx++) {
        uchar salt_str[32];
        
        if (params->use_gpu_random == 1) {
            generate_random_salt(&rng, salt_str);
        } else {
            device const uchar* salt_ptr = salts + (idx * 32);
            for (int i = 0; i < 32; i++) {
                salt_str[i] = salt_ptr[i];
            }
        }
        
        // Build complete bytecode
        uchar bytecode[108];
        for (int i = 0; i < 76; i++) bytecode[i] = bytecode_template[i];
        for (int i = 0; i < 32; i++) bytecode[76 + i] = salt_str[i];
        
        // First Keccak256 hash
        uchar first_hash[32];
        keccak256_thread(bytecode, 55, first_hash);
        
        // Build second part
        uchar second_part[85];
        for (int i = 0; i < 53; i++) second_part[i] = bytecode[55 + i];
        for (int i = 0; i < 32; i++) second_part[53 + i] = first_hash[i];
        
        // Second Keccak256 hash
        uchar second_hash[32];
        keccak256_thread(second_part, 85, second_hash);
        
        // Take last 20 bytes as address
        uchar address_bytes[20];
        for (int i = 0; i < 20; i++) {
            address_bytes[i] = second_hash[12 + i];
        }
        
        // Create Tron address with 0x41 prefix
        uchar tron_addr_bytes[25];
        tron_addr_bytes[0] = 0x41;
        for (int i = 0; i < 20; i++) {
            tron_addr_bytes[1 + i] = address_bytes[i];
        }
        
        // Calculate checksum using double SHA256
        uchar hash1[32];
        sha256(tron_addr_bytes, 21, hash1);
        
        uchar hash2[32];
        sha256(hash1, 32, hash2);
        
        // Add checksum
        for (int i = 0; i < 4; i++) {
            tron_addr_bytes[21 + i] = hash2[i];
        }
        
        // Encode to Base58
        uint32_t addr_len = base58_encode(tron_addr_bytes, 25, results[idx].address);
        results[idx].address_len = addr_len;
        results[idx].salt_index = idx;
    }
}