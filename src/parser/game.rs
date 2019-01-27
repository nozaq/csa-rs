use nom::*;
use std::str;
use std::time::Duration;

use crate::value::*;
use super::time::{datetime, timelimit};

named!(line_sep, is_a!("\r\n,"));
named!(not_line_sep, is_not!("\r\n,"));
named!(comment, preceded!(tag!("'"), not_line_sep));
named!(comment_line, terminated!(comment, line_sep));
named!(color<Color>, map!(one_of!("+-"), |s| match s {
    '+' => Color::Black,
    _ => Color::White
}));
named!(decimal<Duration>, map_res!(digit, |s|
    str::from_utf8(s).map(|s| s.parse::<u64>().unwrap()).map(|i| Duration::from_secs(i))));

named!(fu<PieceType>, value!(PieceType::Pawn, tag!("FU")));
named!(ky<PieceType>, value!(PieceType::Lance, tag!("KY")));
named!(ke<PieceType>, value!(PieceType::Knight, tag!("KE")));
named!(gi<PieceType>, value!(PieceType::Silver, tag!("GI")));
named!(ki<PieceType>, value!(PieceType::Gold, tag!("KI")));
named!(ka<PieceType>, value!(PieceType::Bishop, tag!("KA")));
named!(hi<PieceType>, value!(PieceType::Rook, tag!("HI")));
named!(ou<PieceType>, value!(PieceType::King, tag!("OU")));
named!(to<PieceType>, value!(PieceType::ProPawn, tag!("TO")));
named!(ny<PieceType>, value!(PieceType::ProLance, tag!("NY")));
named!(nk<PieceType>, value!(PieceType::ProKnight, tag!("NK")));
named!(ng<PieceType>, value!(PieceType::ProSilver, tag!("NG")));
named!(um<PieceType>, value!(PieceType::Horse, tag!("UM")));
named!(ry<PieceType>, value!(PieceType::Dragon, tag!("RY")));
named!(al<PieceType>, value!(PieceType::All, tag!("AL")));
named!(piece_type<PieceType>, alt!(fu | ky | ke | gi | ki | ka | hi | ou | to | ny | nk | ng | um | ry | al));

named!(one_digit<u8>, map!(one_of!("0123456789"), |c: char| c.to_digit(10).unwrap() as u8));
named!(square<Square>, map!(tuple!(one_digit, one_digit), |(file, rank)| Square::new(file, rank)));

named!(version, preceded!(tag!("V"), alt_complete!(tag!("2.1") | tag!("2.2") | tag!("2"))));
named!(black_player, preceded!(tag!("N+"), not_line_sep));
named!(white_player, preceded!(tag!("N-"), not_line_sep));
named!(game_text_attr<GameAttribute>, map!(
    map_res!(not_line_sep, |s| str::from_utf8(s)),
    |s: &str| GameAttribute::Str(s.to_string())
));
named!(game_time_attr<GameAttribute>, map!(datetime, |t| GameAttribute::Time(t)));
named!(game_timelimit_attr<GameAttribute>, map!(timelimit, |t| GameAttribute::TimeLimit(t)));

named!(game_attr<(String, GameAttribute)>, preceded!(tag!("$"), separated_pair!(
    map_res!(is_not!(":"), |s: &[u8]| String::from_utf8(s.to_vec())),
    tag!(":"),
    alt_complete!(game_time_attr | game_timelimit_attr | game_text_attr)
)));

named!(handicap<Vec<(Square, PieceType)>>, preceded!(tag!("PI"), many0!(tuple!(square, piece_type))));

named!(grid_piece<Option<(Color, PieceType)>>, switch!(anychar,
    '+' => map!(piece_type, |pt| Some((Color::Black, pt))) |
    '-' => map!(piece_type, |pt| Some((Color::White, pt))) |
    _ => value!(None, take!(2))
));
named!(grid_row<[Option<(Color, PieceType)>; 9]>, count_fixed!(Option<(Color, PieceType)>, grid_piece, 9));
named!(grid<[[Option<(Color, PieceType)>; 9]; 9]>, do_parse!(
    r1: delimited!(tag!("P1"), grid_row, line_sep) >>
    r2: delimited!(tag!("P2"), grid_row, line_sep) >>
    r3: delimited!(tag!("P3"), grid_row, line_sep) >>
    r4: delimited!(tag!("P4"), grid_row, line_sep) >>
    r5: delimited!(tag!("P5"), grid_row, line_sep) >>
    r6: delimited!(tag!("P6"), grid_row, line_sep) >>
    r7: delimited!(tag!("P7"), grid_row, line_sep) >>
    r8: delimited!(tag!("P8"), grid_row, line_sep) >>
    r9: preceded!(tag!("P9"), grid_row) >>
    ([r1, r2, r3, r4, r5, r6, r7, r8, r9])
));

named!(piece_placement<Vec<(Color, Square, PieceType)>>, do_parse!(
    tag!("P") >>
    c: color >>
    pcs: many0!(tuple!(square, piece_type)) >>
    ( pcs.iter().map(|&(sq, pt)| (c, sq, pt)).collect::<Vec<_>>() )
));

named!(normal_move<Action>, do_parse!(
    c: color >>
    from: square >>
    to: square >>
    pt: piece_type >>
    ( Action::Move(c, from, to, pt) )
));

named!(special_move<Action>, preceded!(tag!("%"), alt!(
    value!(Action::Toryo,      tag!("TORYO")) |
    value!(Action::Matta,      tag!("MATTA")) |
    value!(Action::Tsumi,      tag!("TSUMI")) |
    value!(Action::Error,      tag!("ERROR")) |
    value!(Action::Kachi,      tag!("KACHI")) |
    value!(Action::Chudan,     tag!("CHUDAN")) |
    value!(Action::Fuzumi,     tag!("FUZUMI")) |
    value!(Action::Jishogi,    tag!("JISHOGI")) |
    value!(Action::Hikiwake,   tag!("HIKIWAKE")) |
    value!(Action::Sennichite, tag!("SENNICHITE"))
)));

named!(move_record<MoveRecord>, do_parse!(
    action: alt_complete!(normal_move | special_move) >>
    time: opt!(complete!(preceded!(line_sep, preceded!(tag!("T"), decimal)))) >>
        ( MoveRecord{action, time} )
));

named!(pub game_record<GameRecord>, do_parse!(
    many0!(comment_line) >>
    opt!(terminated!(version, line_sep)) >>
    many0!(comment_line) >>
    black_player: opt!(map_res!(terminated!(black_player, line_sep), |b| str::from_utf8(b))) >>
    many0!(comment_line) >>
    white_player: opt!(map_res!(terminated!(white_player, line_sep), |b| str::from_utf8(b))) >>
    many0!(comment_line) >>
    attrs: map!(
        opt!(terminated!(separated_list!(line_sep, preceded!(many0!(comment_line), game_attr)), line_sep)), 
        |v: Option<Vec<(String, GameAttribute)>>| v.unwrap_or(Vec::new())
    ) >>
    many0!(comment_line) >>
    drop_pieces: opt!(terminated!(handicap, line_sep)) >>
    many0!(comment_line) >>
    bulk: opt!(terminated!(grid, line_sep)) >>
    many0!(comment_line) >>
    add_pieces: many0!(terminated!(piece_placement, line_sep)) >>
    many0!(comment_line) >>
    side_to_move: terminated!(color, line_sep) >>
    many0!(comment_line) >>
    moves: many0!(terminated!(move_record, line_sep)) >>
    many0!(comment_line) >>
    (GameRecord{
        black_player: black_player.map(|s| s.to_string()),
        white_player: white_player.map(|s| s.to_string()),
        event: attrs.iter().find(|pair| pair.0 == "EVENT").map(|pair| pair.1.to_string()),
        site: attrs.iter().find(|pair| pair.0 == "SITE").map(|pair| pair.1.to_string()),
        start_time: attrs.iter().find(|pair| pair.0 == "START_TIME").and_then(|pair| 
            match pair.1 {
                GameAttribute::Time(ref t) => Some(t.clone()),
                _ => None
            }
        ),
        end_time: attrs.iter().find(|pair| pair.0 == "END_TIME").and_then(|pair| 
            match pair.1 {
                GameAttribute::Time(ref t) => Some(t.clone()),
                _ => None
            }
        ),
        time_limit: attrs.iter().find(|pair| pair.0 == "TIME_LIMIT").and_then(|pair| 
            match pair.1 {
                GameAttribute::TimeLimit(ref t) => Some(t.clone()),
                _ => None
            }
        ),
        opening: attrs.iter().find(|pair| pair.0 == "OPENING").map(|pair| pair.1.to_string()),
        start_pos: Position{
            drop_pieces: drop_pieces.unwrap_or(vec![]),
            bulk,
            add_pieces: add_pieces.into_iter().flat_map(|c| c).collect(),
            side_to_move,
        },
        moves,
    })
));

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{NaiveDate, NaiveTime};
    use nom::IResult;

    #[test]
    fn parse_comment() {
        assert_eq!(comment(b"'this is a comment"), IResult::Done(&b""[..], &b"this is a comment"[..]));
    }

    #[test]
    fn parse_piece_type() {
        assert_eq!(piece_type(b"FU"), IResult::Done(&b""[..], PieceType::Pawn));
        assert_eq!(piece_type(b"KY"), IResult::Done(&b""[..], PieceType::Lance));
        assert_eq!(piece_type(b"KE"), IResult::Done(&b""[..], PieceType::Knight));
        assert_eq!(piece_type(b"GI"), IResult::Done(&b""[..], PieceType::Silver));
        assert_eq!(piece_type(b"KI"), IResult::Done(&b""[..], PieceType::Gold));
        assert_eq!(piece_type(b"KA"), IResult::Done(&b""[..], PieceType::Bishop));
        assert_eq!(piece_type(b"HI"), IResult::Done(&b""[..], PieceType::Rook));
        assert_eq!(piece_type(b"OU"), IResult::Done(&b""[..], PieceType::King));
        assert_eq!(piece_type(b"TO"), IResult::Done(&b""[..], PieceType::ProPawn));
        assert_eq!(piece_type(b"NY"), IResult::Done(&b""[..], PieceType::ProLance));
        assert_eq!(piece_type(b"NK"), IResult::Done(&b""[..], PieceType::ProKnight));
        assert_eq!(piece_type(b"NG"), IResult::Done(&b""[..], PieceType::ProSilver));
        assert_eq!(piece_type(b"UM"), IResult::Done(&b""[..], PieceType::Horse));
        assert_eq!(piece_type(b"RY"), IResult::Done(&b""[..], PieceType::Dragon));
    }

    #[test]
    fn parse_one_digit() {
        assert_eq!(one_digit(b"0"), IResult::Done(&b""[..], 0));
        assert_eq!(one_digit(b"1"), IResult::Done(&b""[..], 1));
        assert_eq!(one_digit(b"2"), IResult::Done(&b""[..], 2));
        assert_eq!(one_digit(b"3"), IResult::Done(&b""[..], 3));
        assert_eq!(one_digit(b"4"), IResult::Done(&b""[..], 4));
        assert_eq!(one_digit(b"5"), IResult::Done(&b""[..], 5));
        assert_eq!(one_digit(b"6"), IResult::Done(&b""[..], 6));
        assert_eq!(one_digit(b"7"), IResult::Done(&b""[..], 7));
        assert_eq!(one_digit(b"8"), IResult::Done(&b""[..], 8));
        assert_eq!(one_digit(b"9"), IResult::Done(&b""[..], 9));
        assert_eq!(one_digit(b"10"), IResult::Done(&b"0"[..], 1));
    }

    #[test]
    fn parse_square() {
        assert_eq!(square(b"00"), IResult::Done(&b""[..], Square::new(0, 0)));
        assert_eq!(square(b"99"), IResult::Done(&b""[..], Square::new(9, 9)));
    }

    #[test]
    fn parse_version() {
        assert_eq!(version(b"V2"), IResult::Done(&b""[..], &b"2"[..]));
        assert_eq!(version(b"V2.1"), IResult::Done(&b""[..], &b"2.1"[..]));
        assert_eq!(version(b"V2.2"), IResult::Done(&b""[..], &b"2.2"[..]));
    }

    #[test]
    fn parse_players() {
        assert_eq!(black_player(b"N+black player"), IResult::Done(&b""[..], &b"black player"[..]));
        assert_eq!(white_player(b"N-white player"), IResult::Done(&b""[..], &b"white player"[..]));
    }

    #[test]
    fn parse_game_attr() {
        assert_eq!(game_attr(b"$EVENT:event"), IResult::Done(&b""[..], (
            "EVENT".to_string(),
            GameAttribute::Str("event".to_string())
        )));
        assert_eq!(game_attr(b"$START_TIME:2002/01/01 19:00:00"), IResult::Done(&b""[..], (
            "START_TIME".to_string(),
            GameAttribute::Time(Time{
                date: NaiveDate::from_ymd(2002, 1, 1),
                time: Some(NaiveTime::from_hms(19, 0, 0))
            })
        )));
    }

    #[test]
    fn parse_handicap() {
        assert_eq!(handicap(b"PI"), IResult::Done(&b""[..], vec![]));
        assert_eq!(handicap(b"PI82HI22KA"), IResult::Done(&b""[..], vec![
            (Square::new(8, 2), PieceType::Rook), (Square::new(2, 2), PieceType::Bishop)
        ]));
    }

    #[test]
    fn parse_grid() {
        let grid_str = b"\
P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
P2 * -HI *  *  *  *  * -KA * 
P3-FU-FU-FU-FU-FU-FU-FU-FU-FU
P4 *  *  *  *  *  *  *  *  * 
P5 *  *  *  *  *  *  *  *  * 
P6 *  *  *  *  *  *  *  *  * 
P7+FU+FU+FU+FU+FU+FU+FU+FU+FU
P8 * +KA *  *  *  *  * +HI * 
P9+KY+KE+GI+KI+OU+KI+GI+KE+KY";

        let initial_pos = [
            [
                Some((Color::White, PieceType::Lance)),
                Some((Color::White, PieceType::Knight)),
                Some((Color::White, PieceType::Silver)),
                Some((Color::White, PieceType::Gold)),
                Some((Color::White, PieceType::King)),
                Some((Color::White, PieceType::Gold)),
                Some((Color::White, PieceType::Silver)),
                Some((Color::White, PieceType::Knight)),
                Some((Color::White, PieceType::Lance)),
            ],
            [
                None,
                Some((Color::White, PieceType::Rook)),
                None,
                None,
                None,
                None,
                None,
                Some((Color::White, PieceType::Bishop)),
                None,
            ],
            [
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
            ],
            [None, None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None, None],
            [
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
            ],
            [
                None,
                Some((Color::Black, PieceType::Bishop)),
                None,
                None,
                None,
                None,
                None,
                Some((Color::Black, PieceType::Rook)),
                None,
            ],
            [
                Some((Color::Black, PieceType::Lance)),
                Some((Color::Black, PieceType::Knight)),
                Some((Color::Black, PieceType::Silver)),
                Some((Color::Black, PieceType::Gold)),
                Some((Color::Black, PieceType::King)),
                Some((Color::Black, PieceType::Gold)),
                Some((Color::Black, PieceType::Silver)),
                Some((Color::Black, PieceType::Knight)),
                Some((Color::Black, PieceType::Lance)),
            ],
        ];

        assert_eq!(grid(grid_str), IResult::Done(&b""[..], initial_pos));
    }

    #[test]
    fn parse_piece_placement() {
        assert_eq!(piece_placement(b"P+99KY89KE"), IResult::Done(&b""[..], vec![
            (Color::Black, Square::new(9, 9), PieceType::Lance),
            (Color::Black, Square::new(8, 9), PieceType::Knight),
        ]));
        assert_eq!(piece_placement(b"P-00AL"), IResult::Done(&b""[..], vec![
            (Color::White, Square::new(0, 0), PieceType::All),
        ]));
    }

    #[test]
    fn parse_normal_move() {
        assert_eq!(normal_move(b"+2726FU"), IResult::Done(&b""[..],
            Action::Move(Color::Black, Square::new(2, 7), Square::new(2, 6), PieceType::Pawn)));
        assert_eq!(normal_move(b"-3334FU"), IResult::Done(&b""[..],
            Action::Move(Color::White, Square::new(3, 3), Square::new(3, 4), PieceType::Pawn)));
    }

    #[test]
    fn parse_special_move() {
        assert_eq!(special_move(b"%TORYO"), IResult::Done(&b""[..], Action::Toryo));
        assert_eq!(special_move(b"%MATTA"), IResult::Done(&b""[..], Action::Matta));
        assert_eq!(special_move(b"%TSUMI"), IResult::Done(&b""[..], Action::Tsumi));
        assert_eq!(special_move(b"%ERROR"), IResult::Done(&b""[..], Action::Error));
        assert_eq!(special_move(b"%KACHI"), IResult::Done(&b""[..], Action::Kachi));
        assert_eq!(special_move(b"%CHUDAN"), IResult::Done(&b""[..], Action::Chudan));
        assert_eq!(special_move(b"%FUZUMI"), IResult::Done(&b""[..], Action::Fuzumi));
        assert_eq!(special_move(b"%JISHOGI"), IResult::Done(&b""[..], Action::Jishogi));
        assert_eq!(special_move(b"%HIKIWAKE"), IResult::Done(&b""[..], Action::Hikiwake));
        assert_eq!(special_move(b"%SENNICHITE"), IResult::Done(&b""[..], Action::Sennichite));
    }

    #[test]
    fn parse_move_record() {
        assert_eq!(move_record(b"+2726FU\nT5"), IResult::Done(&b""[..],
            MoveRecord{action: Action::Move(Color::Black, Square::new(2, 7), Square::new(2, 6), PieceType::Pawn),
                       time: Some(Duration::from_secs(5))}));
        assert_eq!(move_record(b"+2726FU"), IResult::Done(&b""[..],
            MoveRecord{action: Action::Move(Color::Black, Square::new(2, 7), Square::new(2, 6), PieceType::Pawn),
                       time: None}));

        assert_eq!(move_record(b"%TORYO\nT5"), IResult::Done(&b""[..],
            MoveRecord{action: Action::Toryo,
                       time: Some(Duration::from_secs(5))}));
        assert_eq!(move_record(b"%TORYO"), IResult::Done(&b""[..],
            MoveRecord{action: Action::Toryo,
                       time: None}));
    }

    #[test]
    fn parse_game_record() {
        let csa = "\
'----------棋譜ファイルの例\"example.csa\"-----------------
'バージョン
V2.2
'対局者名
N+NAKAHARA
N-YONENAGA
'棋譜情報
'棋戦名
$EVENT:13th World Computer Shogi Championship
'対局場所
$SITE:KAZUSA ARC
'開始日時
$START_TIME:2003/05/03 10:30:00
'終了日時
$END_TIME:2003/05/03 11:11:05
'持ち時間:25分、切れ負け
$TIME_LIMIT:00:25+00
'戦型:矢倉
$OPENING:YAGURA
'平手の局面
P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
P2 * -HI *  *  *  *  * -KA * 
P3-FU-FU-FU-FU-FU-FU-FU-FU-FU
P4 *  *  *  *  *  *  *  *  * 
P5 *  *  *  *  *  *  *  *  * 
P6 *  *  *  *  *  *  *  *  * 
P7+FU+FU+FU+FU+FU+FU+FU+FU+FU
P8 * +KA *  *  *  *  * +HI * 
P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
'先手番
+
'指し手と消費時間
+2726FU
T12
-3334FU
T6
%CHUDAN
'---------------------------------------------------------
";

        let initial_pos = [
            [
                Some((Color::White, PieceType::Lance)),
                Some((Color::White, PieceType::Knight)),
                Some((Color::White, PieceType::Silver)),
                Some((Color::White, PieceType::Gold)),
                Some((Color::White, PieceType::King)),
                Some((Color::White, PieceType::Gold)),
                Some((Color::White, PieceType::Silver)),
                Some((Color::White, PieceType::Knight)),
                Some((Color::White, PieceType::Lance)),
            ],
            [
                None,
                Some((Color::White, PieceType::Rook)),
                None,
                None,
                None,
                None,
                None,
                Some((Color::White, PieceType::Bishop)),
                None,
            ],
            [
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
                Some((Color::White, PieceType::Pawn)),
            ],
            [None, None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None, None],
            [None, None, None, None, None, None, None, None, None],
            [
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
                Some((Color::Black, PieceType::Pawn)),
            ],
            [
                None,
                Some((Color::Black, PieceType::Bishop)),
                None,
                None,
                None,
                None,
                None,
                Some((Color::Black, PieceType::Rook)),
                None,
            ],
            [
                Some((Color::Black, PieceType::Lance)),
                Some((Color::Black, PieceType::Knight)),
                Some((Color::Black, PieceType::Silver)),
                Some((Color::Black, PieceType::Gold)),
                Some((Color::Black, PieceType::King)),
                Some((Color::Black, PieceType::Gold)),
                Some((Color::Black, PieceType::Silver)),
                Some((Color::Black, PieceType::Knight)),
                Some((Color::Black, PieceType::Lance)),
            ],
        ];

        assert_eq!(game_record(csa.as_bytes()), IResult::Done(&b""[..], GameRecord{
             black_player: Some("NAKAHARA".to_string()),
             white_player: Some("YONENAGA".to_string()),
             event: Some("13th World Computer Shogi Championship".to_string()),
             site: Some("KAZUSA ARC".to_string()),
             start_time: Some(Time{
                date: NaiveDate::from_ymd(2003, 5, 3),
                time: Some(NaiveTime::from_hms(10, 30, 0))
            }),
             end_time: Some(Time{
                date: NaiveDate::from_ymd(2003, 5, 3),
                time: Some(NaiveTime::from_hms(11, 11, 5))
            }),
             time_limit: Some(TimeLimit{
                main_time: Duration::from_secs(1500),
                byoyomi: Duration::from_secs(0)
            }),
             opening: Some("YAGURA".to_string()),
             start_pos: Position{
                 drop_pieces: vec![],
                 bulk: Some(initial_pos),
                 add_pieces: vec![],
                 side_to_move: Color::Black,
             },
             moves: vec![
                 MoveRecord{action: Action::Move(Color::Black, Square::new(2, 7), Square::new(2, 6), PieceType::Pawn),
                            time: Some(Duration::from_secs(12))},
                 MoveRecord{action: Action::Move(Color::White, Square::new(3, 3), Square::new(3, 4), PieceType::Pawn),
                            time: Some(Duration::from_secs(6))},
                MoveRecord{action: Action::Chudan,
                           time: None}
            ],
        }))
    }
}
