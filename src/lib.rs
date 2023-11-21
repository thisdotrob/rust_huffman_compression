pub struct TerminalCode {
    pub bit_count: u8,
    pub value: u32,
}

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

pub struct Huffman {
    pub table: HuffmanTable,
    pub terminal_code: TerminalCode,
}

impl Huffman {
    pub fn new(terminal_code: TerminalCode, table: HuffmanTable) -> Huffman {
        return Huffman { terminal_code, table }
    }

    pub fn compress(&self, src: Vec<u8>, output: &mut Vec<u8>) {
        let mut to_write: u32 = 0;

        let mut to_write_bit_count: u8 = 0;

        for byte in src {
             // What does casting `byte` to usize as below do performance wise?
            let compressed_value = self.table.values[byte as usize];

            let compressed_value_bit_count = self.table.bit_counts[byte as usize];

            to_write = to_write << compressed_value_bit_count;

            to_write = to_write | compressed_value;

            to_write_bit_count = to_write_bit_count + compressed_value_bit_count;

            while to_write_bit_count >= 8 {
                to_write_bit_count = to_write_bit_count - 8;

                let output_byte = to_write >> to_write_bit_count;

                let mask = u32::MAX >> (32 - to_write_bit_count);

                to_write = to_write & mask;

                output.push(output_byte as u8);
            }
        }

        let compressed_value = self.terminal_code.value;

        let compressed_value_bit_count = self.terminal_code.bit_count;

        to_write = to_write << compressed_value_bit_count;

        to_write = to_write | compressed_value;

        to_write_bit_count = to_write_bit_count + compressed_value_bit_count;
        
        let byte_boundary_offset = 8 - (to_write_bit_count % 8);

        to_write = to_write << byte_boundary_offset;

        to_write_bit_count = to_write_bit_count + byte_boundary_offset;

        while to_write_bit_count >= 8 {
            to_write_bit_count = to_write_bit_count - 8;

            let output_byte = to_write >> to_write_bit_count;

            if to_write_bit_count > 0 {
                let mask = u32::MAX >> (32 - to_write_bit_count); // errors if to_write_bit_count is 0

                to_write = to_write & mask;
            }

            output.push(output_byte as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_compresses_a_single_byte() {
        let uncompressed_byte: u8 = 0xE4;

        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[uncompressed_byte as usize] = 0b1;
        bit_counts[uncompressed_byte as usize] = 1;

        let table = HuffmanTable { values, bit_counts };
        
        let terminal_code = TerminalCode { bit_count: 0, value: 0x00 };

        let huffman = Huffman { table, terminal_code };

        let src = vec![uncompressed_byte];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b10000000]);
    }

    #[test]
    fn it_compresses_multiple_bytes() {
        let uncompressed_byte: u8 = 0xFF;

        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[uncompressed_byte as usize] = 0b1;
        bit_counts[uncompressed_byte as usize] = 1;

        let table = HuffmanTable { values, bit_counts };
        
        let terminal_code = TerminalCode { bit_count: 0, value: 0x00 };

        let huffman = Huffman { table, terminal_code };

        let src = vec![uncompressed_byte, uncompressed_byte, uncompressed_byte];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b11100000]);
    }

    #[test]
    fn it_compresses_across_byte_boundaries() {
        let uncompressed_byte: u8 = 0xDB;

        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[uncompressed_byte as usize] = 0b11111;
        bit_counts[uncompressed_byte as usize] = 5;

        let table = HuffmanTable { values, bit_counts };
        
        let terminal_code = TerminalCode { bit_count: 0, value: 0x00 };

        let huffman = Huffman { table, terminal_code };

        let src = vec![uncompressed_byte, uncompressed_byte];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b11111111, 0b11000000]);
    }

    #[test]
    fn it_compresses_different_bytes() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        let src = vec![0xA0, 0xCB, 0xB3];
        let mut output = Vec::new();

        values[0xA0] = 0b01;
        bit_counts[0xA0] = 2;

        values[0xCB] = 0b10;
        bit_counts[0xCB] = 2;

        values[0xB3] = 0b11;
        bit_counts[0xB3] = 2;

        let table = HuffmanTable { values, bit_counts };
        
        let terminal_code = TerminalCode { bit_count: 0, value: 0x00 };

        let huffman = Huffman { table, terminal_code };

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b01101100]);
    }

    #[test]
    fn it_compresses_to_just_zeroes() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        let uncompressed_byte: u8 = 0x92; // to be compressed as two zeroes
        values[uncompressed_byte as usize] = 0b0;
        bit_counts[uncompressed_byte as usize] = 2;

        let uncompressed_byte_2: u8 = 0x0C; // to be compressed as three ones
        values[uncompressed_byte_2 as usize] = 0b111;
        bit_counts[uncompressed_byte_2 as usize] = 3;

        let table = HuffmanTable { values, bit_counts };
        
        let terminal_code = TerminalCode { bit_count: 0, value: 0x00 };

        let huffman = Huffman { table, terminal_code };

        let src = vec![uncompressed_byte, uncompressed_byte_2];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b00111000]);
    }
}
