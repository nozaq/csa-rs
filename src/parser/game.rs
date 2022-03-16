use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, tag, take};
use nom::character::complete::{anychar, digit1, one_of};
use nom::combinator::{map, map_res, opt, value};
use nom::multi::{count, many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::*;
use std::str;
use std::time::Duration;

use super::time::{datetime, timelimit};
use crate::value::*;

fn line_sep(input: &[u8]) -> IResult<&[u8], &[u8]> {
    is_a("\r\n,")(input)
}

fn not_line_sep(input: &[u8]) -> IResult<&[u8], &[u8]> {
    is_not("\r\n,")(input)
}

fn comment(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag("'"), not_line_sep)(input)
}

fn comment_line(input: &[u8]) -> IResult<&[u8], &[u8]> {
    terminated(comment, line_sep)(input)
}

fn color(input: &[u8]) -> IResult<&[u8], Color> {
    map(one_of("+-"), |s| match s {
        '+' => Color::Black,
        _ => Color::White,
    })(input)
}

fn decimal(input: &[u8]) -> IResult<&[u8], Duration> {
    map_res(digit1, |s| {
        str::from_utf8(s)
            .map(|s| s.parse::<u64>().unwrap())
            .map(Duration::from_secs)
    })(input)
}

fn fu(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Pawn, tag("FU"))(input)
}

fn ky(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Lance, tag("KY"))(input)
}

fn ke(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Knight, tag("KE"))(input)
}

fn gi(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Silver, tag("GI"))(input)
}

fn ki(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Gold, tag("KI"))(input)
}

fn ka(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Bishop, tag("KA"))(input)
}

fn hi(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Rook, tag("HI"))(input)
}

fn ou(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::King, tag("OU"))(input)
}

fn to(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::ProPawn, tag("TO"))(input)
}

fn ny(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::ProLance, tag("NY"))(input)
}

fn nk(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::ProKnight, tag("NK"))(input)
}

fn ng(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::ProSilver, tag("NG"))(input)
}

fn um(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Horse, tag("UM"))(input)
}

fn ry(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::Dragon, tag("RY"))(input)
}

fn al(input: &[u8]) -> IResult<&[u8], PieceType> {
    value(PieceType::All, tag("AL"))(input)
}

fn piece_type(input: &[u8]) -> IResult<&[u8], PieceType> {
    alt((fu, ky, ke, gi, ki, ka, hi, ou, to, ny, nk, ng, um, ry, al))(input)
}

fn one_digit(input: &[u8]) -> IResult<&[u8], u8> {
    map(one_of("0123456789"), |c: char| {
        c.to_digit(10).unwrap() as u8
    })(input)
}

fn square(input: &[u8]) -> IResult<&[u8], Square> {
    map(tuple((one_digit, one_digit)), |(file, rank)| {
        Square::new(file, rank)
    })(input)
}

fn version(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag("V"), alt((tag("2.1"), tag("2.2"), tag("2"))))(input)
}

fn black_player(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag("N+"), not_line_sep)(input)
}

fn white_player(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag("N-"), not_line_sep)(input)
}

fn game_text_attr(input: &[u8]) -> IResult<&[u8], GameAttribute> {
    map(map_res(not_line_sep, str::from_utf8), |s: &str| {
        GameAttribute::Str(s.to_string())
    })(input)
}

fn game_time_attr(input: &[u8]) -> IResult<&[u8], GameAttribute> {
    map(datetime, GameAttribute::Time)(input)
}

fn game_timelimit_attr(input: &[u8]) -> IResult<&[u8], GameAttribute> {
    map(timelimit, GameAttribute::TimeLimit)(input)
}

fn game_attr(input: &[u8]) -> IResult<&[u8], (String, GameAttribute)> {
    preceded(
        tag("$"),
        separated_pair(
            map_res(is_not(":"), |s: &[u8]| String::from_utf8(s.to_vec())),
            tag(":"),
            alt((game_time_attr, game_timelimit_attr, game_text_attr)),
        ),
    )(input)
}

fn handicap(input: &[u8]) -> IResult<&[u8], Vec<(Square, PieceType)>> {
    preceded(tag("PI"), many0(tuple((square, piece_type))))(input)
}

fn grid_piece(input: &[u8]) -> IResult<&[u8], Option<(Color, PieceType)>> {
    let (input, result) = anychar(input)?;

    match result {
        '+' => map(piece_type, |pt| Some((Color::Black, pt)))(input),
        '-' => map(piece_type, |pt| Some((Color::White, pt)))(input),
        _ => value(None, take(2usize))(input),
    }
}

type GridRow = [Option<(Color, PieceType)>; 9];
type Grid = [GridRow; 9];

fn grid_row(input: &[u8]) -> IResult<&[u8], GridRow> {
    let (input, vec) = count(grid_piece, 9)(input)?;

    // TODO: Convert into an array instead of copying.
    let mut array = [None; 9];
    array.clone_from_slice(&vec);

    Ok((input, array))
}

fn grid(input: &[u8]) -> IResult<&[u8], Grid> {
    let (input, r1) = delimited(tag("P1"), grid_row, line_sep)(input)?;
    let (input, r2) = delimited(tag("P2"), grid_row, line_sep)(input)?;
    let (input, r3) = delimited(tag("P3"), grid_row, line_sep)(input)?;
    let (input, r4) = delimited(tag("P4"), grid_row, line_sep)(input)?;
    let (input, r5) = delimited(tag("P5"), grid_row, line_sep)(input)?;
    let (input, r6) = delimited(tag("P6"), grid_row, line_sep)(input)?;
    let (input, r7) = delimited(tag("P7"), grid_row, line_sep)(input)?;
    let (input, r8) = delimited(tag("P8"), grid_row, line_sep)(input)?;
    let (input, r9) = preceded(tag("P9"), grid_row)(input)?;

    Ok((input, [r1, r2, r3, r4, r5, r6, r7, r8, r9]))
}

fn piece_placement(input: &[u8]) -> IResult<&[u8], Vec<(Color, Square, PieceType)>> {
    let (input, _) = tag("P")(input)?;
    let (input, c) = color(input)?;
    let (input, pcs) = many0(tuple((square, piece_type)))(input)?;

    Ok((
        input,
        pcs.iter().map(|&(sq, pt)| (c, sq, pt)).collect::<Vec<_>>(),
    ))
}

fn normal_move(input: &[u8]) -> IResult<&[u8], Action> {
    let (input, c) = color(input)?;
    let (input, from) = square(input)?;
    let (input, to) = square(input)?;
    let (input, pt) = piece_type(input)?;

    Ok((input, Action::Move(c, from, to, pt)))
}

fn special_move(input: &[u8]) -> IResult<&[u8], Action> {
    preceded(
        tag("%"),
        alt((
            value(Action::Toryo, tag("TORYO")),
            value(Action::Matta, tag("MATTA")),
            value(Action::Tsumi, tag("TSUMI")),
            value(Action::Error, tag("ERROR")),
            value(Action::Kachi, tag("KACHI")),
            value(Action::Chudan, tag("CHUDAN")),
            value(Action::Fuzumi, tag("FUZUMI")),
            value(Action::Jishogi, tag("JISHOGI")),
            value(Action::Hikiwake, tag("HIKIWAKE")),
            value(Action::Sennichite, tag("SENNICHITE")),
        )),
    )(input)
}

fn move_record(input: &[u8]) -> IResult<&[u8], MoveRecord> {
    let (input, action) = alt((normal_move, special_move))(input)?;
    let (input, time) = opt(preceded(line_sep, preceded(tag("T"), decimal)))(input)?;

    Ok((input, MoveRecord { action, time }))
}

fn move_records(input: &[u8]) -> IResult<&[u8], Vec<MoveRecord>> {
    let (input, moves) = many0(map(
        pair(terminated(move_record, line_sep), many0(comment_line)),
        |(m, _)| m,
    ))(input)?;

    Ok((input, moves))
}

pub fn game_record(input: &[u8]) -> IResult<&[u8], GameRecord> {
    let (input, _) = many0(comment_line)(input)?;
    let (input, _) = opt(terminated(version, line_sep))(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, black_player) = opt(map_res(terminated(black_player, line_sep), |b| {
        str::from_utf8(b)
    }))(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, white_player) = opt(map_res(terminated(white_player, line_sep), |b| {
        str::from_utf8(b)
    }))(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, attrs) = map(
        opt(terminated(
            separated_list0(line_sep, preceded(many0(comment_line), game_attr)),
            line_sep,
        )),
        |v: Option<Vec<(String, GameAttribute)>>| v.unwrap_or_default(),
    )(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, drop_pieces) = opt(terminated(handicap, line_sep))(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, bulk) = opt(terminated(grid, line_sep))(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, add_pieces) = many0(terminated(piece_placement, line_sep))(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, side_to_move) = terminated(color, line_sep)(input)?;
    let (input, _) = many0(comment_line)(input)?;
    let (input, moves) = move_records(input)?;

    Ok((
        input,
        GameRecord {
            black_player: black_player.map(|s| s.to_string()),
            white_player: white_player.map(|s| s.to_string()),
            event: attrs
                .iter()
                .find(|pair| pair.0 == "EVENT")
                .map(|pair| pair.1.to_string()),
            site: attrs
                .iter()
                .find(|pair| pair.0 == "SITE")
                .map(|pair| pair.1.to_string()),
            start_time: attrs.iter().find(|pair| pair.0 == "START_TIME").and_then(
                |pair| match pair.1 {
                    GameAttribute::Time(ref t) => Some(t.clone()),
                    _ => None,
                },
            ),
            end_time: attrs
                .iter()
                .find(|pair| pair.0 == "END_TIME")
                .and_then(|pair| match pair.1 {
                    GameAttribute::Time(ref t) => Some(t.clone()),
                    _ => None,
                }),
            time_limit: attrs.iter().find(|pair| pair.0 == "TIME_LIMIT").and_then(
                |pair| match pair.1 {
                    GameAttribute::TimeLimit(ref t) => Some(t.clone()),
                    _ => None,
                },
            ),
            opening: attrs
                .iter()
                .find(|pair| pair.0 == "OPENING")
                .map(|pair| pair.1.to_string()),
            start_pos: Position {
                drop_pieces: drop_pieces.unwrap_or_default(),
                bulk,
                add_pieces: add_pieces.into_iter().flatten().collect(),
                side_to_move,
            },
            moves,
        },
    ))
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    use time::{Date as NativeDate, Time as NativeTime};

    #[test]
    fn parse_comment() {
        assert_eq!(
            comment(b"'this is a comment"),
            Result::Ok((&b""[..], &b"this is a comment"[..]))
        );
    }

    #[test]
    fn parse_piece_type() {
        assert_eq!(piece_type(b"FU"), Result::Ok((&b""[..], PieceType::Pawn)));
        assert_eq!(piece_type(b"KY"), Result::Ok((&b""[..], PieceType::Lance)));
        assert_eq!(piece_type(b"KE"), Result::Ok((&b""[..], PieceType::Knight)));
        assert_eq!(piece_type(b"GI"), Result::Ok((&b""[..], PieceType::Silver)));
        assert_eq!(piece_type(b"KI"), Result::Ok((&b""[..], PieceType::Gold)));
        assert_eq!(piece_type(b"KA"), Result::Ok((&b""[..], PieceType::Bishop)));
        assert_eq!(piece_type(b"HI"), Result::Ok((&b""[..], PieceType::Rook)));
        assert_eq!(piece_type(b"OU"), Result::Ok((&b""[..], PieceType::King)));
        assert_eq!(
            piece_type(b"TO"),
            Result::Ok((&b""[..], PieceType::ProPawn))
        );
        assert_eq!(
            piece_type(b"NY"),
            Result::Ok((&b""[..], PieceType::ProLance))
        );
        assert_eq!(
            piece_type(b"NK"),
            Result::Ok((&b""[..], PieceType::ProKnight))
        );
        assert_eq!(
            piece_type(b"NG"),
            Result::Ok((&b""[..], PieceType::ProSilver))
        );
        assert_eq!(piece_type(b"UM"), Result::Ok((&b""[..], PieceType::Horse)));
        assert_eq!(piece_type(b"RY"), Result::Ok((&b""[..], PieceType::Dragon)));
    }

    #[test]
    fn parse_one_digit() {
        assert_eq!(one_digit(b"0"), Result::Ok((&b""[..], 0)));
        assert_eq!(one_digit(b"1"), Result::Ok((&b""[..], 1)));
        assert_eq!(one_digit(b"2"), Result::Ok((&b""[..], 2)));
        assert_eq!(one_digit(b"3"), Result::Ok((&b""[..], 3)));
        assert_eq!(one_digit(b"4"), Result::Ok((&b""[..], 4)));
        assert_eq!(one_digit(b"5"), Result::Ok((&b""[..], 5)));
        assert_eq!(one_digit(b"6"), Result::Ok((&b""[..], 6)));
        assert_eq!(one_digit(b"7"), Result::Ok((&b""[..], 7)));
        assert_eq!(one_digit(b"8"), Result::Ok((&b""[..], 8)));
        assert_eq!(one_digit(b"9"), Result::Ok((&b""[..], 9)));
        assert_eq!(one_digit(b"10"), Result::Ok((&b"0"[..], 1)));
    }

    #[test]
    fn parse_square() {
        assert_eq!(square(b"00"), Result::Ok((&b""[..], Square::new(0, 0))));
        assert_eq!(square(b"99"), Result::Ok((&b""[..], Square::new(9, 9))));
    }

    #[test]
    fn parse_version() {
        assert_eq!(version(b"V2"), Result::Ok((&b""[..], &b"2"[..])));
        assert_eq!(version(b"V2.1"), Result::Ok((&b""[..], &b"2.1"[..])));
        assert_eq!(version(b"V2.2"), Result::Ok((&b""[..], &b"2.2"[..])));
    }

    #[test]
    fn parse_players() {
        assert_eq!(
            black_player(b"N+black player"),
            Result::Ok((&b""[..], &b"black player"[..]))
        );
        assert_eq!(
            white_player(b"N-white player"),
            Result::Ok((&b""[..], &b"white player"[..]))
        );
    }

    #[test]
    fn parse_game_attr() {
        assert_eq!(
            game_attr(b"$EVENT:event"),
            Result::Ok((
                &b""[..],
                ("EVENT".to_string(), GameAttribute::Str("event".to_string()))
            ))
        );
        assert_eq!(
            game_attr(b"$START_TIME:2002/01/01 19:00:00"),
            Result::Ok((
                &b""[..],
                (
                    "START_TIME".to_string(),
                    GameAttribute::Time(Time {
                        date: NativeDate::from_calendar_date(2002, time::Month::January, 1)
                            .unwrap(),
                        time: Some(NativeTime::from_hms(19, 0, 0).unwrap())
                    })
                )
            ))
        );
    }

    #[test]
    fn parse_handicap() {
        assert_eq!(handicap(b"PI"), Result::Ok((&b""[..], vec![])));
        assert_eq!(
            handicap(b"PI82HI22KA"),
            Result::Ok((
                &b""[..],
                vec![
                    (Square::new(8, 2), PieceType::Rook),
                    (Square::new(2, 2), PieceType::Bishop)
                ]
            ))
        );
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

        assert_eq!(grid(grid_str), Result::Ok((&b""[..], initial_pos)));
    }

    #[test]
    fn parse_piece_placement() {
        assert_eq!(
            piece_placement(b"P+99KY89KE"),
            Result::Ok((
                &b""[..],
                vec![
                    (Color::Black, Square::new(9, 9), PieceType::Lance),
                    (Color::Black, Square::new(8, 9), PieceType::Knight),
                ]
            ))
        );
        assert_eq!(
            piece_placement(b"P-00AL"),
            Result::Ok((
                &b""[..],
                vec![(Color::White, Square::new(0, 0), PieceType::All),]
            ))
        );
    }

    #[test]
    fn parse_normal_move() {
        assert_eq!(
            normal_move(b"+2726FU"),
            Result::Ok((
                &b""[..],
                Action::Move(
                    Color::Black,
                    Square::new(2, 7),
                    Square::new(2, 6),
                    PieceType::Pawn
                )
            ))
        );
        assert_eq!(
            normal_move(b"-3334FU"),
            Result::Ok((
                &b""[..],
                Action::Move(
                    Color::White,
                    Square::new(3, 3),
                    Square::new(3, 4),
                    PieceType::Pawn
                )
            ))
        );
    }

    #[test]
    fn parse_special_move() {
        assert_eq!(
            special_move(b"%TORYO"),
            Result::Ok((&b""[..], Action::Toryo))
        );
        assert_eq!(
            special_move(b"%MATTA"),
            Result::Ok((&b""[..], Action::Matta))
        );
        assert_eq!(
            special_move(b"%TSUMI"),
            Result::Ok((&b""[..], Action::Tsumi))
        );
        assert_eq!(
            special_move(b"%ERROR"),
            Result::Ok((&b""[..], Action::Error))
        );
        assert_eq!(
            special_move(b"%KACHI"),
            Result::Ok((&b""[..], Action::Kachi))
        );
        assert_eq!(
            special_move(b"%CHUDAN"),
            Result::Ok((&b""[..], Action::Chudan))
        );
        assert_eq!(
            special_move(b"%FUZUMI"),
            Result::Ok((&b""[..], Action::Fuzumi))
        );
        assert_eq!(
            special_move(b"%JISHOGI"),
            Result::Ok((&b""[..], Action::Jishogi))
        );
        assert_eq!(
            special_move(b"%HIKIWAKE"),
            Result::Ok((&b""[..], Action::Hikiwake))
        );
        assert_eq!(
            special_move(b"%SENNICHITE"),
            Result::Ok((&b""[..], Action::Sennichite))
        );
    }

    #[test]
    fn parse_move_record() {
        assert_eq!(
            move_record(b"+2726FU\nT5"),
            Result::Ok((
                &b""[..],
                MoveRecord {
                    action: Action::Move(
                        Color::Black,
                        Square::new(2, 7),
                        Square::new(2, 6),
                        PieceType::Pawn
                    ),
                    time: Some(Duration::from_secs(5))
                }
            ))
        );
        assert_eq!(
            move_record(b"+2726FU"),
            Result::Ok((
                &b""[..],
                MoveRecord {
                    action: Action::Move(
                        Color::Black,
                        Square::new(2, 7),
                        Square::new(2, 6),
                        PieceType::Pawn
                    ),
                    time: None
                }
            ))
        );

        assert_eq!(
            move_record(b"%TORYO\nT5"),
            Result::Ok((
                &b""[..],
                MoveRecord {
                    action: Action::Toryo,
                    time: Some(Duration::from_secs(5))
                }
            ))
        );
        assert_eq!(
            move_record(b"%TORYO"),
            Result::Ok((
                &b""[..],
                MoveRecord {
                    action: Action::Toryo,
                    time: None
                }
            ))
        );
    }

    #[test]
    fn parse_move_records() {
        let records = b"\
+7776FU
'** 30 -3334FU +2726FU
-3334FU
T5
'*jouseki
+2726FU
";
        assert_eq!(
            move_records(records),
            Result::Ok((
                &b""[..],
                vec![
                    MoveRecord {
                        action: Action::Move(
                            Color::Black,
                            Square::new(7, 7),
                            Square::new(7, 6),
                            PieceType::Pawn
                        ),
                        time: None,
                    },
                    MoveRecord {
                        action: Action::Move(
                            Color::White,
                            Square::new(3, 3),
                            Square::new(3, 4),
                            PieceType::Pawn
                        ),
                        time: Some(Duration::from_secs(5)),
                    },
                    MoveRecord {
                        action: Action::Move(
                            Color::Black,
                            Square::new(2, 7),
                            Square::new(2, 6),
                            PieceType::Pawn
                        ),
                        time: None,
                    },
                ]
            ))
        );
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

        assert_eq!(
            game_record(csa.as_bytes()),
            Result::Ok((
                &b""[..],
                GameRecord {
                    black_player: Some("NAKAHARA".to_string()),
                    white_player: Some("YONENAGA".to_string()),
                    event: Some("13th World Computer Shogi Championship".to_string()),
                    site: Some("KAZUSA ARC".to_string()),
                    start_time: Some(Time {
                        date: NativeDate::from_calendar_date(2003, time::Month::May, 3).unwrap(),
                        time: Some(NativeTime::from_hms(10, 30, 0).unwrap())
                    }),
                    end_time: Some(Time {
                        date: NativeDate::from_calendar_date(2003, time::Month::May, 3).unwrap(),
                        time: Some(NativeTime::from_hms(11, 11, 5).unwrap())
                    }),
                    time_limit: Some(TimeLimit {
                        main_time: Duration::from_secs(1500),
                        byoyomi: Duration::from_secs(0)
                    }),
                    opening: Some("YAGURA".to_string()),
                    start_pos: Position {
                        drop_pieces: vec![],
                        bulk: Some(initial_pos),
                        add_pieces: vec![],
                        side_to_move: Color::Black,
                    },
                    moves: vec![
                        MoveRecord {
                            action: Action::Move(
                                Color::Black,
                                Square::new(2, 7),
                                Square::new(2, 6),
                                PieceType::Pawn
                            ),
                            time: Some(Duration::from_secs(12))
                        },
                        MoveRecord {
                            action: Action::Move(
                                Color::White,
                                Square::new(3, 3),
                                Square::new(3, 4),
                                PieceType::Pawn
                            ),
                            time: Some(Duration::from_secs(6))
                        },
                        MoveRecord {
                            action: Action::Chudan,
                            time: None
                        }
                    ],
                }
            ))
        )
    }
}
