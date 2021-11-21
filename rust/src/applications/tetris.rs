use core::cell::RefCell;

use alloc::{format, rc::Rc, vec::Vec, vec};
use rand::{Rng, SeedableRng};

use crate::{interface::{ButtonEvent, ButtonInput, Colour, ShapeFill, framework}, operating_system::OSInput};

use super::{ApplicationInfo, real_time::{RealTimeApplication, RealTimeResult, RealTimeState}};

const PLAYFIELD_WIDTH: usize = 10;
const PLAYFIELD_HEIGHT: usize = 16;

type Playfield = [[Rc<RefCell<Tile>>; PLAYFIELD_WIDTH]; PLAYFIELD_HEIGHT];

pub struct TetrisApplication {
    real_time_state: RealTimeState<TetrisEvent>,
    playfield: Playfield,
    tetromino: Option<Tetromino>,
    rng: rand::StdRng,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum TetrominoShape {
    T,
    I,
    Z,
}

struct Tetromino {
    shape: TetrominoShape,
    colour: Colour,
    origin_x: usize,
    origin_y: usize,
}

impl Tetromino {
    fn mask(&self) -> Vec<Vec<bool>> {
        match self.shape {
            TetrominoShape::T => vec![
                vec![false, true, false],
                vec![true,  true, true ],
            ],
            TetrominoShape::I => vec![
                vec![true],
                vec![true],
                vec![true],
                vec![true],
            ],
            TetrominoShape::Z => vec![
                vec![true,  true, false],
                vec![false, true, true ],
            ]
        }
    }

    fn width(&self) -> usize {
        self.mask()[0].len()
    }

    fn height(&self) -> usize {
        self.mask().len()
    }

    fn touching_floor(&self, playfield: &Playfield) -> bool {
        // Are we touching the actual floor?
        if self.origin_y + self.height() >= PLAYFIELD_HEIGHT {
            return true;
        }

        // Does any part of the mask have a playfield block right underneath?
        let mask = self.mask();
        for x in self.origin_x..(self.origin_x + self.width()) {
            for y in self.origin_y..(self.origin_y + self.height()) {
                if mask[y - self.origin_y][x - self.origin_x] && y + 1 < PLAYFIELD_HEIGHT {
                    if let Tile::Filled(_) = *playfield[y + 1][x].borrow() {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn copy_to_playfield(&self, playfield: &mut Playfield) {
        let mask = self.mask();

        for x in self.origin_x..(self.origin_x + self.width()) {
            for y in self.origin_y..(self.origin_y + self.height()) {
                if mask[y - self.origin_y][x - self.origin_x] {
                    playfield[y][x] = Rc::new(RefCell::new(Tile::Filled(self.colour)));
                }
            }
        }
    }
}

enum Tile {
    Filled(Colour),
    Blank,
}

macro_rules! blank {
    () => {
        Rc::new(RefCell::new(Tile::Blank))
    };
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TetrisEvent {
    Left,
    Right,
    Slam,

    TickDown,
}

impl RealTimeApplication for TetrisApplication {
    real_time_boilerplate!(TetrisEvent);

    fn info() -> ApplicationInfo where Self: Sized {
        ApplicationInfo { name: "Tetris".into(), visible: true }
    }

    fn new() -> Self where Self: Sized {
        let mut new = Self {
            real_time_state: Default::default(),
            playfield: [
                // I am sorry
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
                [blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!(), blank!()],
            ],
            tetromino: Some(Tetromino {
                origin_x: 3,
                origin_y: 0,
                shape: TetrominoShape::T,
                colour: Colour::ORANGE,
            }),
            rng: rand::StdRng::from_seed(&[5, 6, 7, 8]) // TODO
        };

        new
            .on_input(OSInput::MoveLeft, TetrisEvent::Left)
            .on_input(OSInput::MoveRight, TetrisEvent::Right)
            .on_input(OSInput::MoveDown, TetrisEvent::Slam)
            .schedule(250, TetrisEvent::TickDown);

        new
    }

    fn on_event(&mut self, event: &Self::RealTimeEvent) -> RealTimeResult {
        match event {
            // TODO: movement bounds checks
            TetrisEvent::Left => {
                if let Some(tetromino) = &mut self.tetromino {
                    tetromino.origin_x -= 1;
                    RealTimeResult::Redraw
                } else {
                    RealTimeResult::None
                }
            }

            TetrisEvent::Right => {
                if let Some(tetromino) = &mut self.tetromino {
                    tetromino.origin_x += 1;
                    RealTimeResult::Redraw
                } else {
                    RealTimeResult::None
                }
            }

            TetrisEvent::Slam => todo!(),

            TetrisEvent::TickDown => {
                // TODO: deduplicate floor touch check
                if let Some(tetromino) = &mut self.tetromino {
                    if tetromino.touching_floor(&self.playfield) {
                        tetromino.copy_to_playfield(&mut self.playfield);
                        tetromino.origin_y = 0;
                        tetromino.origin_x = 3;
                    }

                    tetromino.origin_y += 1;

                    if tetromino.touching_floor(&self.playfield) {
                        tetromino.copy_to_playfield(&mut self.playfield);

                        tetromino.origin_y = 0;
                        tetromino.origin_x = 3;

                        tetromino.colour = *self.rng.choose(&[
                            Colour::BLUE,
                            Colour::ORANGE,
                            Colour::DARK_BLUE,
                            Colour::WHITE
                        ]).unwrap();
                        tetromino.shape = *self.rng.choose(&[
                            TetrominoShape::I,
                            TetrominoShape::T,
                            TetrominoShape::Z,
                        ]).unwrap();
                    }
                }

                self.schedule(250, TetrisEvent::TickDown);
                RealTimeResult::Redraw
            },
        }
    }

    fn draw(&mut self) {
        framework().display.fill_screen(Colour::BLACK);
        
        // Draw playfield
        let mut y = 0;
        for row in &self.playfield {
            let mut x = 0;
            for item in row {
                framework().display.draw_rect(x, y, 20, 20, match *item.borrow() {
                    Tile::Filled(c) => c,
                    Tile::Blank => Colour::GREY,
                }, ShapeFill::Filled, 0);
                x += 20;
            }
            y += 20;
        }

        // Draw tetromino
        if let Some(tetromino) = &self.tetromino {
            let mut y = 20 * tetromino.origin_y;
            for row in tetromino.mask() {
                let mut x = 20 * tetromino.origin_x;
                for item in row {
                    if item {
                        framework().display.draw_rect(x as i64, y as i64, 20, 20, tetromino.colour, ShapeFill::Filled, 0);
                    }
                    x += 20;
                }
                y += 20;
            }
        }

        (framework().display.draw)();
    }

}