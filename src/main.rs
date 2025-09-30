//use chessbb::chessmove::*;
use chessbb::*;
use core::error;
//use chessbb::chessmove::ChessMove;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time::Duration;
use std::{env, time::Instant};

mod epds;
use epds::LARGE_TEST_EPDS;
const USE_RUSTIC_EPDS: bool = false;

extern crate chessbb;
fn main() {
    old_main();
    //perft_test(None);
}

fn old_main() {
    /* from starting pos */
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    //let datas = [2551, 2455, 1959, 1999];
    let datas = [];
    let moves = datas.map(|data: u16| return ChessMove::from_raw(data));
    let mut chessgame = ChessBoard::from_fen(fen);
    chessgame.print_debug();
    for chessmove in moves {
        chessgame.update_state(&chessmove);
    }
    let (mut moves, _) = chessgame.try_generate_moves();
    moves.sort_by(LexiOrd::lexi_cmp);

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
    let mut depth: usize = 8;
    let max_depth: usize = 8;
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

    let lines: Vec<&str> = match USE_RUSTIC_EPDS {
        true => LARGE_TEST_EPDS.to_vec(),
        false => s.split('\n').collect(),
    };
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
            let section_vec: Vec<_> = section.split_ascii_whitespace().collect();
            let depth: usize = section_vec[0].chars().filter(|x| x.is_ascii_digit()).collect::<String>().parse().unwrap();
            let result_count: u64 = section_vec[1].parse().unwrap();
            let now: Instant = Instant::now();
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
