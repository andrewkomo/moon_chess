use std::hash::{Hash};
use std::cmp;
use anchor_lang::prelude::*;


#[derive(PartialEq, Hash, Clone, Copy)]
pub enum Pieces {
    Empty,
    R,
    N,
    B,
    Q,
    K,
    P,
    PToR,
    PToN,
    PToB,
    PToQ,
}
impl Default for Pieces {
    fn default() -> Self { Pieces::Empty }
}
impl Pieces {
    pub fn is_pawn(&self) -> bool {
        match self {
            Self::P | Self::PToR | Self::PToN | Self::PToB | Self::PToQ => true,
            _ => false,
        }
    }
}

#[derive(PartialEq, AnchorSerialize, AnchorDeserialize, Default, Clone, Copy)]
pub struct Turn {
    pub turn: u16, // piece (4 bits) | from_rank (3) | from_col (3) | to_rank (3) | to_col (3)
}
impl Turn {
    pub fn piece(&self) -> Pieces {
        let piece_num = self.turn >> 12;
        match piece_num {
            0 => Pieces::R,
            1 => Pieces::N,
            2 => Pieces::B,
            3 => Pieces::Q,
            4 => Pieces::K,
            5 => Pieces::P,
            6 => Pieces::PToR,
            7 => Pieces::PToN,
            8 => Pieces::PToB,
            9 => Pieces::PToQ,
            _ => Pieces::Empty,
        }
    }
    pub fn from_rank(&self) -> usize {
        ((self.turn >> 9) & 0b111).into()
    }
    pub fn from_col(&self) -> usize {
        ((self.turn >> 6) & 0b111).into()
    }
    pub fn to_rank(&self) -> usize {
        ((self.turn >> 3) & 0b111).into()
    }
    pub fn to_col(&self) -> usize {
        (self.turn & 0b111).into()
    }
    pub fn is_valid_dir(&self) -> bool {
        // Check the coordinates are good and that the piece moves
        if (self.from_rank() >= 8) || (self.from_col() >= 8) || (self.to_rank() >= 8) || (self.to_col() >= 8) {
            return false;
        }
        if (self.from_rank() == self.to_rank()) && (self.from_col() == self.to_col()) {
            return false;
        }

        // Check the piece is allowed to move as specified
        let rank_diff = cmp::max(self.from_rank(),self.to_rank()) - cmp::min(self.from_rank(),self.to_rank());
        let col_diff = cmp::max(self.from_col(),self.to_col()) - cmp::min(self.from_col(),self.to_col());
        match self.piece() {
            Pieces::R => (rank_diff == 0) || (col_diff == 0),
            Pieces::N => (rank_diff == 2 && col_diff == 1) || (rank_diff == 1 && col_diff == 2),
            Pieces::B => rank_diff == col_diff,
            Pieces::Q => (rank_diff == col_diff) || (rank_diff == 0) || (col_diff == 0),
            Pieces::K => ((rank_diff <= 1) && (col_diff <= 1)) ||
                ((col_diff == 2) && (rank_diff == 0) && ((self.to_col() == 6) || (self.to_col() == 2))),
            Pieces::Empty => false,
            // Handle all the pawns the same
            _ => ((col_diff == 0) && (rank_diff <= 2)) || ((col_diff == 1) && (rank_diff == 1)),
        }
    }
}