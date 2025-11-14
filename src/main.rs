use actix_web::{App, HttpServer, Responder, Result, post, web};
use std::sync::Mutex;
use serde::Deserialize;
struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn visit_count(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Visit count: {counter}") // <- response with count
}


#[derive(Deserialize)]
struct Song {
    // id: isize,
    title: String,
    artist: String, 
    genre: String,
    // play_count: isize
}

struct AppStateWithSong {
    id: Mutex<isize>,
    play_count: Mutex<isize>
}

/// deserialize `Info` from request's body
#[post("/songs/new")]
async fn add_new_song(song: web::Json<Song>) -> Result<String> {
    Ok(format!("\"title\":{},\"artist\":{},\"genre\":{}", song.title, song.artist, song.genre))
    // Ok(format!("{\"id\":{},\"title\":{},\"artist\":{},\"genre\":{},\"play_count\":{}}", song.id, song.title, song.artist, song.genre, song.play_count))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let song_states = web::Data::new(AppStateWithSong{
        id: Mutex::new(0),
        play_count: Mutex::new(0)
    });

    println!("The server is currently listening on localhost:8080.");
    HttpServer::new(move || 
        App::new()
            .app_data(counter.clone())
            // .app_data(song_states.clone())
            .route("/", web::get().to(welcome))
            .route("/count", web::get().to(visit_count))
            .service(add_new_song)
    )
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn welcome() -> impl Responder{
    "Welcome to the Rust-powered web server!"
}


