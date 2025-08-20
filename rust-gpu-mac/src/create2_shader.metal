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

// Keccak-f[1600] permutation
void keccak_f(thread uint64_t state[25]) {
    uint64_t C[5], D[5], B[25];
    
    for (int round = 0; round < 24; round++) {
        // Theta
        for (int i = 0; i < 5; i++) {
            C[i] = state[i] ^ state[i + 5] ^ state[i + 10] ^ state[i + 15] ^ state[i + 20];
        }
        for (int i = 0; i < 5; i++) {
            D[i] = C[(i + 4) % 5] ^ ((C[(i + 1) % 5] << 1) | (C[(i + 1) % 5] >> 63));
        }
        for (int i = 0; i < 25; i++) {
            state[i] ^= D[i % 5];
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
        
        // Chi
        for (int j = 0; j < 25; j += 5) {
            uint64_t T[5];
            for (int i = 0; i < 5; i++) {
                T[i] = B[j + i];
            }
            for (int i = 0; i < 5; i++) {
                state[j + i] = T[i] ^ ((~T[(i + 1) % 5]) & T[(i + 2) % 5]);
            }
        }
        
        // Iota
        state[0] ^= RC[round];
    }
}

// Keccak256 hash function for thread local data
void keccak256_thread(const thread uchar* input, uint32_t input_len, thread uchar* output) {
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

// Helper function to convert hex character to value
uchar hex_to_value(uchar c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return 0;
}

// Helper function to decode hex string from device memory
void hex_decode_device(const device uchar* hex, thread uchar* bytes, uint32_t len) {
    for (uint32_t i = 0; i < len; i++) {
        bytes[i] = (hex_to_value(hex[i * 2]) << 4) | hex_to_value(hex[i * 2 + 1]);
    }
}

// Helper function to decode hex string from thread memory
void hex_decode_thread(const thread uchar* hex, thread uchar* bytes, uint32_t len) {
    for (uint32_t i = 0; i < len; i++) {
        bytes[i] = (hex_to_value(hex[i * 2]) << 4) | hex_to_value(hex[i * 2 + 1]);
    }
}

// Helper function to encode bytes to hex
void hex_encode(const thread uchar* bytes, thread uchar* hex, uint32_t len) {
    const char hex_chars[] = "0123456789abcdef";
    for (uint32_t i = 0; i < len; i++) {
        hex[i * 2] = hex_chars[bytes[i] >> 4];
        hex[i * 2 + 1] = hex_chars[bytes[i] & 0x0f];
    }
}

struct Create2Params {
    uchar implementation[40];  // hex string without 0x
    uchar deployer[40];        // hex string without 0x
    uint32_t batch_size;       // number of addresses to compute
};

struct Create2Result {
    uchar address[40];         // resulting address in hex
    uint32_t salt_index;       // which salt produced this address
};

kernel void compute_create2_batch(
    device const Create2Params* params [[buffer(0)]],
    device const uchar* salts [[buffer(1)]],  // Array of salts (32 bytes each)
    device Create2Result* results [[buffer(2)]],
    uint gid [[thread_position_in_grid]]
) {
    if (gid >= params->batch_size) return;
    
    // Constants
    const uchar PREFIX[20] = {
        0x3d, 0x60, 0x2d, 0x80, 0x60, 0x0a, 0x3d, 0x39, 0x81, 0xf3,
        0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73
    };
    const uchar SUFFIX[16] = {
        0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d,
        0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b, 0xf3, 0xff
    };
    
    // Get salt for this thread from the salts buffer
    uchar salt_str[32] = {0};
    device const uchar* salt_ptr = salts + (gid * 32);
    for (int i = 0; i < 32; i++) {
        salt_str[i] = salt_ptr[i];
    }
    
    // Build bytecode
    uchar bytecode[140];
    uchar bytecode_hex[280];
    uint32_t pos_b = 0;
    
    // Add PREFIX
    for (int i = 0; i < 20; i++) {
        bytecode[pos_b++] = PREFIX[i];
    }
    
    // Add implementation address (decode from hex)
    uchar impl_bytes[20];
    hex_decode_device(params->implementation, impl_bytes, 20);
    for (int i = 0; i < 20; i++) {
        bytecode[pos_b++] = impl_bytes[i];
    }
    
    // Add SUFFIX
    for (int i = 0; i < 16; i++) {
        bytecode[pos_b++] = SUFFIX[i];
    }
    
    // Add deployer address (decode from hex)
    uchar depl_bytes[20];
    hex_decode_device(params->deployer, depl_bytes, 20);
    for (int i = 0; i < 20; i++) {
        bytecode[pos_b++] = depl_bytes[i];
    }
    
    // Add salt (32 bytes)
    for (int i = 0; i < 32; i++) {
        bytecode[pos_b++] = salt_str[i];
    }
    
    // First hash - encode first 55 bytes to hex
    hex_encode(bytecode, bytecode_hex, 55);
    
    // Decode hex and compute first hash
    uchar first_part[55];
    hex_decode_thread(bytecode_hex, first_part, 55);
    
    uchar first_hash[32];
    keccak256_thread(first_part, 55, first_hash);
    
    // Build second part hex
    hex_encode(bytecode + 55, bytecode_hex + 110, 53);
    hex_encode(first_hash, bytecode_hex + 216, 32);
    
    // Decode hex and compute second hash
    uchar second_part[85];
    hex_decode_thread(bytecode_hex + 110, second_part, 85);
    
    uchar second_hash[32];
    keccak256_thread(second_part, 85, second_hash);
    
    // Take last 20 bytes as address
    uchar address_bytes[20];
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
    for (int i = 0; i < 40; i++) {
        results[gid].address[i] = address_hex[i];
    }
    results[gid].salt_index = gid;
}