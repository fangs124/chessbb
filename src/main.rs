//use chessbb::chessmove::*;
use chessbb::*;
use core::error;
//use chessbb::chessmove::ChessMove;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use std::{env, time::Instant};

extern crate chessbb;
fn main() {
    unsafe { env::set_var("RUST_BACKTRACE", "1") };
    //old_main();
    perft_test(None);
}

fn old_main() {
    /* from starting pos */
    let fen = "8/2N4k/8/3p4/N1P1p3/1P2P2K/PQB2P2/R1B3R1 w - - 0 1";
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    //let datas = [2551, 2455, 1959, 1999];
    let datas = [1674];
    let moves = datas.map(|data: u16| return ChessMove::from_raw(data));
    let mut chessgame = ChessBoard::from_fen(fen);
    chessgame.print_debug();
    for chessmove in moves {
        chessgame.update_state(&chessmove);
    }
    let (mut moves, _) = chessgame.try_generate_moves();
    moves.sort_by(LexiOrd::lexi_cmp);

    //old test
    /*
    let mut chessboard = ChessBoard::start_pos();
    let move1 = ChessMove { data: 1487 }; //a2a3
    let move2 = ChessMove { data: 2551 }; //a7a5
    let move3 = ChessMove { data: 1934 }; //b2b4
    let move4 = ChessMove { data: 1959 }; //a5b4
    let move5 = ChessMove { data: 1869 }; //c2c4
    let move6 = ChessMove { data: 1535 }; //a8a3
    let move7 = ChessMove { data: 1988 }; //d1a4
    //three errors: b7b5, f7f5, f7f6
    let move8a = ChessMove { data: 2486 }; //b7b5
    let move9a = ChessMove { data: 2461 }; //c4b5
    //two errors: a3a1, c7c5
    let move10aa = ChessMove { data: 471 }; //a3a1
    let move11aa = ChessMove { data: 479 }; //a4a1
    let move12aa = ChessMove { data: 2421 }; //c7c5
    let move13aa = ChessMove { data: 259 }; //e1d1
    let move14aa = ChessMove { data: 1438 }; //b4b3
    let move15aa = ChessMove { data: 2503 }; //a1a5
    let move16aa = ChessMove { data: 2226 }; //f7f5
    let move17aa = ChessMove { data: 3879 }; //a5d8
    chessboard.update_state(move1);
    chessboard.update_state(move2);
    chessboard.update_state(move3);
    chessboard.update_state(move4);
    chessboard.update_state(move5);
    chessboard.update_state(move6);
    chessboard.update_state(move7);
    chessboard.update_state(move8a);
    chessboard.update_state(move9a);
    chessboard.update_state(move10aa);
    chessboard.update_state(move11aa);
    chessboard.update_state(move12aa);
    chessboard.update_state(move13aa);
    chessboard.update_state(move14aa);
    chessboard.update_state(move15aa);
    chessboard.update_state(move16aa);
    chessboard.update_state(move17aa);
    */
    /* from kiwipete pos */
    //let start_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"; //kiwipete
    //missing e5f4
    //e5d7, a6b5, d7b8, b6d7, e2b5, c7c6, b8d7, e8d7, d5c6, d7c8, b5a6, e7b7, f3f5, h3g2, a1b1, g2h1b, a2a3, c8b8
    //f6b6, e6e5, d2f4
    //let datas_branch_a = [
    //    3363, 2479, 4020, 3374, 2443, 2933, 3390, 3387, 2916, 3956, 3046, 3507, 2194, 592, 391, 53257, 1487, 4029,
    //    2466, 2283, 1676,
    //];
    //e5d7, a6b5, d7b8, b6d7, e2b5, c7c6, b8d7, e8d7, d5c6, d7c8, c6c7
    //let datas_branch_b = [3363, 2479, 4020, 3374, 2443, 2933, 3390, 3387, 2916, 3956, 3956];

    // Missing move f4e3

    /* position 3 */
    //let start_fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    ////a5a6, h5h7, a6a7, c7c5, a7a6, c5c4, e2e4, c4c3, b4a4, c3c2, a4a5, h4h5, g2g4
    //let datas = [3047, 3104, 3567, 2421, 3063, 1893, 1739, 1373, 2014, 853, 2527, 2072, 1609];
    //let moves = datas.map(|x| ChessMove { data: x });
    //let mut chessboard = ChessBoard::from_fen(start_fen);
    //for chessmove in moves {
    //    chessboard.update_state(chessmove);
    //}

    /* position 4 */
    //let start_fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    ////b4c5, e8c8, a1b1, d8e8
    //let datas = [2398, 32635, 391, 3836];
    //let moves = datas.map(|x| ChessMove { data: x });
    //let mut chessboard = ChessBoard::from_fen(start_fen);
    //for chessmove in moves {
    //    chessboard.update_state(chessmove);
    //}
    //let mut moves = chessboard.generate_moves();
    //moves.sort();

    /* position 5 */
    //let start_fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    ////
    //let datas = [];
    //let moves = datas.map(|x| ChessMove { data: x });
    //let mut chessboard = ChessBoard::from_fen(start_fen);
    //for chessmove in moves {
    //    chessboard.update_state(chessmove);
    //}
    //let mut moves = chessboard.generate_moves();
    //moves.sort();

    /* position 6 */
    //let start_fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    ////
    //let datas = [];
    //let moves = datas.map(|x| ChessMove { data: x });
    //let mut chessboard = ChessBoard::from_fen(start_fen);
    //for chessmove in moves {
    //    chessboard.update_state(chessmove);
    //}
    //let mut moves = chessboard.generate_moves();
    //moves.sort();

    /* error msg here */

    /* ============== */
    //println!("chessmove: {:#?}", moves[8]);
    //println!("castle_bools: {:#?}", chessboard.castle_bools);
    //println!("chessboard.enpassant_bb:\n{}\n", chessboard.enpassant_bb);
    //println!("chessboard.pinned_bb:\n{}\n", chessboard.pinned_bb);
    //println!("chessboard.pinner_bb:\n{}\n", chessboard.pinner_bb);
    //println!("chessboard.check_bb:\n{}\n", chessboard.check_bb);
    //println!("chessboard.check_mask:\n{}\n", chessgame.check_mask);
    println!("==== start position ====\n");
    println!("{}", chessgame);
    let mut result_str_vec = Vec::<String>::new();
    for chessmove in moves {
        let mut s = chessmove.print_move();
        s.push_str(format!(" - data: {}", chessmove.data()).as_str());
        result_str_vec.push(s);
    }
    for result_str in result_str_vec {
        println!("{result_str}");
    }
    println!();
    println!("========================");
    //println!("white rook:\n{}", chessboard.piece_bb(cpt!(R)));
    //println!("black rook:\n{}", chessboard.piece_bb(cpt!(r)));
    //println!("mailbox:\n{:#?}", chessboard.mailbox());
    //println!("========================");
    //panic!();
    let mut depth: usize = 1;
    let max_depth: usize = 7;
    while depth <= max_depth {
        let now = Instant::now();
        let total = chessgame.perft_count(depth);
        let elapsed = now.elapsed();
        let (mut moves, _) = chessgame.try_generate_moves();
        moves.sort_by(LexiOrd::lexi_cmp);
        let mut result_str_vec = Vec::<String>::new();

        for chessmove in moves {
            let mut s = chessmove.print_move();
            let mut state = chessgame.clone();
            state.update_state(&chessmove);
            let branch_total = state.perft_count(depth - 1);
            s.push_str(format!(" - {branch_total}").as_str());
            result_str_vec.push(s);
        }
        println!("depth: {depth}, time: {}ms, total: {total}", elapsed.as_millis());

        for result_str in result_str_vec {
            println!("{result_str}");
        }
        println!("\n");
        depth += 1;
    }
}

fn perft_test(skip_to: Option<usize>) {
    let mut node_count: u64 = 0;
    let path = Path::new("standard.epd");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {display}: {why}"),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => print!("{} contains:\n{}", display, s),
    }
    //print!("{s}");
    let lines = s.split('\n');
    let mut error_vec = Vec::new();
    let mut num: usize = 0;
    let mut elapsed_total: Duration = Duration::new(0, 0);
    for line in lines {
        let mut sections = line.split(';');
        //let str_vec: Vec<&str> = line.split(';').collect();
        let start_fen = sections.next().unwrap();
        num += 1;
        if skip_to.is_some() {
            if num != skip_to.unwrap() {
                continue;
            }
        }
        let mut chessgame = ChessBoard::from_fen(start_fen);
        println!("\n======== position number {num} ========\n");
        println!("fen: {start_fen}");
        println!("{}", chessgame);
        println!("=======================================");
        for section in sections {
            let now: Instant = Instant::now();
            let section_vec: Vec<_> = section.split_ascii_whitespace().collect();
            let depth: usize = section_vec[0].chars().filter(|x| x.is_ascii_digit()).collect::<String>().parse().unwrap();
            let result_count: u64 = section_vec[1].parse().unwrap();
            let total_count: u64 = chessgame.perft_count(depth); //here
            elapsed_total += now.elapsed();
            //println!("AAAAAAAAAAAAAAAA");
            let result_str = match result_count == total_count {
                true => "Ok!",
                false => "Error!",
            };
            if result_str == "Error!" {
                error_vec.push(format!("start_fen: {start_fen}\ndepth: {depth}, result_count: {result_count}, total_count: {total_count}, {result_str}"))
            }
            println!("depth: {depth}, result_count: {result_count}, total_count: {total_count}, {result_str}");
            node_count += total_count;
        }
        println!("=======================================");
    }
    let has_error = !error_vec.is_empty();
    if has_error {
        for str_error in error_vec {
            println!("{str_error}");
        }
    } else {
        println!("done... no error!");
    }
    println!("total positions: {node_count}, time: {}ms", elapsed_total.as_millis());
    println!("speed: {:.2}Mnps", ((node_count as f64) / (1000000.0)) / elapsed_total.as_secs_f64());
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn perft_test() {
        let path = Path::new("standard.epd");
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {display}: {why}"),
            Ok(file) => file,
        };

        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => print!("{} contains:\n{}", display, s),
        }
        //print!("{s}");
        let lines = s.split('\n');
        let mut error_vec = Vec::new();
        let mut num: usize = 0;
        for line in lines {
            let mut sections = line.split(';');
            //let str_vec: Vec<&str> = line.split(';').collect();
            let start_fen = sections.next().unwrap();
            num += 1;
            let mut perft_result: Vec<(usize, usize)> = Vec::new();
            let mut chessgame = ChessBoard::from_fen(start_fen);
            println!("\n======== position number {num} ========\n");
            println!("fen: {start_fen}");
            println!("{}", chessgame);
            println!("=======================================");
            for section in sections {
                let section_vec: Vec<_> = section.split_ascii_whitespace().collect();
                let depth: usize = section_vec[0].chars().filter(|x| x.is_ascii_digit()).collect::<String>().parse().unwrap();
                let result_count: u64 = section_vec[1].parse().unwrap();
                let total_count = chessgame.perft_count(depth);
                let result_str = match result_count == total_count {
                    true => "Ok!",
                    false => "Error!",
                };
                if result_str == "Error!" {
                    error_vec.push(format!("start_fen: {start_fen}\ndepth: {depth}, result_count: {result_count}, total_count: {total_count}, {result_str}"))
                }
                println!("depth: {depth}, result_count: {result_count}, total_count: {total_count}, {result_str}");
            }
            println!("=======================================");
        }
        let has_error = !error_vec.is_empty();
        if has_error {
            for str_error in error_vec {
                println!("{str_error}");
            }
        } else {
            println!("done... no error!");
        }
        assert!(!has_error);
    }
}
