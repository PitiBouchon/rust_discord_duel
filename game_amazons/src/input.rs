use crate::pos::Pos;
use anyhow::Error;
use std::str::FromStr;

pub struct PlayerInput {
    pub from: Pos,
    pub to: Pos,
    pub arrow: Pos,
}

impl FromStr for PlayerInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elm: Vec<Pos> = s.trim().split('|').filter_map(Pos::parse_pos).collect();
        Ok(Self {
            from: *elm
                .get(0)
                .ok_or(Error::msg(format!("Error parsing output: {}", s)))?,
            to: *elm
                .get(1)
                .ok_or(Error::msg(format!("Error parsing output: {}", s)))?,
            arrow: *elm
                .get(2)
                .ok_or(Error::msg(format!("Error parsing output: {}", s)))?,
        })
    }
}
