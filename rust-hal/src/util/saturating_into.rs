pub trait SaturatingInto<R> where Self: Sized {
    fn saturating_into(self) -> R;
}

impl SaturatingInto<u16> for u32 {
    fn saturating_into(self) -> u16 {
        if self > u16::MAX.into() { return u16::MAX }
        self as u16
    }
}

impl SaturatingInto<u16> for i32 {
    fn saturating_into(self) -> u16 {
        if self > u16::MAX.into() { return u16::MAX }
        if self < 0 { return 0 }
        self as u16
    }
}

impl SaturatingInto<u16> for i64 {
    fn saturating_into(self) -> u16 {
        if self > u16::MAX.into() { return u16::MAX }
        if self < 0 { return 0 }
        self as u16
    }
}

impl SaturatingInto<u32> for i64 {
    fn saturating_into(self) -> u32 {
        if self > u32::MAX.into() { return u32::MAX }
        if self < 0 { return 0 }
        self as u32
    }
}

impl SaturatingInto<i32> for i64 {
    fn saturating_into(self) -> i32 {
        if self > i32::MAX.into() { return i32::MAX }
        if self < i32::MIN.into() { return i32::MIN }
        self as i32
    }
}

impl SaturatingInto<i16> for i32 {
    fn saturating_into(self) -> i16 {
        if self > i16::MAX.into() { return i16::MAX }
        if self < i16::MIN.into() { return i16::MIN }
        self as i16
    }
}
