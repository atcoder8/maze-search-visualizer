use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use itertools::Itertools;

use crate::maze::generate_maze::generate_maze;
use crate::maze::searcher::{create_searcher, MazeSearcher};
use crate::maze::MazeGrid;
use crate::{MainWindow, MazeCellProperty};

/// Signal to the thread that performs the automatic search of the maze.
enum TaskSignal {
    /// Signal to interrupt the automatic search.
    Interrupt,
}

#[derive(Debug)]
struct AutoSearchTask {
    /// Handle of the thread performing automatic maze search.
    handle: thread::JoinHandle<()>,

    /// Sender of the signal to the thread.
    sender: mpsc::Sender<TaskSignal>,
}

/// If automatic search is begin performed, interrupt and wait for the thread to finish.
fn interrupt_search(task: Arc<Mutex<Option<AutoSearchTask>>>) {
    if let Some(AutoSearchTask { handle, sender }) = task.lock().unwrap().take() {
        let _ = sender.send(TaskSignal::Interrupt);
        handle.join().unwrap();
    };
}

/// Draws the maze in an unexplored state.
fn initialize_maze_drawing(
    maze: &MazeGrid,
    handle_weak: slint::Weak<MainWindow>,
) -> Result<(), slint::EventLoopError> {
    let properties = maze
        .cells
        .iter()
        .cloned()
        .map(MazeCellProperty::init)
        .collect_vec();

    handle_weak.upgrade_in_event_loop(move |handle| {
        let model = Rc::new(slint::VecModel::from(properties));
        handle.set_properties_of_cells(model.into());
    })
}

/// Advance the maze search by one cell and reflect it in the drawing.
fn advance_search(
    searcher: Arc<Mutex<Box<dyn MazeSearcher>>>,
    handle_weak: slint::Weak<MainWindow>,
) -> bool {
    if searcher.lock().unwrap().terminated() {
        return false;
    }

    let updated_statuses = searcher.lock().unwrap().advance().unwrap();

    for updated_status in updated_statuses {
        handle_weak
            .upgrade_in_event_loop(move |handle| {
                handle.redraw_cell(updated_status.cell_idx, updated_status.status);
            })
            .unwrap();
    }

    true
}

/// Automatically search the maze.
fn auto_search_maze(
    searcher: Arc<Mutex<Box<dyn MazeSearcher>>>,
    receiver: mpsc::Receiver<TaskSignal>,
    handle_weak: slint::Weak<MainWindow>,
) {
    loop {
        if let Ok(signal) = receiver.try_recv() {
            match signal {
                TaskSignal::Interrupt => break,
            }
        }

        if !advance_search(Arc::clone(&searcher), handle_weak.clone()) {
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }
}

/// Spawns an automatic search task.
fn spawn_auto_search_task(
    searcher: Arc<Mutex<Box<dyn MazeSearcher>>>,
    handle_weak: slint::Weak<MainWindow>,
) -> AutoSearchTask {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || auto_search_maze(searcher, receiver, handle_weak));

    AutoSearchTask { handle, sender }
}

fn update_maze_searcher(
    maze: MazeGrid,
    searcher: Arc<Mutex<Box<dyn MazeSearcher>>>,
    task: Arc<Mutex<Option<AutoSearchTask>>>,
    handle_weak: slint::Weak<MainWindow>,
) {
    interrupt_search(task);
    initialize_maze_drawing(&maze, handle_weak.clone()).unwrap();
    *searcher.lock().unwrap() =
        create_searcher(maze, &handle_weak.unwrap().get_selected_search_algorithm());
}

pub(crate) struct Visualizer {
    searcher: Arc<Mutex<Box<dyn MazeSearcher>>>,
    task: Arc<Mutex<Option<AutoSearchTask>>>,
}

impl Visualizer {
    /// Sets the behavior when the advance button is pressed.
    fn set_play_pause_callback(&self, handle_weak: slint::Weak<MainWindow>) {
        let task = Arc::clone(&self.task);
        let searcher = Arc::clone(&self.searcher);

        handle_weak.unwrap().on_play_pause_callback(move || {
            if task.lock().unwrap().is_some() {
                interrupt_search(task.clone());
            } else {
                *task.lock().unwrap() = Some(spawn_auto_search_task(
                    searcher.clone(),
                    handle_weak.clone(),
                ));
            }
        });
    }

    /// Sets the process when the advance button is pressed.
    fn set_advance_callback(&self, handle_weak: slint::Weak<MainWindow>) {
        let task = Arc::clone(&self.task);
        let searcher = Arc::clone(&self.searcher);

        handle_weak.unwrap().on_advance_callback(move || {
            interrupt_search(task.clone());

            advance_search(searcher.clone(), handle_weak.clone());
        });
    }

    /// Sets the process when the reset button is pressed.
    fn set_reset_callback(&self, handle_weak: slint::Weak<MainWindow>) {
        let task = Arc::clone(&self.task);
        let searcher = Arc::clone(&self.searcher);

        handle_weak.unwrap().on_reset_callback(move || {
            interrupt_search(task.clone());

            searcher.lock().unwrap().reset();
            initialize_maze_drawing(searcher.lock().unwrap().maze(), handle_weak.clone()).unwrap();
        });
    }

    /// Sets the process when the change button is pressed.
    fn set_change_callback(&self, handle_weak: slint::Weak<MainWindow>) {
        let task = Arc::clone(&self.task);
        let searcher = Arc::clone(&self.searcher);
        let maze_shape = self.searcher.lock().unwrap().maze().shape;

        handle_weak.unwrap().on_change_callback(move || {
            update_maze_searcher(
                generate_maze(maze_shape, &mut rand::thread_rng()),
                Arc::clone(&searcher),
                Arc::clone(&task),
                handle_weak.clone(),
            );
        });
    }

    fn set_select_algorithm_callback(&self, handle_weak: slint::Weak<MainWindow>) {
        let task = Arc::clone(&self.task);
        let searcher = Arc::clone(&self.searcher);

        handle_weak.unwrap().on_select_algorithm_callback(move || {
            let maze = searcher.lock().unwrap().maze().clone();
            update_maze_searcher(
                maze,
                Arc::clone(&searcher),
                Arc::clone(&task),
                handle_weak.clone(),
            );
        });
    }

    pub(crate) fn new(
        searcher: Box<dyn MazeSearcher>,
        handle_weak: slint::Weak<MainWindow>,
    ) -> Self {
        initialize_maze_drawing(searcher.maze(), handle_weak.clone()).unwrap();

        let searcher = Arc::new(Mutex::new(searcher));
        let task = Arc::new(Mutex::new(None));
        let visualizer = Self { searcher, task };

        // Set the process when each button is pressed.
        visualizer.set_advance_callback(handle_weak.clone());
        visualizer.set_play_pause_callback(handle_weak.clone());
        visualizer.set_reset_callback(handle_weak.clone());
        visualizer.set_change_callback(handle_weak.clone());
        visualizer.set_select_algorithm_callback(handle_weak);

        visualizer
    }
}
