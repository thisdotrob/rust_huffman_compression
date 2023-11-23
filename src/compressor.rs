mod buffer;

use crate::compressor::buffer::CompressorBuffer;
use crate::huffman_table::HuffmanTable;
use crate::terminal_code::TerminalCode;

pub struct Compressor<'a> {
    table: &'a HuffmanTable,
    buffer: CompressorBuffer,
}

impl<'a> Compressor<'a> {
    pub fn new(table: &'a HuffmanTable) -> Self {
        Compressor {
            table,
            buffer: CompressorBuffer::new(),
        }
    }

    pub fn compress_byte(&mut self, byte: u8) {
        let value = self.table.get_compressed_value(byte);
        let bit_count = self.table.get_compressed_value_bit_count(byte);
        self.buffer.write_bits(value, bit_count);
    }

    fn get_compressed_byte(&mut self) -> Option<u8> {
        self.buffer.read_byte()
    }

    pub fn append_terminal_code(&mut self, terminal_code: &TerminalCode) {
        self.buffer
            .write_bits(terminal_code.value, terminal_code.bit_count);
    }

    pub fn end(&mut self) {
        let byte_boundary_offset = self.buffer.byte_boundary_offset();

        if byte_boundary_offset != 0 {
            let padding_value = 0b0;
            let padding_bit_count = 8 - byte_boundary_offset;
            self.buffer.write_bits(padding_value, padding_bit_count);
        }
    }
}

impl<'a> Iterator for Compressor<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.get_compressed_byte()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_none_when_compress_byte_has_not_been_called() {
        let table = HuffmanTable {
            values: [0; 256],
            bit_counts: [1; 256],
        };

        let mut compressor = Compressor::new(&table);

        let result = compressor.next();

        assert_eq!(result, None);
    }

    #[test]
    fn it_returns_none_when_compressed_values_total_less_than_a_byte() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x1F] = 0b10;
        bit_counts[0x1F] = 2;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x1F);

        let result = compressor.next();

        assert_eq!(result, None);
    }

    #[test]
    fn it_returns_a_byte_padded_with_zeroes_when_end_is_called() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x08] = 0b10;
        bit_counts[0x08] = 2;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x08);

        compressor.end();

        let result = compressor.next();

        assert_eq!(result, Some(0b10_000000));
    }

    #[test]
    fn it_returns_the_byte_when_compressed_values_total_exactly_a_byte() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0xFF] = 0b1010;
        bit_counts[0xFF] = 4;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0xFF);
        compressor.compress_byte(0xFF);

        let result = compressor.next();

        assert_eq!(result, Some(0b10101010));
    }

    #[test]
    fn it_returns_the_byte_and_then_none_when_compressed_values_total_between_one_and_two_bytes() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0xAD] = 0b11111;
        bit_counts[0xAD] = 5;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0xAD);
        compressor.compress_byte(0xAD);

        let result = compressor.next();

        assert_eq!(result, Some(0b11111111));

        let result = compressor.next();

        assert_eq!(result, None);
    }

    #[test]
    fn it_compresses_different_bytes() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x0B] = 0b11;
        bit_counts[0x0B] = 2;

        values[0x11] = 0b01;
        bit_counts[0x11] = 2;

        values[0x9D] = 0b0010;
        bit_counts[0x9D] = 4;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x0B);
        compressor.compress_byte(0x11);
        compressor.compress_byte(0x9D);

        let result = compressor.next();

        assert_eq!(result, Some(0b11_01_0010));
    }

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn it_panics_when_attempting_to_compress_to_a_single_32_bit_value() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x3C] = u32::MAX;
        bit_counts[0x3C] = 32;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x3C);
    }

    #[test]
    #[should_panic(expected = "attempt to shift right with overflow")]
    fn it_panics_when_next_is_called_and_compressed_values_exceed_32_bits() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x77] = 0b1111111100000000;
        bit_counts[0x77] = 16;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x77); // compressed values = 16 bits
        compressor.compress_byte(0x77); // compressed values = 32 bits
        compressor.compress_byte(0x77); // compressed values = 48 bits

        compressor.next();
    }

    #[test]
    fn it_does_not_panic_when_next_is_called_between_compressing_bytes() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x77] = 0b1111111100000000;
        bit_counts[0x77] = 16;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x77); // compressed values = 16 bits
        compressor.next(); // compressed values = 8 bits
        compressor.compress_byte(0x77); // compressed values = 24 bits
        compressor.next(); // compressed values = 16 bits
        compressor.compress_byte(0x77); // compressed values = 32 bits
        compressor.next();
    }

    #[test]
    fn it_can_append_a_termination_code() {
        let mut values = [0; 256];
        let mut bit_counts = [0; 256];

        values[0x12] = 0b11;
        bit_counts[0x12] = 2;

        let table = HuffmanTable { values, bit_counts };

        let mut compressor = Compressor::new(&table);

        compressor.compress_byte(0x12);

        let termination_code = TerminalCode {
            value: 0b100001,
            bit_count: 6,
        };

        compressor.append_terminal_code(&termination_code);

        assert_eq!(compressor.next(), Some(0b11_100001));
    }
}
