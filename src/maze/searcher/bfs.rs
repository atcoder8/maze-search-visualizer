use std::collections::VecDeque;

use ndarray::prelude::*;

use crate::maze::MazeGrid;
use crate::maze::{searcher::ExtraSearchError, MazeCellStatus};

use super::{MazeSearcher, ReservedRedraw, SearchProgress};

#[derive(Debug, Clone, Copy)]
struct SearchEdge {
    from: Option<(usize, usize)>,
    to: (usize, usize),
    distance: usize,
}

impl SearchEdge {
    fn init(start: (usize, usize)) -> Self {
        Self {
            from: None,
            to: start,
            distance: 0,
        }
    }

    fn next(self, next: (usize, usize)) -> Self {
        SearchEdge {
            from: Some(self.to),
            to: next,
            distance: self.distance + 1,
        }
    }
}

pub(crate) struct BFSSearcher {
    maze: MazeGrid,
    cell_statuses: Array2<MazeCellStatus>,
    edge_queue: VecDeque<SearchEdge>,
    progress: SearchProgress,
    dist_grid: Array2<Option<usize>>,
}

impl MazeSearcher for BFSSearcher {
    fn maze(&self) -> &MazeGrid {
        &self.maze
    }

    fn reset(&mut self) {
        let Self {
            maze,
            cell_statuses,
            edge_queue,
            progress,
            dist_grid,
        } = self;

        *cell_statuses = maze.cells.mapv(MazeCellStatus::new);
        *edge_queue = VecDeque::from([SearchEdge::init(self.maze.start)]);
        *progress = SearchProgress::InSearch;
        dist_grid.fill(None);
    }

    fn advance(&mut self) -> Result<Vec<ReservedRedraw>, ExtraSearchError> {
        match self.progress {
            SearchProgress::InSearch => {}
            SearchProgress::Solved => return Err(ExtraSearchError),
            SearchProgress::NoSolution => return Err(ExtraSearchError),
        }

        let mut pop_effective_node = || {
            while let Some(edge) = self.edge_queue.pop_front() {
                if self.dist_grid[edge.to].is_none() {
                    return Some(edge);
                }
            }

            None
        };

        let Some(edge) = pop_effective_node() else {
            self.progress = SearchProgress::NoSolution;
            return Ok(vec![]);
        };

        let mut reserved_redraws = vec![];

        self.dist_grid[edge.to] = Some(edge.distance);

        let maze_shape = self.maze.shape;

        // Update visible cell components.
        if let Some(from) = edge.from {
            self.cell_statuses[from].exit(false);

            reserved_redraws.push(ReservedRedraw {
                cell_idx: maze_shape.coord_to_idx(from),
                status: self.cell_statuses[from],
            });
        }

        self.cell_statuses[edge.to].enter(false);

        reserved_redraws.push(ReservedRedraw {
            cell_idx: maze_shape.coord_to_idx(edge.to),
            status: self.cell_statuses[edge.to],
        });

        let find_prev_coord = |coord| {
            let dist = self.dist_grid[coord].unwrap();
            self.maze
                .shape
                .adjacent_coordinates(coord)
                .find(|&adj_coord| {
                    self.dist_grid[adj_coord].is_some_and(|adj_dist| adj_dist == dist - 1)
                })
                .unwrap()
        };

        // Process when the maze is solved.
        if edge.to == self.maze.goal {
            // Restore a path from the start to the goal.
            let mut path = vec![self.maze.goal];
            path.reserve(edge.distance);
            for _ in 0..edge.distance {
                let prev_coord = find_prev_coord(*path.last().unwrap());
                path.push(prev_coord);
            }
            path.reverse();

            // Display the path from the start to the goal.
            for &coord in &path {
                self.cell_statuses[coord].set_on_path(true);

                reserved_redraws.push(ReservedRedraw {
                    cell_idx: maze_shape.coord_to_idx(coord),
                    status: self.cell_statuses[coord],
                });
            }

            // Update the progress.
            self.progress = SearchProgress::Solved;

            return Ok(reserved_redraws);
        }

        // Update the edge stack.
        for adj_coord in self.maze.shape.adjacent_coordinates(edge.to) {
            if self.maze.cells[adj_coord].is_passable() {
                self.edge_queue.push_back(edge.next(adj_coord));
            }
        }

        Ok(reserved_redraws)
    }

    fn progress(&self) -> &SearchProgress {
        &self.progress
    }
}

impl BFSSearcher {
    /// Attaches a maze to be visualized.
    pub(crate) fn new(maze: MazeGrid) -> Self {
        let shape = maze.shape;
        let cell_statuses = maze.cells.mapv(MazeCellStatus::new);
        let init_edge = SearchEdge::init(maze.start);

        Self {
            maze,
            cell_statuses,
            edge_queue: VecDeque::from([init_edge]),
            progress: SearchProgress::InSearch,
            dist_grid: Array2::from_elem((shape.rows, shape.cols), None),
        }
    }
}
