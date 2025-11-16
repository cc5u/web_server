use actix_web::{App, HttpServer, Responder, post, web};
use std::{collections::HashMap, sync::Mutex};
use serde::{Deserialize, Serialize};
struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn visit_count(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Visit count: {counter}") // <- response with count
}


#[derive(Deserialize)]
struct NewSong {
    title: String,
    artist: String, 
    genre: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Song {
    song_id: isize,
    title: String,
    artist: String, 
    genre: String,
    play_count: isize
}

impl Song{
    fn play(&mut self){
        self.play_count += 1;
    }
}

struct AppStateWithSong {
    song_id: Mutex<isize>,
    songs_library: Mutex<HashMap<isize, Song>>
}

/// deserialize `Info` from request's body
#[post("/songs/new")]
async fn add_new_song(new_song: web::Json<NewSong>, music_library: web::Data<AppStateWithSong>) -> String {
    let mut song_id = music_library.song_id.lock().unwrap();
    let mut music_library = music_library.songs_library.lock().unwrap();
    let song = Song{
        song_id: *song_id,
        title: new_song.title.clone(),
        artist: new_song.artist.clone(),
        genre: new_song.genre.clone(),
        play_count: 0
    };
    music_library.insert(*song_id, song.clone());
    *song_id += 1;
    format!("{}", serde_json::to_string_pretty(&song).unwrap())
}

async fn play_song(song_id: web::Path<isize>, music_library: web::Data<AppStateWithSong>) -> String{
    let mut music_library = music_library.songs_library.lock().unwrap();
    if let Some(song) = music_library.get(&song_id){
        // song.play();
        format!("{}", serde_json::to_string_pretty(&song).unwrap())
    } else {
        format!("{{\"error\":\"Song not found\"}}")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let song_states = web::Data::new(AppStateWithSong{
        song_id: Mutex::new(0),
        songs_library: Mutex::new(HashMap::new())
    });

    println!("The server is currently listening on localhost:8080.");
    HttpServer::new(move || 
        App::new()
            .app_data(counter.clone())
            .app_data(song_states.clone())
            .route("/", web::get().to(welcome))
            .route("/count", web::get().to(visit_count))
            .route("/songs/play/{song_id}", web::get().to(play_song))
            .service(add_new_song)
    )
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn welcome() -> impl Responder{
    "Welcome to the Rust-powered web server!"
}


