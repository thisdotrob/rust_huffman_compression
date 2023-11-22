pub struct HuffmanTable {
    // the compressed values that will be written for each uncompressed byte.
    // the index in the array is the original byte
    // e.g. if uncompressed byte is 0x01 -> index 1 -> 0x01F (11111)
    pub values: [u32; 256],

    // the number of bits needed to write each compressed value.
    // the index in the array is the original byte
    // e.g. if uncompressed byte is 0x01 -> index 1 -> 0x5 (5) bits needed
    pub bit_counts: [u8; 256],
}

impl HuffmanTable {
    pub fn get_compressed_value(&self, uncompressed_byte: u8) -> u32 {
        self.values[uncompressed_byte as usize]
    }

    pub fn get_compressed_value_bit_count(&self, uncompressed_byte: u8) -> u8 {
        self.bit_counts[uncompressed_byte as usize]
    }
}
