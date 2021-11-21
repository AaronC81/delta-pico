use core::cell::RefCell;

use alloc::{boxed::Box, format, rc::Rc, string::{String, ToString}, vec::Vec, vec};
use rbop::{UnstructuredNodeList, nav::NavPath, node::unstructured::{UnstructuredNodeRoot, Upgradable}, render::{Area, Renderer, Viewport}};
use rust_decimal::prelude::ToPrimitive;
use rand::{self, SeedableRng, Rng};

use crate::{interface::{ButtonInput, Colour, ShapeFill}, operating_system::{OSInput, os}, rbop_impl::{RbopContext}};
use super::{Application, ApplicationInfo};
use crate::interface::framework;

pub struct NumbersGame {
    score: u64,
    board: [[Rc<RefCell<Tile>>; 4]; 4], // Row, then column
    rng: rand::StdRng,
    game_over: bool,
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
enum Tile {
    Blank,
    Filled(u64),
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

macro_rules! blank {
    () => {
        Rc::new(RefCell::new(Tile::Blank))
    };
}

impl Application for NumbersGame {
    fn info() -> ApplicationInfo {
        ApplicationInfo {
            name: "2048".into(),
            visible: true,
        }
    }

    fn new() -> Self where Self: Sized { Self {
        score: 0,
        board: [
            [blank!(), blank!(), blank!(), Rc::new(RefCell::new(Tile::Filled(2)))],
            [blank!(), blank!(), blank!(), blank!()],
            [blank!(), Rc::new(RefCell::new(Tile::Filled(2))), blank!(), blank!()],
            [blank!(), blank!(), blank!(), blank!()],
        ],
        rng: rand::StdRng::from_seed(&[1, 2, 3, 4]), // TODO
        game_over: false,
    } }

    fn tick(&mut self) {
        framework().display.fill_screen(Colour::BLACK);

        os().ui_draw_title("Numbers Game");

        let padding = 10 as i64;
        let tile_size = (framework().display.width as i64 - 5 * padding) as i64 / 4;

        let mut y = 50;
        
        for row in self.board.iter() {
            let mut x = padding;

            for item in row.iter() {
                let (text, colour) = match *item.borrow() {
                    Tile::Blank => ("".into(), Colour::GREY),
                    Tile::Filled(n) => (n.to_string(), Colour::ORANGE),
                };

                framework().display.draw_rect(x, y, tile_size, tile_size, colour, ShapeFill::Filled, 4);

                framework().display.print_centred(x, y + tile_size / 3, tile_size, format!("  {}  ", text));

                x += tile_size + padding;
            }

            y += tile_size + padding;
        }

        framework().display.print_at(10, 285, format!("{}{}", self.score, if self.game_over {
            " | [EXE] Restart"
        } else { "" }));

        (framework().display.draw)();

        if let Some(input) = framework().buttons.wait_press() {
            if input == OSInput::Exe {
                os().restart_application();
            }

            if !self.game_over {
                match input {
                    OSInput::MoveDown => self.take_turn(Direction::Down),
                    OSInput::MoveUp => self.take_turn(Direction::Up),
                    OSInput::MoveLeft => self.take_turn(Direction::Left),
                    OSInput::MoveRight => self.take_turn(Direction::Right),
                    _ => (),
                }
            }
        };
    }
}

impl NumbersGame {
    fn take_turn(&mut self, direction: Direction) {
        // Movement
        self.score += self.move_tiles(direction);

        // Spawn new tile
        let mut blank_tiles: Vec<(usize, usize)> = vec![];
        for row in 0..4 {
            for col in 0..4 {
                if *self.board[row][col].borrow() == Tile::Blank {
                    blank_tiles.push((row, col));
                }
            }
        }

        if blank_tiles.is_empty() {
            self.game_over = true;
        } else {
            let (row_spawn, col_spawn) = *self.rng.choose(&blank_tiles[..]).unwrap();
            self.board[row_spawn][col_spawn] = Rc::new(RefCell::new(Tile::Filled(2)));
        }
    }

    fn move_tiles(&mut self, direction: Direction) -> u64 {
        use Direction::*;

        let mut score = 0;

        for i in 0..4 {
            score += match direction {
                Up => {
                    Self::move_1d_tiles(
                        &mut self.board[0][i].borrow_mut(),
                        &mut self.board[1][i].borrow_mut(),
                        &mut self.board[2][i].borrow_mut(),
                        &mut self.board[3][i].borrow_mut(),
                    )
                },

                Down => {
                    Self::move_1d_tiles(
                        &mut self.board[3][i].borrow_mut(),
                        &mut self.board[2][i].borrow_mut(),
                        &mut self.board[1][i].borrow_mut(),
                        &mut self.board[0][i].borrow_mut(),
                    )
                },

                Left => {
                    Self::move_1d_tiles(
                        &mut self.board[i][0].borrow_mut(),
                        &mut self.board[i][1].borrow_mut(),
                        &mut self.board[i][2].borrow_mut(),
                        &mut self.board[i][3].borrow_mut(),
                    )
                },

                Right => {
                    Self::move_1d_tiles(
                        &mut self.board[i][3].borrow_mut(),
                        &mut self.board[i][2].borrow_mut(),
                        &mut self.board[i][1].borrow_mut(),
                        &mut self.board[i][0].borrow_mut(),
                    )
                },
            }
        }

        score
    }

    /// Performs a movement along one row or column. Returns any score gained from the move.
    /// 
    /// t1 is the tile which is "the most squished" in the direction being moved to.
    /// For example, if moving left: 
    ///
    /// |----|----|----|----|
    /// | t1 | t2 | t3 | t4 |
    /// |----|----|----|----|
    ///
    /// Or moving down:
    ///
    /// |----|
    /// | t4 |
    /// |----|
    /// | t3 |
    /// |----|
    /// | t2 |
    /// |----|
    /// | t1 |
    /// |----|
    ///
    fn move_1d_tiles(t1: &mut Tile, t2: &mut Tile, t3: &mut Tile, t4: &mut Tile) -> u64 {
        // For 4-item rows/columns, there are few enough permutations that it's just easier to do
        // this, than figure out how to implement them in an array fashion! (He says, at 11pm)
        // (If you Command+Shift+P > Rust Analyzer: Toggle Inlay Hints, the match arms nicely
        //  line up!)

        use Tile::*;

        match (&t1, &t2, &t3, &t4) {
            // All blank, or one item but it's already in the right place - nothing to do!
            (_, Blank, Blank, Blank) => 0,

            // Only one item - move it along
              (Blank, t@Filled(_), Blank,       Blank      )
            | (Blank, Blank,       t@Filled(_), Blank      )
            | (Blank, Blank,       Blank,       t@Filled(_))
            => {
                *t1 = **t;
                *t2 = Blank;
                *t3 = Blank;
                *t4 = Blank;

                0
            }

            // Two items, where there is NO ROOM for a possible third on the less-squished side - we
            // may be able to merge them
              (a@Filled(an), Blank,        Blank,        b@Filled(bn))
            | (Blank       , a@Filled(an), Blank,        b@Filled(bn))
            | (Blank       , Blank,        a@Filled(an), b@Filled(bn))
            => {
                if an == bn {
                    // If they are equal, they can be merged!
                    let merged = an * 2;

                    *t1 = Filled(merged);
                    *t2 = Blank;
                    *t3 = Blank;
                    *t4 = Blank;

                    merged
                } else {
                    // They are not equal, so cannot be merged - move them along
                    *t1 = **a;
                    *t2 = **b;
                    *t3 = Blank;
                    *t4 = Blank;

                    0
                }
            }

            // TWO or THREE items, where if the third item is filled, it may be possible to merge
            // either adjacent pair
              (a@Filled(an), b@Filled(bn), c,            Blank)
            | (a@Filled(an), b@Filled(bn), Blank,        c    )
            | (a@Filled(an), Blank,        b@Filled(bn), c    )
            | (Blank       , a@Filled(an), b@Filled(bn), c    )
            => {
                if an == bn {
                    // If the first and second are equal, they can be merged!
                    // The third item stays intact
                    let merged = an * 2;

                    *t1 = Filled(merged);
                    *t2 = **c;
                    *t3 = Blank;
                    *t4 = Blank;

                    merged
                } else if let Filled(cn) = c {
                    if bn == cn {
                        // We can merge the second and third!
                        let merged = bn * 2;

                        *t1 = **a;
                        *t2 = Filled(merged);
                        *t3 = Blank;
                        *t4 = Blank;

                        merged
                    } else {
                        // They are not equal, so cannot be merged - move them along
                        *t1 = **a;
                        *t2 = **b;
                        *t3 = **c;
                        *t4 = Blank;

                        0
                    }
                } else {
                    // They are not equal, so cannot be merged - move them along
                    *t1 = **a;
                    *t2 = **b;
                    *t3 = **c;
                    *t4 = Blank;

                    0
                }
            }

            // FOUR items
            (a@Filled(an), b@Filled(bn), c@Filled(cn), d@Filled(dn)) => {
                // It is allowed for one of these cases to be true, and checked in this order:
                // - A and B are mergable, and C and D are mergable
                // - A and B are mergable
                // - B and C are mergable
                // - C and D are mergable

                let a_b_mergable = an == bn;
                let c_d_mergable = cn == dn;
                let b_c_mergable = bn == cn;

                if a_b_mergable && c_d_mergable {
                    // Awesome, merge them both!
                    let merged_first = an * 2;
                    let merged_second = cn * 2;

                    *t1 = Filled(merged_first);
                    *t2 = Filled(merged_second);
                    *t3 = Blank;
                    *t4 = Blank;

                    merged_first + merged_second
                } else if a_b_mergable {
                    // Merge A and B, move others along
                    let merged = an * 2;

                    *t1 = Filled(merged);
                    *t2 = **c;
                    *t3 = **d;
                    *t4 = Blank;

                    merged
                } else if b_c_mergable {
                    // Merge B and C, move along
                    let merged = bn * 2;

                    *t2 = Filled(merged);
                    *t3 = **d;
                    *t4 = Blank;

                    merged
                } else if c_d_mergable {
                    // Merge C and D, set new blank
                    let merged = cn * 2;

                    *t3 = Filled(merged);
                    *t4 = Blank;

                    merged
                } else {
                    // No movement at all is possible
                    0
                }
            }
        }
    }
}
