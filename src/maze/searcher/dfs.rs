use ndarray::prelude::*;

use crate::maze::MazeGrid;
use crate::maze::{searcher::ExtraSearchError, MazeCellStatus};

use super::{MazeSearcher, ReservedRedraw, SearchProgress};

#[derive(Debug, Clone, Copy)]
struct SearchEdge {
    from: Option<(usize, usize)>,
    to: (usize, usize),
    forward: bool,
}

impl SearchEdge {
    fn init(start: (usize, usize)) -> Self {
        Self {
            from: None,
            to: start,
            forward: true,
        }
    }

    fn back(self) -> Self {
        assert!(self.forward);

        SearchEdge {
            forward: false,
            ..self
        }
    }

    fn next_forward(self, next: (usize, usize)) -> Self {
        assert!(self.forward);

        SearchEdge {
            from: Some(self.to),
            to: next,
            forward: true,
        }
    }
}

pub(crate) struct DFSSearcher {
    maze: MazeGrid,
    cell_statuses: Array2<MazeCellStatus>,
    edge_stack: Vec<SearchEdge>,
    progress: SearchProgress,
    path: Vec<(usize, usize)>,
}

impl MazeSearcher for DFSSearcher {
    fn maze(&self) -> &MazeGrid {
        &self.maze
    }

    fn reset(&mut self) {
        let Self {
            maze,
            cell_statuses,
            edge_stack,
            progress,
            path,
        } = self;

        *cell_statuses = maze.cells.mapv(MazeCellStatus::new);
        let init_edge = SearchEdge::init(self.maze.start);
        *edge_stack = vec![init_edge.back(), init_edge];
        *progress = SearchProgress::InSearch;
        *path = vec![];
    }

    fn advance(&mut self) -> Result<Vec<ReservedRedraw>, ExtraSearchError> {
        let maze_shape = self.maze.shape;

        match self.progress {
            SearchProgress::InSearch => {}
            SearchProgress::Solved => return Err(ExtraSearchError),
            SearchProgress::NoSolution => return Err(ExtraSearchError),
        }

        let pop_effective_edge = |edge_stack: &mut Vec<SearchEdge>| {
            while let Some(edge) = edge_stack.pop() {
                if !edge.forward || !self.cell_statuses[edge.to].visited {
                    return Some(edge);
                }
            }

            None
        };

        let edge = match pop_effective_edge(&mut self.edge_stack) {
            Some(edge) => edge,
            None => {
                self.progress = SearchProgress::NoSolution;
                return Ok(vec![]);
            }
        };

        let mut reserved_redraws = vec![];

        // Update the path.
        if edge.forward {
            self.path.push(edge.to);
        } else {
            self.path.pop();
        }

        // Update the edge stack.
        if edge.forward {
            assert!(!self.cell_statuses[edge.to].visited);

            self.edge_stack.push(edge.back());

            for adj_coord in self.maze.shape.adjacent_coordinates(edge.to) {
                if self.maze.cells[adj_coord].is_passable() {
                    self.edge_stack.push(edge.next_forward(adj_coord));
                }
            }
        }

        // Update visible cell components.
        if let Some(from) = edge.from {
            if edge.forward {
                self.cell_statuses[from].exit(true);
            } else {
                self.cell_statuses[from].enter(true);
            }

            reserved_redraws.push(ReservedRedraw {
                cell_idx: maze_shape.coord_to_idx(from),
                status: self.cell_statuses[from],
            });
        }

        if edge.forward {
            self.cell_statuses[edge.to].enter(true);
        } else {
            self.cell_statuses[edge.to].exit(false);
        }

        reserved_redraws.push(ReservedRedraw {
            cell_idx: maze_shape.coord_to_idx(edge.to),
            status: self.cell_statuses[edge.to],
        });

        // Process when the maze is solved.
        if edge.to == self.maze.goal {
            // Display the path from the start to the goal.
            for &coord in &self.path {
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

        Ok(reserved_redraws)
    }

    fn progress(&self) -> &SearchProgress {
        &self.progress
    }
}

impl DFSSearcher {
    /// Attaches a maze to be visualized.
    pub(crate) fn new(maze: MazeGrid) -> Self {
        let cell_statuses = maze.cells.mapv(MazeCellStatus::new);
        let init_edge = SearchEdge::init(maze.start);

        Self {
            maze,
            cell_statuses,
            edge_stack: vec![init_edge.back(), init_edge],
            progress: SearchProgress::InSearch,
            path: vec![],
        }
    }
}
