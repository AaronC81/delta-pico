#include "hardware.hpp"

#define I ButtonInput

#ifdef DELTA_PICO_PROTOTYPE
const ButtonInput buttonMapping[7][7] = {
  { I::None,      I::MoveUp,    I::None,      I::None,      I::None,      I::None,      I::None, },
  { I::MoveLeft,  I::None,      I::MoveRight, I::None,      I::None,      I::None,      I::None, },
  { I::None,      I::MoveDown,  I::None,      I::None,      I::None,      I::None,      I::None, },
  { I::Digit7,    I::Digit8,    I::Digit9,    I::Delete,    I::None,      I::None,      I::None, },
  { I::Digit4,    I::Digit5,    I::Digit6,    I::Multiply,  I::Fraction,  I::None,      I::None, },
  { I::Digit1,    I::Digit2,    I::Digit3,    I::Add,       I::Subtract,  I::None,      I::None, },
  { I::Digit0,    I::Point,     I::None,      I::None,      I::None,      I::None,      I::None, },
};
#endif

#ifdef DELTA_PICO_REV1
const ButtonInput buttonMapping[7][7] = {
  { I::MoveUp, I::MoveRight, I::Menu, I::List, I::None, I::None, I::None, },
  { I::MoveLeft, I::MoveDown, I::Shift, I::LeftParen, I::None, I::None, I::RightParen, },
  { I::Digit7, I::Digit8, I::Digit9, I::Delete, I::None, I::None, I::None, },
  { I::Digit4, I::Digit5, I::Digit6, I::Multiply, I::None, I::None, I::Fraction, },
  { I::None, I::None, I::None, I::None, I::None, I::None, I::None, },
  { I::Digit0, I::Point, I::None, I::None, I::None, I::None, I::Exe, },
  { I::Digit1, I::Digit2, I::Digit3, I::Add, I::None, I::None, I::Subtract, },
};
#endif

#undef I

