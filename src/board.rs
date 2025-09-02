use std::collections::HashSet;

use crate::piece::Color::{self, Black, White};
use crate::piece::PieceKind::{self, *};
use crate::piece::{Piece, Position};

pub struct Board {
    pieces: Vec<Piece>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: vec![
                Piece::new(Rook, White, (0, 0).into()),
                Piece::new(Knight, White, (1, 0).into()),
                Piece::new(Bishop, White, (2, 0).into()),
                Piece::new(Queen, White, (3, 0).into()),
                Piece::new(King, White, (4, 0).into()),
                Piece::new(Bishop, White, (5, 0).into()),
                Piece::new(Knight, White, (6, 0).into()),
                Piece::new(Rook, White, (7, 0).into()),
                // #
                Piece::new(Pawn, White, (0, 1).into()),
                Piece::new(Pawn, White, (1, 1).into()),
                Piece::new(Pawn, White, (2, 1).into()),
                Piece::new(Pawn, White, (3, 1).into()),
                Piece::new(Pawn, White, (4, 1).into()),
                Piece::new(Pawn, White, (5, 1).into()),
                Piece::new(Pawn, White, (6, 1).into()),
                Piece::new(Pawn, White, (7, 1).into()),
                // #
                // Rook.new(:black, 0, 7),
                // Knight.new(:black, 1, 7),
                // Bishop.new(:black, 2, 7),
                // Queen.new(:black, 3, 7),
                // King.new(:black, 4, 7),
                // Bishop.new(:black, 5, 7),
                // Knight.new(:black, 6, 7),
                // Rook.new(:black, 7, 7),
                Piece::new(Rook, Black, (0, 7).into()),
                Piece::new(Knight, Black, (1, 7).into()),
                Piece::new(Bishop, Black, (2, 7).into()),
                Piece::new(Queen, Black, (3, 7).into()),
                Piece::new(King, Black, (4, 7).into()),
                Piece::new(Bishop, Black, (5, 7).into()),
                Piece::new(Knight, Black, (6, 7).into()),
                Piece::new(Rook, Black, (7, 7).into()),
                // #
                Piece::new(Pawn, Black, (0, 6).into()),
                Piece::new(Pawn, Black, (1, 6).into()),
                Piece::new(Pawn, Black, (2, 6).into()),
                Piece::new(Pawn, Black, (3, 6).into()),
                Piece::new(Pawn, Black, (4, 6).into()),
                Piece::new(Pawn, Black, (5, 6).into()),
                Piece::new(Pawn, Black, (6, 6).into()),
                Piece::new(Pawn, Black, (7, 6).into()),
            ],
        }
    }

    /// update the board to move the piece and remove the taken piece, if there is one
    pub fn move_piece(&mut self, from: &Position, to: &Position) -> Option<Piece> {
        let taken_piece = self.take_piece_at(to);

        let piece_to_move = self.get_piece_mut(from).unwrap();

        piece_to_move.position = *to;
        piece_to_move.has_moved = true;

        if piece_to_move.kind == PieceKind::Pawn && [0, 7].contains(&to.row) {
            piece_to_move.kind = PieceKind::Queen
        }

        taken_piece
    }

    /// get what piece is at position
    pub fn get_piece(&self, position: &Position) -> Option<&Piece> {
        self.pieces.iter().find(|piece| piece.position == *position)
    }

    /// all pieces of given color
    pub fn get_pieces(&self, color: Color) -> impl Iterator<Item = &Piece> {
        self.pieces.iter().filter(move |piece| piece.color == color)
    }

    /// all positions being attacked by given color
    /// this is a naive calculation that does not take
    /// the opponent's attacks into account
    pub fn all_attacks(&self, color: Color) -> HashSet<Position> {
        let mut attacks = HashSet::new();

        let pieces = self.get_pieces(color);

        for piece in pieces {
            let moves = piece.attacks(self);
            attacks.extend(moves);
        }

        attacks
    }

    fn get_piece_mut(&mut self, position: &Position) -> Option<&mut Piece> {
        self.pieces
            .iter_mut()
            .find(|piece| piece.position == *position)
    }

    fn take_piece_at(&mut self, position: &Position) -> Option<Piece> {
        let i = self
            .pieces
            .iter()
            .position(|piece| piece.position == *position);

        if let Some(i) = i {
            Some(self.pieces.remove(i))
        } else {
            None
        }
    }
}
