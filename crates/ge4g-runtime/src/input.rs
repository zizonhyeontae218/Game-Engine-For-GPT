//! Deterministic step-walk arbitration; maze steering is deliberately independent.
use ge4g_core::Input;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DirectionInput {
    held: [bool; 4],
    priority: Vec<usize>,
}
impl DirectionInput {
    pub fn cardinal(&mut self, input: &Input) -> [i64; 2] {
        let now = [input.left, input.right, input.up, input.down];
        self.priority.retain(|&i| now[i]);
        for (i, &down) in now.iter().enumerate() {
            if down && !self.held[i] {
                self.priority.retain(|&old| old != i);
                self.priority.push(i);
            }
        }
        self.held = now;
        let directions = [[-1, 0], [1, 0], [0, -1], [0, 1]];
        if let Some(intent) = input.direction
            && let Some(i) = directions.iter().position(|&d| d == intent)
            && now[i]
        {
            self.priority.retain(|&old| old != i);
            self.priority.push(i);
        }
        self.priority.last().map_or([0, 0], |&i| directions[i])
    }
    pub fn valid(&self) -> bool {
        self.priority.len() <= 4
            && self.priority.iter().all(|&i| i < 4 && self.held[i])
            && self
                .priority
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == self.priority.len()
    }
}
/// Reversal within an already-entered cell is legal; orthogonal turns wait for alignment.
pub fn mid_cell_direction(
    position: ge4g_core::Vec2,
    cell: i64,
    current: [i64; 2],
    desired: [i64; 2],
) -> [i64; 2] {
    let horizontal = position.x.rem_euclid(cell) != 0 && position.y.rem_euclid(cell) == 0;
    let vertical = position.y.rem_euclid(cell) != 0 && position.x.rem_euclid(cell) == 0;
    if ((horizontal && desired[0] != 0) || (vertical && desired[1] != 0))
        && (current == [0, 0] || desired == [-current[0], -current[1]])
    {
        desired
    } else {
        current
    }
}
