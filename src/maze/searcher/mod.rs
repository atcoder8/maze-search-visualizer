use std::{error, fmt};

use super::{MazeCellStatus, MazeGrid};

pub(crate) mod astar;
pub(crate) mod bfs;
pub(crate) mod dfs;

/// Error returned if the maze search has already been finished or interrupted,
/// but an attempt is made to advance the search.
#[derive(Debug)]
pub(crate) struct ExtraSearchError;

impl fmt::Display for ExtraSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Maze search has already been completed or interrupted.")
    }
}

impl error::Error for ExtraSearchError {}

#[derive(Debug, Clone)]
pub(crate) enum SearchProgress {
    InSearch,
    Solved,
    NoSolution,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ReservedRedraw {
    pub(crate) cell_idx: usize,
    pub(crate) status: MazeCellStatus,
}

pub(crate) trait MazeSearcher: 'static + Send + Sync {
    fn maze(&self) -> &MazeGrid;

    fn reset(&mut self);

    /// Advance the maze search by one cell.
    fn advance(&mut self) -> Result<Vec<ReservedRedraw>, ExtraSearchError>;

    fn progress(&self) -> &SearchProgress;

    fn terminated(&self) -> bool {
        match self.progress() {
            SearchProgress::InSearch => false,
            SearchProgress::Solved => true,
            SearchProgress::NoSolution => true,
        }
    }
}

pub(crate) fn create_searcher<S>(maze: MazeGrid, algorithm: &S) -> Box<dyn MazeSearcher>
where
    S: AsRef<str>,
{
    match algorithm.as_ref() {
        "DFS" => Box::new(dfs::DFSSearcher::new(maze)),
        "BFS" => Box::new(bfs::BFSSearcher::new(maze)),
        "A*" => Box::new(astar::ASterSearcher::new(maze)),
        algorithm => panic!("{} is the unknown search algorithm.", algorithm),
    }
}
