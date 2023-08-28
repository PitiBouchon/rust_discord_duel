#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Pos {
    // Should be in the form of "(5,6)"
    pub fn parse_pos(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim().split(',').collect();
        let (_, x_str) = parts.get(0)?.split_once('(')?;
        let x = x_str.trim().parse().ok()?;
        let (y_str, _) = parts.get(1)?.split_once(')')?;
        let y = y_str.trim().parse().ok()?;
        Some(Self { x, y })
    }

    pub fn shift(&self, dx: isize, dy: isize) -> Option<Self> {
        Some(Self {
            x: self.x.checked_add_signed(dx)?,
            y: self.y.checked_add_signed(dy)?,
        })
    }
}
