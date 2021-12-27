# csa-rs

[![Github Actions](https://github.com/nozaq/csa-rs/workflows/build/badge.svg)](https://github.com/nozaq/csa-rs/actions?workflow=build)
[![Coverage Status](https://coveralls.io/repos/github/nozaq/csa-rs/badge.svg)](https://coveralls.io/github/nozaq/csa-rs)
[![crates.io](https://img.shields.io/crates/v/csa.svg)](https://crates.io/crates/csa)
[![docs.rs](https://docs.rs/csa/badge.svg)](https://docs.rs/csa)

A Shogi game serialization/deserialization library in CSA format.
CSA format is a plaintext format for recording Shogi games. This library supports parsing CSA-formatted string as well as composing CSA-formatted string from structs. Detail about CSA format is found at [here](http://www.computer-shogi.org/protocol/record_v22.html).

[Documentation](https://docs.rs/csa)

## Usage

Below is an example of parsing CSA-formatted string into structs.

```rust
use std::time::Duration;
use csa::{parse_csa, Action, Color, GameRecord, MoveRecord, PieceType, Square};

let csa_str = "\
V2.2
N+NAKAHARA
N-YONENAGA
$EVENT:13th World Computer Shogi Championship
PI
+
+2726FU
T12
";

let game = parse_csa(csa_str).expect("failed to parse the csa content");
assert_eq!(game.black_player, Some("NAKAHARA".to_string()));
assert_eq!(game.white_player, Some("YONENAGA".to_string()));
assert_eq!(game.event, Some("13th World Computer Shogi Championship".to_string()));
assert_eq!(game.moves[0],  MoveRecord{
    action: Action::Move(Color::Black, Square::new(2, 7), Square::new(2, 6), PieceType::Pawn),
    time: Some(Duration::from_secs(12))
});
```

In contrast, structs can be composed into CSA-formatted string.

```rust
use std::time::Duration;
use csa::{ Action, Color, GameRecord, MoveRecord, PieceType, Square};

let mut g = GameRecord::default();
g.black_player = Some("NAKAHARA".to_string());
g.white_player = Some("YONENAGA".to_string());
g.event = Some("13th World Computer Shogi Championship".to_string());
g.moves.push(MoveRecord {
    action: Action::Move(
        Color::Black,
        Square::new(2, 7),
        Square::new(2, 6),
        PieceType::Pawn,
    ),
    time: Some(Duration::from_secs(5)),
});
g.moves.push(MoveRecord {
    action: Action::Toryo,
    time: None,
});

let csa_str = "\
V2.2
N+NAKAHARA
N-YONENAGA
$EVENT:13th World Computer Shogi Championship
PI
+
+2726FU
T5
%TORYO
";

assert_eq!(csa_str, g.to_string());
```

## License

`csa-rs` is licensed under the MIT license. Please read the [LICENSE](LICENSE) file in this repository for more information.
