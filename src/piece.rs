use crate::Color::*;
use crate::{board::Board, color::Color};
use std::collections::HashSet;
use std::ops::Rem;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub row: i8,
    pub column: i8,
}

impl Position {
    pub fn new(column: i8, row: i8) -> Self {
        Self { column, row }
    }

    pub fn is_on_board(&self) -> bool {
        self.column <= 7 && self.column >= 0 && self.row <= 7 && self.row >= 0
    }

    pub fn color(&self) -> Color {
        if (self.column + self.row).rem(2) == 0 {
            Color::Black
        } else {
            Color::White
        }
    }
}

impl From<(i8, i8)> for Position {
    fn from(value: (i8, i8)) -> Self {
        Self {
            column: value.0,
            row: value.1,
        }
    }
}

// #[derive(Clone, Copy)]
// pub struct Position(i8);

// impl Position {
//     pub fn new(column: u8, row: u8) -> Self {
//         assert!(column <= 7);
//         assert!(row <= 7);
//         let mut data = 0;
//         data |= column & 0b0000_1111;
//         data |= row << 4;
//         let data = data as i8;
//         Self(data)
//     }

//     pub fn column(&self) -> u8 {
//         (self.0 & 0b0000_1111) as u8
//     }

//     pub fn row(&self) -> u8 {
//         ((self.0 & 0b1111_0000) >> 4) as u8
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
    pub position: Position,
    pub has_moved: bool,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color, position: Position) -> Self {
        Self {
            kind,
            color,
            position,
            has_moved: false,
        }
    }

    pub fn moves(&self, board: &Board) -> Vec<Position> {
        match self.kind {
            PieceKind::King => king_moves(self, board),
            PieceKind::Queen => queen_moves(self, board),
            PieceKind::Rook => rook_moves(self, board),
            PieceKind::Bishop => bishop_moves(self, board),
            PieceKind::Knight => knight_moves(self, board),
            PieceKind::Pawn => pawn_moves(self, board),
        }
    }

    pub fn repr(&self) -> &str {
        use PieceKind::*;

        match (self.kind, self.color) {
            (King, Black) => "♚",
            (King, White) => "♔",
            (Queen, Black) => "♛",
            (Queen, White) => "♕",
            (Rook, Black) => "♜",
            (Rook, White) => "♖",
            (Bishop, Black) => "♝",
            (Bishop, White) => "♗",
            (Knight, Black) => "♞",
            (Knight, White) => "♘",
            (Pawn, Black) => "♟",
            (Pawn, White) => "♙",
        }
    }
}

// TODO
// - en passant
fn pawn_moves(piece: &Piece, board: &Board) -> Vec<Position> {
    let straight_move_1 = match piece.color {
        Black => |position: &Position| (position.column, position.row - 1),
        White => |position: &Position| (position.column, position.row + 1),
    };

    let diagonal_take_left = match piece.color {
        Black => |position: &Position| (position.column + 1, position.row - 1),
        White => |position: &Position| (position.column + 1, position.row + 1),
    };

    let diagonal_take_right = match piece.color {
        Black => |position: &Position| (position.column - 1, position.row - 1),
        White => |position: &Position| (position.column - 1, position.row + 1),
    };

    let mut moves = vec![];

    if piece.has_moved {
        let ahead_1 = straight_move_1(&piece.position).into();
        if board.get_piece(&ahead_1).is_none() {
            moves.push(ahead_1);
        }
    } else {
        let ahead_1 = straight_move_1(&piece.position).into();
        if board.get_piece(&ahead_1).is_none() {
            moves.push(ahead_1);

            let ahead_2 = straight_move_1(&ahead_1).into();
            if board.get_piece(&ahead_2).is_none() {
                moves.push(ahead_2);
            }
        }
    }

    let diagonal_left = diagonal_take_left(&piece.position).into();
    if let Some(other_piece) = board.get_piece(&diagonal_left)
        && other_piece.color != piece.color
    {
        moves.push(diagonal_left);
    }

    let diagonal_right = diagonal_take_right(&piece.position).into();
    if let Some(other_piece) = board.get_piece(&diagonal_right)
        && other_piece.color != piece.color
    {
        moves.push(diagonal_right);
    }

    moves
}

fn bishop_moves(piece: &Piece, board: &Board) -> Vec<Position> {
    let mut moves = vec![];

    for how_to_move in [
        |position: &Position| (position.column + 1, position.row + 1).into(),
        |position: &Position| (position.column + 1, position.row - 1).into(),
        |position: &Position| (position.column - 1, position.row + 1).into(),
        |position: &Position| (position.column - 1, position.row - 1).into(),
    ] {
        let moves_for_direction = moves_in_direction(piece, board, how_to_move);

        moves.extend_from_slice(&moves_for_direction);
    }

    moves
}

// TODO
// - castling
fn rook_moves(piece: &Piece, board: &Board) -> Vec<Position> {
    let mut moves = vec![];

    for how_to_move in [
        |position: &Position| (position.column + 1, position.row).into(),
        |position: &Position| (position.column - 1, position.row).into(),
        |position: &Position| (position.column, position.row + 1).into(),
        |position: &Position| (position.column, position.row - 1).into(),
    ] {
        let moves_for_direction = moves_in_direction(piece, board, how_to_move);

        moves.extend_from_slice(&moves_for_direction);
    }

    moves
}

fn queen_moves(piece: &Piece, board: &Board) -> Vec<Position> {
    let mut rook_moves = rook_moves(piece, board);
    let bishop_moves = bishop_moves(piece, board);
    rook_moves.extend_from_slice(&bishop_moves);
    rook_moves
}

// TODO
// - castling
fn king_moves(piece: &Piece, board: &Board) -> Vec<Position> {
    let same_color_piece_positions: HashSet<_> = board
        .get_pieces(piece.color)
        .filter(|other_piece| *other_piece != piece)
        .map(|piece| piece.position)
        .collect();

    let all_enemy_attacks = board.all_attacks(piece.color.invert());

    [-1, 0, 1]
        .into_iter()
        .flat_map(|column| {
            [-1, 0, 1].into_iter().filter_map(move |row| {
                if column == 0 && row == 0 {
                    None
                } else {
                    Some(Position::new(
                        piece.position.column + column,
                        piece.position.row + row,
                    ))
                }
            })
        })
        .filter(|position| position.is_on_board())
        .filter(|position| !same_color_piece_positions.contains(position))
        .filter(|position| !all_enemy_attacks.contains(position))
        .collect()
}

fn knight_moves(piece: &Piece, board: &Board) -> Vec<Position> {
    [
        (piece.position.column + 2, piece.position.row + 1).into(),
        (piece.position.column + 2, piece.position.row - 1).into(),
        (piece.position.column + 1, piece.position.row + 2).into(),
        (piece.position.column + 1, piece.position.row - 2).into(),
        (piece.position.column - 2, piece.position.row + 1).into(),
        (piece.position.column - 2, piece.position.row - 1).into(),
        (piece.position.column - 1, piece.position.row + 2).into(),
        (piece.position.column - 1, piece.position.row - 2).into(),
    ]
    .into_iter()
    .filter(|position: &Position| position.is_on_board())
    .filter(|position| {
        if let Some(other_piece) = board.get_piece(position) {
            other_piece.color != piece.color
        } else {
            true
        }
    })
    .collect()
}

// for rook, bishop, and by extension, queen
fn moves_in_direction<F>(piece: &Piece, board: &Board, how_to_move: F) -> Vec<Position>
where
    F: Fn(&Position) -> Position,
{
    let mut moves = vec![];

    loop {
        let previous_position = if let Some(last_move) = moves.last() {
            last_move
        } else {
            &piece.position
        };

        let possible_next_move: Position = how_to_move(previous_position);

        if let Some(other_piece) = board.get_piece(&possible_next_move) {
            if other_piece.color != piece.color {
                moves.push(possible_next_move);
                break;
            } else {
                break;
            }
        } else if possible_next_move.is_on_board() {
            moves.push(possible_next_move);
        } else {
            break;
        }
    }

    moves
}

// #[cfg(test)]
// mod tests {
//     use crate::piece::Position;

//     #[test]
//     fn position_test() {
//         let column = 4;
//         let row = 7;
//         let position = Position::new(column, row);
//         assert_eq!(position.column(), column);
//         assert_eq!(position.row(), row);
//     }
// }
