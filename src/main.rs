// TODO
//
// - [ ] load game from db
// - [x] store game moves in db
// - [ ] broadcast move to opponent and spectators
// - [ ] turns
// - [ ] takes view
// - [x] regular moves
// - [x] takes
// - [ ] special moves (en passant, castling)
// - [ ] users/csrf/magiclinks
// - [ ] something to interactively update game state and show when
//       opponent moves, my turn, etc. (sse, polling, etc.)
// - [ ] fly deploy (dockerfile, fly.toml)

use crate::board::Board;
use crate::piece::{Color, Piece, Position};
use anyhow::anyhow;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post, put};
use clap::Parser;
use maud::{Markup, html};
use serde::Deserialize;
use sqlx::{Acquire, Pool, Sqlite};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing::{debug, info};
use uuid::Uuid;

mod board;
mod piece;

macro_rules! layout {
    ($content:expr) => {
        maud::html! {
            (maud::DOCTYPE)
            html lang="en" {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    title { "chess" }
                    script src="https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4" {}
                    script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" {}
                    style {
                        ".bg-dark {
                            background-color: gray;
                        }
                        .bg-light {
                            background-color: white;
                        }
                        .aspect-square {
                            aspect-ratio: 1 / 1;
                        }
                        "

                    }
                }
                ($content)
            }
        }
    };
}

async fn games_new() -> Result<impl IntoResponse, AppError> {
    Ok(layout! {
        html! {
            div {
                ("Play as:")
                div {
                    button
                        hx-post="/games/create?playing_as=black"
                        hx-target="body"
                        hx-push-url="true"
                    {
                        "Black"
                    }
                    button
                        hx-post="/games/create?playing_as=white"
                        hx-target="body"
                        hx-push-url="true"
                    {
                        "White"
                    }
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct GamesPlayParams {
    playing_as: Color,
}

async fn games_create(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<GamesPlayParams>,
) -> Result<impl IntoResponse, AppError> {
    let mut state = state.lock().await;

    let mut conn = state.pool.acquire().await?;

    let (game_id,): (Uuid,) = sqlx::query_as(
        "
    insert into games (id) values (?) returning id;
    ",
    )
    .bind(Uuid::new_v4())
    .fetch_one(&mut *conn)
    .await?;

    state.games.entry(game_id).or_insert_with(|| GameState {
        board: Board::new(),
        selected: None,
        possible_moves: vec![],
        takes: vec![],
        to_move: Color::White,
    });

    let mut headers = HeaderMap::new();

    headers.insert(
        "HX-Location",
        format!(
            "/games/{game_id}/play?playing_as={}",
            match params.playing_as {
                Color::Black => "black",
                Color::White => "white",
            }
        )
        .parse()
        .unwrap(),
    );

    Ok(headers)
}

async fn games_play(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(game_id): Path<Uuid>,
    Query(params): Query<GamesPlayParams>,
) -> Result<impl IntoResponse, AppError> {
    let state = state.lock().await;

    if let Some(game_state) = state.games.get(&game_id) {
        Ok(layout! {
            (board(game_id, &game_state.board, params.playing_as))
        })
    } else {
        Err(anyhow!("could not find game with that id").into())
    }
}

fn board(game_id: Uuid, board_data: &Board, playing_as: Color) -> Markup {
    const INCREASING: [i8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    const DECREASING: [i8; 8] = [7, 6, 5, 4, 3, 2, 1, 0];

    let (row_range, column_range) = match playing_as {
        Color::Black => (INCREASING, DECREASING),
        Color::White => (DECREASING, INCREASING),
    };

    html! {
        div id="board" class="max-h-svh sm:order-2 sm:col-span-4 items-center justify-center " {
            div class="max-h-svh m-6 border-solid border-1 aspect-square" {
                @for row in row_range.into_iter() {
                    div class="flex"  {
                        @for column in column_range.into_iter() {
                            (square(
                                game_id,
                                &(column, row).into(),
                                Position::from((column, row)).color().into(),
                                board_data.get_piece(&(column, row).into()).map_or("", |piece| piece.repr()),
                                false
                            ))
                        }
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct SquareClick {
    column: i8,
    row: i8,
}

async fn square_clicked(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(game_id): Path<Uuid>,
    Query(params): Query<SquareClick>,
) -> Result<impl IntoResponse, AppError> {
    let mut state = state.lock().await;

    let mut conn = state.pool.acquire().await?;

    let game_state = state.games.get_mut(&game_id).unwrap();

    let position = (params.column, params.row).into();

    if let Some(selected) = game_state.selected {
        if selected == position {
            game_state.selected = None;

            Ok(html! {
                @for position in &game_state.possible_moves {
                    @if let Some(piece_at) = game_state.board.get_piece(position) {
                        (square(game_id, position, position.color().into(), piece_at.repr(), true))
                    } @else {
                        (square(game_id, position, position.color().into(), "", true))
                    }
                }
            })
        } else {
            if game_state.possible_moves.contains(&position) {
                debug!("made a valid move");
                // do the move
                //
                // update board
                let current_piece_location =
                    game_state.board.get_piece(&selected).unwrap().to_owned();

                if let Some(take) = game_state.board.move_piece(&selected, &position) {
                    game_state.takes.push(take);
                }
                // record move in db
                sqlx::query(
                    "insert into moves
                (game_id, from_column, from_row, to_column, to_row)
                values (?, ?, ?, ?, ?);",
                )
                .bind(game_id)
                .bind(selected.column)
                .bind(selected.row)
                .bind(position.column)
                .bind(position.row)
                .execute(&mut *conn)
                .await?;

                // change render of board,
                // deselect

                let out = html! {
                    // blank the from position
                    (square(game_id, &selected, selected.color().into(), "", true))
                    // move the piece to the target position
                    (square(game_id, &position, position.color().into(), current_piece_location.repr(), true))

                    @for m in &game_state.possible_moves {
                        @if let Some(piece_at) = game_state.board.get_piece(m) {
                            (square(game_id, &m, m.color().into(), piece_at.repr(), true))
                        } @else {
                            (square(game_id, &m, m.color().into(), "", true))
                        }
                    }
                };

                game_state.selected = None;

                game_state.possible_moves.clear();

                // update position to contain piece
                Ok(out)
            } else {
                // deselect
                // game_state.possible_moves.clear();

                let out = html! {
                    @for m in &game_state.possible_moves {
                        @if let Some(piece_at) = game_state.board.get_piece(m) {
                            (square(game_id, &m, m.color().into(), piece_at.repr(), true))
                        } @else {
                            (square(game_id, &m, m.color().into(), "", true))
                        }
                    }
                };

                game_state.selected = None;
                game_state.possible_moves.clear();
                game_state.to_move = game_state.to_move.invert();

                Ok(out)
            }
        }
    } else {
        if let Some(piece) = game_state.board.get_piece(&position) {
            debug!("no piece selected: clicked on a piece: {:?}", &piece);
            let moves = piece.possible_moves(&game_state.board);

            game_state.possible_moves = moves;
            game_state.selected = Some(position);

            Ok(html! {
                @for position in &game_state.possible_moves {
                    @if let Some(piece_at) = game_state.board.get_piece(position) {
                        (square(game_id, position, SquareColor::Highlighted, piece_at.repr(), true))
                    } @else {
                        (square(game_id, position, SquareColor::Highlighted, "", true))
                    }
                }
            })
        } else {
            debug!("no piece selected: clicked on an empty square");

            game_state.possible_moves.clear();

            Ok(html! {})
        }
    }
}

enum SquareColor {
    Black,
    White,
    Highlighted,
}

impl From<Color> for SquareColor {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => Self::Black,
            Color::White => Self::White,
        }
    }
}

fn square(
    game_id: Uuid,
    position: &Position,
    square_background_color: SquareColor,
    body: &str,
    swap_oob: bool,
) -> Markup {
    if swap_oob {
        html! {
            div
                id=(format!("square-{}{}", position.column, position.row))
                hx-swap-oob="true"
                hx-put=(format!("/games/{}/play/square_clicked?column={}&row={}", game_id, position.column, position.row))
                hx-swap="none"
                class=(background_color(square_background_color)) {
                (body)
            }
        }
    } else {
        html! {
            div
                id=(format!("square-{}{}", position.column, position.row))
                hx-put=(format!("/games/{}/play/square_clicked?column={}&row={}", game_id, position.column, position.row))
                hx-swap="none"
                class=(background_color(square_background_color)) {
                (body)
            }
        }
    }
}

fn background_color(color: SquareColor) -> &'static str {
    match color {
        SquareColor::Black => {
            "bg-gray-400 flex basis-1/8 aspect-square select-none items-center justify-center"
        }
        SquareColor::White => {
            "bg-gray-50 flex basis-1/8 aspect-square select-none items-center justify-center"
        }
        SquareColor::Highlighted => {
            "bg-pink-400 flex basis-1/8 aspect-square select-none items-center justify-center"
        }
    }
}

struct AppState {
    pool: Pool<Sqlite>,
    games: HashMap<Uuid, GameState>,
}

// TODO figure out how/what to store for each individual game
// such that we can display the currently selected piece, prospective moves, etc.
struct GameState {
    board: Board,
    selected: Option<Position>,
    possible_moves: Vec<Position>,
    takes: Vec<Piece>,
    to_move: Color,
}

#[derive(Parser)]
struct Options {
    #[arg(short, long, env, default_value = "8080")]
    port: u16,
    #[arg(short, long, env, default_value = "chez.db")]
    database: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let options = Options::parse();

    let opts =
        sqlx::sqlite::SqliteConnectOptions::from_str(&format!("sqlite://{}", options.database))?
            .busy_timeout(std::time::Duration::from_secs(5))
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .create_if_missing(true)
            .foreign_keys(true);

    let pool = sqlx::SqlitePool::connect_with(opts).await?;

    let mut conn = pool.acquire().await?;

    let mut tx = conn.begin().await?;

    sqlx::query(
        "
    create table if not exists games (
        id blob primary key,
        taken text,
        inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
    )
    ",
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "
    create table if not exists moves (
        id blob primary key,
        game_id blob not null,
        from_column integer not null,
        from_row integer not null,
        to_column integer not null,
        to_row integer not null,
        inserted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        foreign key(game_id) references games(id)
    )",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let router = Router::new()
        .route("/", get(|| async { Redirect::to("/games/new") }))
        .route("/games/new", get(games_new))
        .route("/games/create", post(games_create))
        .route("/games/{game_id}/play", get(games_play))
        .route("/games/{game_id}/play/square_clicked", put(square_clicked))
        .with_state(Arc::new(Mutex::new(AppState {
            pool,
            games: HashMap::new(),
        })))
        .layer(tower_http::compression::CompressionLayer::new());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", options.port))
        .await
        .unwrap();

    info!("listening on {}", options.port);

    axum::serve(listener, router).await.unwrap();

    Ok(())
}

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
