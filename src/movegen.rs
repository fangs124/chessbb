use super::chessmove::*;
use super::*;

const fn update_state(chessboard: &ChessBoard, chess_move: ChessMove) -> ChessBoard {
    !TODO
}

const fn generate_moves(chessboard: &ChessBoard) -> MoveVec {
    !TODO
}

//pub const fn get_history(&self) -> [([BitBoard; 12], u16); 8] {
//    let mut new = self.history;
//    let mut i = 0;
//    while i < 7 {
//        new[i] = self.history[i + 1];
//        i += 1;
//    }
//    new[7] = (self.piece_bbs, self.rep_count());
//
//    return new;
//}
