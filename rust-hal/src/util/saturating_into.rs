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

