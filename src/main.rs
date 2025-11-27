use actix_web::{App, HttpResponse, HttpServer, Responder, post, web};
use std::{collections::HashMap, sync::Mutex};
use serde::{Deserialize, Serialize};


// Visit Count 
struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn visit_count(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;

    format!("Visit count: {counter}") 
}

// Music Library
#[derive(Deserialize)]
struct NewSong {
    title: String,
    artist: String, 
    genre: String,
}

#[derive(Deserialize)]
struct SongSearchQuery {
    title: Option<String>,
    artist: Option<String>,
    genre: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Song {
    id: isize,
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
    id: Mutex<isize>,
    songs_library: Mutex<HashMap<isize, Song>>
}

async fn search_song(
    query: web::Query<SongSearchQuery>,
    music_library: web::Data<AppStateWithSong>,
) -> impl Responder{
    let music_library = music_library.songs_library.lock().unwrap();
    let title_q = query.title.as_ref().map(|s| s.to_lowercase());
    let artist_q = query.artist.as_ref().map(|s| s.to_lowercase());
    let genre_q = query.genre.as_ref().map(|s| s.to_lowercase());

    let mut results: Vec<&Song> = music_library
        .values()
        .filter(|song| {
            let title_ok = match &title_q {
                Some(t) => song.title.to_lowercase().contains(t),
                None => true,
            };
            let artist_ok = match &artist_q {
                Some(a) => song.artist.to_lowercase().contains(a),
                None => true,
            };
            let genre_ok = match &genre_q {
                Some(g) => song.genre.to_lowercase().contains(g),
                None => true,
            };

            title_ok && artist_ok && genre_ok
        })
        .collect();
    results.sort_by_key(|song| song.id);
    HttpResponse::Ok().json(results)
}

#[post("/songs/new")]
async fn add_new_song(new_song: web::Json<NewSong>, music_library: web::Data<AppStateWithSong>) -> impl Responder {
    let mut song_id = music_library.id.lock().unwrap();
    let mut music_library = music_library.songs_library.lock().unwrap();
    let song = Song{
        id: *song_id,
        title: new_song.title.clone(),
        artist: new_song.artist.clone(),
        genre: new_song.genre.clone(),
        play_count: 0
    };
    music_library.insert(*song_id, song.clone());
    *song_id += 1;
    HttpResponse::Ok().json(&song)
}

async fn play_song(song_id: web::Path<isize>, music_library: web::Data<AppStateWithSong>) -> impl Responder{
    let mut music_library = music_library.songs_library.lock().unwrap();
    if let Some(song) = music_library.get_mut(&song_id.into_inner()){
        song.play();
        // format!("{}", serde_json::to_string_pretty(&song).unwrap())
        return HttpResponse::Ok().json(song);
    } 
    HttpResponse::NotFound().body("{\"error\":\"Song not found\"}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let song_states = web::Data::new(AppStateWithSong{
        id: Mutex::new(1),
        songs_library: Mutex::new(HashMap::new())
    });

    println!("The server is currently listening on localhost:8080.");
    HttpServer::new(move || 
        App::new()
            .app_data(counter.clone())
            .app_data(song_states.clone())
            .route("/", web::get().to(welcome))
            .route("/count", web::get().to(visit_count))
            .route("/songs/search", web::get().to(search_song))
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


