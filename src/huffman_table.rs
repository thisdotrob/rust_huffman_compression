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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_the_compressed_value_at_the_index_of_the_byte_provided() {
        let mut values: [u32; 256] = [0; 256];
        let mut bit_counts: [u8; 256] = [0; 256];

        values[0x33] = 0xFFF;
        bit_counts[0x33] = 12;

        let huffman_table = HuffmanTable {
            values,
            bit_counts,
        };

        let compressed_value = huffman_table.get_compressed_value(0x33);
        assert_eq!(compressed_value, 0xFFF);
    }

    #[test]
    fn it_returns_the_bit_count_of_the_compressed_value_at_the_index_of_the_byte_provided() {
        let mut values: [u32; 256] = [0; 256];
        let mut bit_counts: [u8; 256] = [0; 256];

        values[0x33] = 0xFFF;
        bit_counts[0x33] = 12;

        let huffman_table = HuffmanTable {
            values,
            bit_counts,
        };

        let compressed_value = huffman_table.get_compressed_value_bit_count(0x33);
        assert_eq!(compressed_value, 12);
    }

}
