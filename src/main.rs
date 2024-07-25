slint::include_modules!();

use application::AppConfig;
use maze::{generate_maze, searcher::dfs::DFSSearcher};
use slint::ComponentHandle;
use visualizer::Visualizer;

mod application;
mod maze;
mod utils;
mod visualizer;

const APP_TITLE: &str = "Maze search visualizer";
const MAX_MAZE_HEIGHT: usize = 720;
const MAX_MAZE_WIDTH: usize = 720;
const MAZE_ROWS: usize = 25;
const MAZE_COLS: usize = 25;
const CELL_SIZE: f32 = 32.0;
const MARGIN: f32 = 1.0;

fn main() -> Result<(), slint::PlatformError> {
    let mut rng = rand::thread_rng();

    let config = AppConfig {
        title: APP_TITLE.to_string(),
        max_maze_height: MAX_MAZE_HEIGHT,
        max_maze_width: MAX_MAZE_WIDTH,
        maze_rows: MAZE_ROWS,
        maze_cols: MAZE_COLS,
        max_cell_size: CELL_SIZE,
        margin: MARGIN,
    };

    let handle = MainWindow::empty_maze_window(&config)?;
    let handle_weak = handle.as_weak();

    let init_maze = generate_maze::generate_maze(config.maze_shape(), &mut rng);
    let dfs_searcher = DFSSearcher::new(init_maze.clone());

    Visualizer::new(Box::new(dfs_searcher), handle_weak);

    handle.run()
}
