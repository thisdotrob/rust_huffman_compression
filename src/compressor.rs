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
