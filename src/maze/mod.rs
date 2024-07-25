pub(crate) mod generate_maze;
pub(crate) mod searcher;

use itertools::Itertools;
use ndarray::prelude::*;

use crate::utils::palette;

pub(crate) const ADJACENT_DISPLACEMENT: [(usize, usize); 4] = [(!0, 0), (0, !0), (0, 1), (1, 0)];

/// Represents the role of a cell on the maze.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MazeCellType {
    /// Passable cell.
    /// However, it is neither the start nor the goal.
    Passage,

    /// Impassable cell.
    Wall,

    /// Starting point (passable).
    Start,

    /// Goal point (passable).
    Goal,
}

impl From<MazeCellType> for char {
    fn from(value: MazeCellType) -> Self {
        match value {
            MazeCellType::Passage => '.',
            MazeCellType::Wall => '#',
            MazeCellType::Start => 'S',
            MazeCellType::Goal => 'G',
        }
    }
}

impl MazeCellType {
    /// Returns whether this cell is passable or not.
    /// If a cell is not a wall, it is passable.
    pub(crate) fn is_passable(&self) -> bool {
        !matches!(self, MazeCellType::Wall)
    }
}

/// Maze shape (number of rows and columns).
#[derive(Debug, Clone, Copy)]
pub(crate) struct MazeShape {
    /// Number of maze rows.
    pub(crate) rows: usize,

    /// Number of maze columns.
    pub(crate) cols: usize,
}

impl MazeShape {
    pub(crate) fn new(rows: usize, cols: usize) -> Self {
        Self { rows, cols }
    }

    /// Returns whether the coordinate exist in the maze.
    pub(crate) fn in_range(&self, coord: (usize, usize)) -> bool {
        coord.0 < self.rows && coord.1 < self.cols
    }

    /// Returns the vector of cells adjacent to `coord` by an edge.
    pub(crate) fn adjacent_coordinates(
        &self,
        coord: (usize, usize),
    ) -> impl '_ + Iterator<Item = (usize, usize)> {
        assert!(self.in_range(coord));

        ADJACENT_DISPLACEMENT.iter().filter_map(move |&(dr, dc)| {
            let adj_coord = (coord.0.wrapping_add(dr), coord.1.wrapping_add(dc));

            if self.in_range(adj_coord) {
                Some(adj_coord)
            } else {
                None
            }
        })
    }

    #[allow(unused)]
    pub(crate) fn area(&self) -> usize {
        self.rows * self.cols
    }

    #[allow(unused)]
    pub(crate) fn coord_to_idx(&self, coord: (usize, usize)) -> usize {
        assert!(self.in_range(coord));

        coord.0 * self.cols + coord.1
    }

    #[allow(unused)]
    pub(crate) fn idx_to_coord(&self, cell_idx: usize) -> (usize, usize) {
        assert!(cell_idx < self.area());

        (cell_idx / self.cols, cell_idx % self.cols)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MazeGrid {
    pub(crate) shape: MazeShape,
    pub(crate) cells: Array2<MazeCellType>,
    pub(crate) start: (usize, usize),
    pub(crate) goal: (usize, usize),
}

impl std::fmt::Display for MazeGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let maze_str = self
            .cells
            .axis_iter(Axis(0))
            .map(|maze_row| maze_row.iter().map(|&cell| char::from(cell)).join(" "))
            .join("\n");

        write!(f, "{}", maze_str)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct MazeCellStatus {
    pub(crate) cell_type: MazeCellType,
    pub(crate) stay: bool,
    pub(crate) visited: bool,
    pub(crate) footprint: bool,
    pub(crate) on_path: bool,
}

impl MazeCellStatus {
    pub(crate) fn new(cell_type: MazeCellType) -> Self {
        MazeCellStatus {
            cell_type,
            stay: false,
            visited: false,
            footprint: false,
            on_path: false,
        }
    }

    pub(crate) fn cell_color(&self) -> slint::Color {
        match self.cell_type {
            MazeCellType::Passage => {}
            MazeCellType::Wall => return palette::GRAY,
            MazeCellType::Start => return palette::RED,
            MazeCellType::Goal => return palette::BLUE,
        }

        if self.on_path {
            return palette::YELLOW;
        }

        if self.visited {
            return palette::BRIGHT_GREEN;
        }

        palette::WHITE
    }

    pub(crate) fn enter(&mut self, footprint: bool) {
        self.stay = true;
        self.visited = true;
        self.footprint = footprint;
    }

    pub(crate) fn exit(&mut self, footprint: bool) {
        self.stay = false;
        self.footprint = footprint;
    }

    pub(crate) fn set_on_path(&mut self, on_path: bool) {
        self.on_path = on_path;
    }
}
