use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Error};

use crate::board;

pub struct Db {
    connection: PooledConnection<SqliteConnectionManager>,
}

impl Db {
    pub fn new(connection: PooledConnection<SqliteConnectionManager>) -> Self {
        Self { connection }
    }

    /// Initializes the DB to this app's specific shape. Creates tables if they don't exist.
    /// Can be called every time the program starts.
    pub fn initialize(connection: PooledConnection<SqliteConnectionManager>) {
        let create_boards = "
        CREATE TABLE IF NOT EXISTS Boards (
            id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,

            size INTEGER NOT NULL,
            starting_subdivisions INTEGER NOT NULL,
            starting_subdiv_utilization INTEGER NOT NULL,
            cells TEXT NOT NULL,

            period INTEGER,
            iterations INTEGER NOT NULL
            );
        ";

        [create_boards].map(|query| {
            connection.execute(query, params![]).unwrap();
        });
    }

    /// Takes a number of cells and a board size and saves that board to the db
    /// Returns Result<board_id>
    pub fn save_board(&mut self, board: &board::Solved) -> Result<i64, Error> {
        let cells = Db::serialize_cells(&board.initial.cells);

        // Insert one new board
        self.connection.execute(
            "INSERT INTO Boards (
                size,
                starting_subdivisions,
                starting_subdiv_utilization,
                cells,
                period,
                iterations
                ) VALUES (?, ?, ?, ?, ?, ?)",
            params![
                board.initial.size,
                board.initial.starting_subdivisions,
                board.initial.starting_subdiv_utilization,
                cells,
                board.period,
                board.iterations
            ],
        )?;

        // What was that last id?
        let board_id = self.connection.last_insert_rowid();

        Ok(board_id)
    }

    /// Get a single board from the db
    pub fn load_board(&self, board_id: i64) -> Result<board::Saved, Error> {
        let (
            id,
            size,
            starting_subdivisions,
            starting_subdiv_utilization,
            cells_str,
            period,
            iterations,
        ): (i64, u32, usize, usize, String, usize, usize) = self.connection.query_row(
            "SELECT * FROM Boards WHERE id = ?",
            params![board_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )?;

        let cells = Db::deserialize_cells(&cells_str);

        Ok(board::Saved {
            id,
            solved: board::Solved {
                iterations,
                period: Some(period),
                initial: board::Initial {
                    size,
                    cells,
                    starting_subdivisions,
                    starting_subdiv_utilization,
                },
            },
        })
    }

    /// Get all the boards from the db
    pub fn load_boards(&self) -> Result<Vec<board::Saved>, Error> {
        let mut stmt = self.connection.prepare("SELECT * FROM Boards")?;

        let boards_iter = stmt.query_map([], |row| {
            let cells = Db::deserialize_cells(&row.get(4)?);
            Ok(board::Saved {
                id: row.get(0)?,
                solved: board::Solved {
                    iterations: row.get(6)?,
                    period: Some(row.get(5)?),
                    initial: board::Initial {
                        size: row.get(1)?,
                        cells,
                        starting_subdivisions: row.get(2)?,
                        starting_subdiv_utilization: row.get(3)?,
                    },
                },
            })
        })?;

        let mut boards = vec![];

        for board in boards_iter {
            boards.push(board.unwrap());
        }

        Ok(boards)
    }

    pub fn delete_board(&mut self, board_id: &i64) -> Result<(), Error> {
        self.connection
            .execute("DELETE FROM Boards WHERE id = ?", params![board_id])?;

        Ok(())
    }

    /// How many boards are there in the db?
    pub fn get_board_count(&self) -> Result<u64, Error> {
        let count = self
            .connection
            .query_row("SELECT COUNT(*) FROM Boards", params![], |row| {
                Ok(row.get(0)?)
            })?;

        Ok(count)
    }

    /// For simplicity's sake, even though it's not technically correct, we're stringifying the
    /// board's cells and storing them in a single db cell in the Boards table.
    fn serialize_cells(cells: &Vec<(u32, u32)>) -> String {
        let mut cells: String = cells.iter().map(|(i, j)| format!("{}-{},", i, j)).collect();
        cells.pop(); // we don't want the last |
        cells
    }

    fn deserialize_cells(cells_str: &String) -> Vec<(u32, u32)> {
        let cells = cells_str
            .split(",")
            .map(|str| {
                let mut subsplit = str.split("-");
                let i: u32 = subsplit.next().unwrap().parse().unwrap();
                let j: u32 = subsplit.next().unwrap().parse().unwrap();
                (i, j)
            })
            .collect();

        cells
    }
}

#[test]
fn saving_and_loading_boards() {
    let manager = SqliteConnectionManager::memory();
    let pool = r2d2::Pool::new(manager).unwrap();
    Db::initialize(pool.get().unwrap());
    let mut db = Db::new(pool.get().unwrap());

    let board = board::Solved {
        initial: board::Initial {
            size: 10,
            starting_subdivisions: 5,
            starting_subdiv_utilization: 3,
            cells: vec![(1, 1), (2, 2), (3, 3)],
        },
        period: Some(10),
        iterations: 100,
    };

    let board_id = db.save_board(&board).unwrap();

    let retrieved_board = db.load_board(board_id).unwrap();

    assert_eq!(board.initial.size, retrieved_board.solved.initial.size);
    assert_eq!(board.iterations, retrieved_board.solved.iterations);
    assert_eq!(board.period, retrieved_board.solved.period);
    assert_eq!(board.initial.cells, retrieved_board.solved.initial.cells);
    assert_eq!(board.initial.starting_subdivisions, retrieved_board.solved.initial.starting_subdivisions);
    assert_eq!(board.initial.starting_subdiv_utilization, retrieved_board.solved.initial.starting_subdiv_utilization);

    let id = retrieved_board.id;
    assert!(id > 0);

    let retrieved_boards = db.load_boards().unwrap();

    assert_eq!(retrieved_boards.len(), 1);
}
