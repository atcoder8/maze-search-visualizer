use itertools::{iproduct, Itertools};
use ndarray::prelude::*;
use rand::prelude::*;

use super::{MazeCellType, MazeGrid, MazeShape, ADJACENT_DISPLACEMENT};

const MAX_CANDIDATE_ENDPOINTS: usize = 10;

/// Generates a maze consisting of only passages and walls.
///
/// In generating the maze, the approach used is to dig through the walls to create passages.
fn generate_partial_maze<R>(shape: MazeShape, rng: &mut R) -> Array2<MazeCellType>
where
    R: Rng,
{
    let MazeShape { rows, cols } = shape;

    let effective_rows = (rows + 1) / 2;
    let effective_cols = (cols + 1) / 2;

    let mut cells = Array2::from_elem((rows, cols), MazeCellType::Wall);

    let init_row = 2 * rng.gen_range(0..effective_rows);
    let init_col = 2 * rng.gen_range(0..effective_cols);

    let mut coordinates_pool = vec![(init_row, init_col)];
    let mut idx = 0;

    loop {
        let coord = coordinates_pool[idx];
        let (row, col) = coord;

        cells[coord] = MazeCellType::Passage;

        let mut adjacent_diffs = ADJACENT_DISPLACEMENT;
        adjacent_diffs.shuffle(rng);

        let next_coord = adjacent_diffs.iter().find_map(|&(diff_row, diff_col)| {
            let cand_next_row = row.wrapping_add(diff_row.wrapping_mul(2));
            let cand_next_col = col.wrapping_add(diff_col.wrapping_mul(2));

            if cand_next_row < rows
                && cand_next_col < cols
                && cells[(cand_next_row, cand_next_col)] == MazeCellType::Wall
            {
                let adj_row = row.wrapping_add(diff_row);
                let adj_col = col.wrapping_add(diff_col);

                cells[(adj_row, adj_col)] = MazeCellType::Passage;
                cells[(cand_next_row, cand_next_col)] = MazeCellType::Passage;

                Some((cand_next_row, cand_next_col))
            } else {
                None
            }
        });

        match next_coord {
            Some(next_coord) => {
                coordinates_pool.push(next_coord);
                idx = coordinates_pool.len() - 1;
            }
            None => {
                coordinates_pool.remove(idx);

                if coordinates_pool.is_empty() {
                    break;
                }

                idx = rng.gen_range(0..coordinates_pool.len());
            }
        }
    }

    cells
}

fn calculate_path_length(
    maze_shape: MazeShape,
    maze: &Array2<MazeCellType>,
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<usize> {
    use std::collections::VecDeque;

    let MazeShape { rows, cols } = maze_shape;

    let mut coord_queue = VecDeque::from([(start, 0)]);
    let mut visited = Array2::from_elem((rows, cols), false);

    while let Some((coord, dist)) = coord_queue.pop_front() {
        if visited[coord] {
            continue;
        }

        visited[coord] = true;

        if coord == goal {
            return Some(dist);
        }

        let (row, col) = coord;
        for (diff_row, diff_col) in ADJACENT_DISPLACEMENT {
            let adj_row = row.wrapping_add(diff_row);
            let adj_col = col.wrapping_add(diff_col);
            let adj_coord = (adj_row, adj_col);

            if adj_row < rows && adj_col < cols && maze[adj_coord].is_passable() {
                coord_queue.push_back((adj_coord, dist + 1));
            }
        }
    }

    None
}

/// Generates a maze.
/// The maze consists of passages, walls, one starting point and one goal point.
pub(crate) fn generate_maze<R>(shape: MazeShape, rng: &mut R) -> MazeGrid
where
    R: Rng,
{
    let MazeShape { rows, cols } = shape;

    assert!(
        rows % 2 == 1 && cols % 2 == 1,
        "The number of rows and columns of the maze must be odd."
    );

    assert!(rows * cols >= 2, "The maze must contain multiple squares.");

    // Cells in the maze with undetermined start and goal points.
    let mut cells = generate_partial_maze(shape, rng);

    // Count the number of adjacent passable cells.
    let count_degree_num = |row: usize, col: usize| {
        ADJACENT_DISPLACEMENT
            .iter()
            .filter(|&&(diff_row, diff_col)| {
                let adj_row = row.wrapping_add(diff_row);
                let adj_col = col.wrapping_add(diff_col);

                adj_row < rows
                    && adj_col < cols
                    && cells[(adj_row, adj_col)] == MazeCellType::Passage
            })
            .count()
    };

    // Randomly select a pair of start and finish points from dead-end cells.
    let mut select_endpoints = || {
        let dead_ends = iproduct!((0..=rows).step_by(2), (0..=cols).step_by(2))
            .filter(|&(row, col)| count_degree_num(row, col) == 1)
            .collect_vec();

        // Randomly select candidate pairs of start and goal points.
        let mut coord_pairs = dead_ends.into_iter().combinations(2).collect_vec();
        coord_pairs.shuffle(rng);
        coord_pairs.truncate(MAX_CANDIDATE_ENDPOINTS);

        // Among the candidates, the one with the greatest distance when solving the maze is adopted.
        let mut coord_pair = coord_pairs
            .into_iter()
            .max_by_key(|coord_pair| {
                calculate_path_length(shape, &cells, coord_pair[0], coord_pair[1]).unwrap()
            })
            .unwrap();

        // Randomly choose one side as the start and the other side as the goal.
        coord_pair.shuffle(rng);
        (coord_pair[0], coord_pair[1])
    };

    // Determine the start and goal points.
    let (start, goal) = select_endpoints();
    cells[start] = MazeCellType::Start;
    cells[goal] = MazeCellType::Goal;

    MazeGrid {
        cells,
        start,
        goal,
        shape: MazeShape::new(rows, cols),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_maze() {
        const MAZE_ROWS: usize = 21;
        const MAZE_COLS: usize = 21;

        let mut rng = rand::thread_rng();

        let maze = generate_maze(MazeShape::new(MAZE_ROWS, MAZE_COLS), &mut rng);
        println!("{}", maze);
    }
}
