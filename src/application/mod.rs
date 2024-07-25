use slint::Model;

use crate::maze::{MazeCellStatus, MazeCellType, MazeShape};
use crate::{MainWindow, MazeCellProperty};

impl MainWindow {
    pub(crate) fn empty_maze_window(
        config: &AppConfig,
    ) -> Result<MainWindow, slint::PlatformError> {
        let &AppConfig {
            ref title,
            maze_rows,
            maze_cols,
            margin,
            ..
        } = config;

        let handle = MainWindow::new()?;
        handle.set_app_title(title.into());

        // Set a maze shape.
        handle.set_maze_rows(maze_rows as i32);
        handle.set_maze_cols(maze_cols as i32);
        handle.set_cell_size(config.calc_cell_size());
        handle.set_margin(margin);

        Ok(handle)
    }

    pub(crate) fn redraw_cell(&self, cell_idx: usize, status: MazeCellStatus) {
        self.get_properties_of_cells()
            .set_row_data(cell_idx, MazeCellProperty::from_status(status));
    }
}

impl MazeCellProperty {
    pub(crate) fn init(cell_type: MazeCellType) -> Self {
        use slint::Color;

        let color = match cell_type {
            MazeCellType::Passage => Color::from_rgb_u8(255, 255, 255),
            MazeCellType::Wall => Color::from_rgb_u8(127, 127, 127),
            MazeCellType::Start => Color::from_rgb_u8(255, 40, 0),
            MazeCellType::Goal => Color::from_rgb_u8(0, 65, 255),
        };

        Self {
            cell_color: color,
            footprint: false,
        }
    }

    pub(crate) fn from_status(status: MazeCellStatus) -> Self {
        Self {
            cell_color: status.cell_color(),
            footprint: status.footprint,
        }
    }
}

#[derive(Debug)]
pub(crate) struct AppConfig {
    pub(crate) title: String,
    pub(crate) max_maze_height: usize,
    pub(crate) max_maze_width: usize,
    pub(crate) maze_rows: usize,
    pub(crate) maze_cols: usize,
    pub(crate) max_cell_size: f32,
    pub(crate) margin: f32,
}

impl AppConfig {
    pub(crate) fn calc_cell_size(&self) -> f32 {
        self.max_cell_size
            .min(self.max_maze_height as f32 / self.maze_rows as f32)
            .min(self.max_maze_width as f32 / self.maze_cols as f32)
    }

    pub(crate) fn maze_shape(&self) -> MazeShape {
        MazeShape::new(self.maze_rows, self.maze_cols)
    }
}
