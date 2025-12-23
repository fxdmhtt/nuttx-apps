use core::fmt;
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::sync::LazyLock;
use thiserror::Error;
use ultraviolet::{Mat4, Vec4};

#[derive(Error, Copy, Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[error("The chessboard is full.")]
pub struct ChessboardFull;

pub struct Game2048 {
    m: Mat4,
    score: usize,
}

impl Default for Game2048 {
    fn default() -> Self {
        Self {
            m: Mat4::from([
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ]),
            score: 0,
        }
    }
}

impl fmt::Display for Game2048 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..4 {
            for col in 0..4 {
                write!(f, "{:>5}", self.m[col][row])?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Score: {}\n", self.score)?;

        Ok(())
    }
}

impl Game2048 {
    pub fn up(&mut self) -> Vec<Path2D> {
        let (m, ds, ps) = matrix_merge(flip_v(self.m));
        self.score += ds;
        self.m = flip_v(m);
        ps.into_iter()
            .map(|Path2D { orig, dest, end }| Path2D {
                orig: Coord { row: 4 - 1 - orig.row, ..orig },
                dest: Coord { row: 4 - 1 - dest.row, ..dest },
                end,
            })
            .collect()
    }

    pub fn down(&mut self) -> Vec<Path2D> {
        let (m, ds, ps) = matrix_merge(self.m);
        self.score += ds;
        self.m = m;
        ps
    }

    pub fn left(&mut self) -> Vec<Path2D> {
        let (m, ds, ps) = matrix_merge(flip_h(self.m).transposed());
        self.score += ds;
        self.m = flip_h(m.transposed());
        ps.into_iter()
            .map(|Path2D { orig, dest, end }| Path2D {
                orig: Coord { col: orig.row, row: orig.col },
                dest: Coord { col: dest.row, row: dest.col },
                end,
            })
            .map(|Path2D { orig, dest, end }| Path2D {
                orig: Coord { col: 4 - 1 - orig.col, ..orig },
                dest: Coord { col: 4 - 1 - dest.col, ..dest },
                end,
            })
            .collect()
    }

    pub fn right(&mut self) -> Vec<Path2D> {
        let (m, ds, ps) = matrix_merge(self.m.transposed());
        self.score += ds;
        self.m = m.transposed();
        ps.into_iter()
            .map(|Path2D { orig, dest, end }| Path2D {
                orig: Coord { col: orig.row, row: orig.col },
                dest: Coord { col: dest.row, row: dest.col },
                end,
            })
            .collect()
    }

    fn select_space(&self) -> Vec<Coord> {
        (0..4)
            .flat_map(|col| (0..4).map(move |row| (row, col)))
            .filter(|&(row, col)| self.m[col][row] == 0.0)
            .map(|(row, col)| Coord { row, col })
            .collect()
    }

    fn random_select_space(&mut self, rng: &mut SmallRng) -> Result<Coord, ChessboardFull> {
        let positions = self.select_space();

        match positions.is_empty() {
            true => Err(ChessboardFull),
            false => Ok(positions[rng.random_range(0..positions.len())]),
        }
    }

    pub fn random_fill(&mut self) -> Result<(usize, Coord), ChessboardFull> {
        let mut rng = SmallRng::seed_from_u64(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .unwrap(),
        );

        let Coord { row, col } = self.random_select_space(&mut rng)?;
        let x = ((rng.random_bool(0.5) as i32 as f32 + 1.0) * 2.0).round();
        debug_assert!(x == 2.0 || x == 4.0);

        self.m[col][row] = x;
        Ok((x as usize, Coord { row, col }))
    }

    fn is_all_distinct(&self) -> bool {
        let cols = self.m.cols;

        for row in 0..4 {
            for col in 1..4 {
                if cols[col][row] == cols[col - 1][row] {
                    return false;
                }
            }
        }
        for col in 0..4 {
            for row in 1..4 {
                if cols[col][row] == cols[col][row - 1] {
                    return false;
                }
            }
        }
        true
    }

    pub fn is_it_over(&self) -> bool {
        self.select_space().is_empty() && self.is_all_distinct()
    }

    pub fn is_it_win(&self) -> bool {
        (0..4)
            .flat_map(|col| (0..4).map(move |row| (row, col)))
            .any(|(row, col)| self.m[col][row] == 2048.0)
    }

    pub fn get_score(&self) -> usize {
        self.score
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NumOp {
    OnlyMove,
    Double,
    Disappear,
}

#[derive(Clone)]
struct Path1D {
    orig: usize,
    dest: usize,
    end: NumOp,
}

impl Path1D {
    fn invalid(&self) -> bool {
        self.orig == self.dest && self.end == NumOp::OnlyMove
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub col: usize,
    pub row: usize,
}

#[derive(Debug)]
pub struct Path2D {
    pub orig: Coord,
    pub dest: Coord,
    pub end: NumOp,
}

static REV: LazyLock<Mat4> = LazyLock::new(|| {
    Mat4::from([
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ])
});

fn reduce_col(v: Vec4) -> (Vec4, usize, Vec<Path1D>) {
    let x = [v.x, v.y, v.z, v.w].into_iter().filter(|x| *x != 0.0).collect::<Vec<_>>();

    let o = [v.x, v.y, v.z, v.w]
        .into_iter()
        .enumerate()
        .filter(|(_, x)| *x != 0.0)
        .map(|(orig, _)| orig)
        .collect::<Vec<_>>();

    let (vec, dscore, mut ps) = match x.len() {
        0 => ([0.0, 0.0, 0.0, 0.0].into(), 0, vec![]),
        1 => (
            [0.0, 0.0, 0.0, x[0]].into(),
            0,
            vec![Path1D { orig: o[0], dest: 3, end: NumOp::OnlyMove }],
        ),
        2 => {
            if x[0] == x[1] {
                (
                    [0.0, 0.0, 0.0, x[0] + x[1]].into(),
                    (x[0] + x[1]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 3, end: NumOp::Double },
                        Path1D { orig: o[1], dest: 3, end: NumOp::Disappear },
                    ],
                )
            } else {
                (
                    [0.0, 0.0, x[0], x[1]].into(),
                    0,
                    vec![
                        Path1D { orig: o[0], dest: 2, end: NumOp::OnlyMove },
                        Path1D { orig: o[1], dest: 3, end: NumOp::OnlyMove },
                    ],
                )
            }
        }
        3 => {
            if x[1] == x[2] {
                (
                    [0.0, 0.0, x[0], x[1] + x[2]].into(),
                    (x[1] + x[2]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 2, end: NumOp::OnlyMove },
                        Path1D { orig: o[1], dest: 3, end: NumOp::Double },
                        Path1D { orig: o[2], dest: 3, end: NumOp::Disappear },
                    ],
                )
            } else if x[0] == x[1] {
                (
                    [0.0, 0.0, x[0] + x[1], x[2]].into(),
                    (x[0] + x[1]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 2, end: NumOp::Double },
                        Path1D { orig: o[1], dest: 2, end: NumOp::Disappear },
                        Path1D { orig: o[2], dest: 3, end: NumOp::OnlyMove },
                    ],
                )
            } else {
                (
                    [0.0, x[0], x[1], x[2]].into(),
                    0,
                    vec![
                        Path1D { orig: o[0], dest: 1, end: NumOp::OnlyMove },
                        Path1D { orig: o[1], dest: 2, end: NumOp::OnlyMove },
                        Path1D { orig: o[2], dest: 3, end: NumOp::OnlyMove },
                    ],
                )
            }
        }
        4 => {
            if x[0] == x[1] && x[2] == x[3] {
                (
                    [0.0, 0.0, x[0] + x[1], x[2] + x[3]].into(),
                    (x[0] + x[1] + x[2] + x[3]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 2, end: NumOp::Double },
                        Path1D { orig: o[1], dest: 2, end: NumOp::Disappear },
                        Path1D { orig: o[2], dest: 3, end: NumOp::Double },
                        Path1D { orig: o[3], dest: 3, end: NumOp::Disappear },
                    ],
                )
            } else if x[2] == x[3] {
                (
                    [0.0, x[0], x[1], x[2] + x[3]].into(),
                    (x[2] + x[3]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 1, end: NumOp::OnlyMove },
                        Path1D { orig: o[1], dest: 2, end: NumOp::OnlyMove },
                        Path1D { orig: o[2], dest: 3, end: NumOp::Double },
                        Path1D { orig: o[3], dest: 3, end: NumOp::Disappear },
                    ],
                )
            } else if x[1] == x[2] {
                (
                    [0.0, x[0], x[1] + x[2], x[3]].into(),
                    (x[1] + x[2]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 1, end: NumOp::OnlyMove },
                        Path1D { orig: o[1], dest: 2, end: NumOp::Double },
                        Path1D { orig: o[2], dest: 2, end: NumOp::Disappear },
                    ],
                )
            } else if x[0] == x[1] {
                (
                    [0.0, x[0] + x[1], x[2], x[3]].into(),
                    (x[0] + x[1]).round() as usize,
                    vec![
                        Path1D { orig: o[0], dest: 1, end: NumOp::Double },
                        Path1D { orig: o[1], dest: 1, end: NumOp::Disappear },
                    ],
                )
            } else {
                ([x[0], x[1], x[2], x[3]].into(), 0, vec![])
            }
        }
        _ => unreachable!(),
    };

    ps.retain(|p| !p.invalid());

    (vec, dscore, ps)
}

fn matrix_merge(m: Mat4) -> (Mat4, usize, Vec<Path2D>) {
    let r = m.cols.map(reduce_col);
    let m = Mat4 { cols: r.clone().map(|(v, _, _)| v) };
    let dscore = r.iter().map(|(_, s, _)| s).sum::<usize>();
    let ps = r
        .iter()
        .enumerate()
        .flat_map(|(col, (_, _, ps))| {
            ps.iter().map(move |Path1D { orig, dest, end }| Path2D {
                orig: Coord { col, row: *orig },
                dest: Coord { col, row: *dest },
                end: *end,
            })
        })
        .collect::<Vec<_>>();

    (m, dscore, ps)
}

fn flip_h(m: Mat4) -> Mat4 {
    #[allow(clippy::borrow_interior_mutable_const)]
    (m * (*REV))
}

fn flip_v(m: Mat4) -> Mat4 {
    #[allow(clippy::borrow_interior_mutable_const)]
    ((*REV) * m)
}
