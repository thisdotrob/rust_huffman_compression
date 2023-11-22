mod compressor;
mod huffman_table;
mod terminal_code;

pub use crate::huffman_table::HuffmanTable;
pub use crate::terminal_code::TerminalCode;

use crate::compressor::Compressor;

pub struct Huffman {
    pub table: HuffmanTable,
    pub terminal_code: Option<TerminalCode>,
}

impl Huffman {
    pub fn new(table: HuffmanTable, terminal_code: Option<TerminalCode>) -> Huffman {
        return Huffman {
            terminal_code,
            table,
        };
    }

    pub fn compress(&mut self, src: Vec<u8>, output: &mut Vec<u8>) {
        let mut compressor = Compressor::new(&self.table);

        for byte in src {
            compressor.compress_byte(byte);

            for compressed_byte in &mut compressor {
                output.push(compressed_byte);
            }
        }

        if let Some(terminal_code) = &self.terminal_code {
            compressor.append_terminal_code(terminal_code);
        }

        compressor.end();

        for compressed_byte in &mut compressor {
            output.push(compressed_byte);
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

        let mut huffman = Huffman::new(table, None);

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

        let mut huffman = Huffman::new(table, None);

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

        let mut huffman = Huffman::new(table, None);

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

        let mut huffman = Huffman::new(table, None);

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

        let mut huffman = Huffman::new(table, None);

        let src = vec![uncompressed_byte, uncompressed_byte_2];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b00111000]);
    }

    #[test]
    fn it_compresses_to_values_with_leading_zeroes() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        let uncompressed_byte: u8 = 0x92;
        values[uncompressed_byte as usize] = 0b1;
        bit_counts[uncompressed_byte as usize] = 8;

        let table = HuffmanTable { values, bit_counts };

        let mut huffman = Huffman::new(table, None);

        let src = vec![uncompressed_byte];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b00000001]);
    }

    #[test]
    fn it_adds_a_termination_code_if_provided() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        let uncompressed_byte: u8 = 0x92;
        values[uncompressed_byte as usize] = 0b1010;
        bit_counts[uncompressed_byte as usize] = 4;

        let terminal_code = TerminalCode {
            bit_count: 3,
            value: 0b111,
        };

        let table = HuffmanTable { values, bit_counts };

        let mut huffman = Huffman::new(table, Some(terminal_code));

        let src = vec![uncompressed_byte];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b1010_111_0]);
    }

    #[test]
    fn it_adds_a_new_compressed_byte_for_the_termination_code_if_necessary() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        let uncompressed_byte: u8 = 0x92;
        values[uncompressed_byte as usize] = 0b10000000;
        bit_counts[uncompressed_byte as usize] = 8;

        let terminal_code = TerminalCode {
            bit_count: 3,
            value: 0b101,
        };

        let table = HuffmanTable { values, bit_counts };

        let mut huffman = Huffman::new(table, Some(terminal_code));

        let src = vec![uncompressed_byte];
        let mut output = Vec::new();

        huffman.compress(src, &mut output);

        assert_eq!(output, vec![0b10000000, 0b10100000]);
    }
}
