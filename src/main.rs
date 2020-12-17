use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{self, Read};
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::{Serialize, Deserialize};
use serde::ser::{Serializer, SerializeStruct};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
enum Team {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Attack,
    Heal,
}

#[derive(Debug, Clone)]
struct Player {
    id: u8,
    name: String,
    stream: Arc<Mutex<TcpStream>>,
    ready: bool,
    team: Team,
    health: i8,
}

impl Serialize for Player { // Can't serialize TCPStream so it's custom serializer time
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Player", 5)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("ready", &self.ready)?;
        state.serialize_field("team", &self.team)?;
        state.serialize_field("health", &self.health)?;
        state.end()
    }
}

#[derive(Serialize, Debug, Clone)]
struct GameState {
    team_red: HashMap<String, Player>,
    team_black: HashMap<String, Player>,
    current_turn: Team,
}

fn main() {
    let mut game = GameState {
        team_red: HashMap::<String, Player>::new(),
        team_black: HashMap::<String, Player>::new(),
        current_turn: Team::Black,
    };
    setup(&mut game);
    loop {
        if game.current_turn == Team::Black {
            let (black, red) = team_turn(game.team_black, game.team_red);
            game.team_black = black;
            game.team_red = red;
            game.current_turn = Team::Red;
        } else {
            let (red, black) = team_turn(game.team_red, game.team_black);
            game.team_red = red;
            game.team_black = black;
            game.current_turn = Team::Black;
        }
        println!("updating..");
        update(&game);
        thread::sleep(Duration::from_millis(30));
    }
}

fn setup(game: &mut GameState) {
    let listener = TcpListener::bind("127.0.0.1:1370").unwrap();
    let mut ready = false;
    let (tx, rx) = mpsc::channel();
    let mut ready_check = Vec::new();
    while !ready {
        thread::sleep(Duration::from_millis(30));
        listener.set_nonblocking(true).unwrap();
        let connection = listener.accept();
        match connection {
            Ok(conn) => {
                ready_check.push(false);
                let id = ready_check.len() - 1;
                let tx_clone = tx.clone();
                conn.0.set_nodelay(true).unwrap();
                thread::spawn(move || {
                    single_connection(conn.0, tx_clone, id as u8);
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("encountered IO error: {}", e),
        }
        for receiver in rx.try_iter() {
            ready_check[receiver.id as usize] = receiver.ready;
            if receiver.ready {
                if receiver.team == Team::Black {
                    game.team_black.insert(receiver.name.clone(), receiver);
                } else if receiver.team == Team::Red {
                    game.team_red.insert(receiver.name.clone(), receiver);
                }
            }
        }
        ready = !ready_check.contains(&false) && !ready_check.is_empty();
    }
    update(&game);
}

fn team_turn(
    initial_map: HashMap<String, Player>,
    opposing_map: HashMap<String, Player>,
) -> (HashMap<String, Player>, HashMap<String, Player>) {
    let (tx_action, rx_action) = mpsc::channel();
    let mut opposing_map = opposing_map.clone();

    let new_map: HashMap<String, Player> = initial_map
        .into_iter()
        .map(|pair| {
            let (name, mut player) = pair;
            if player.health > 0 {
                let tx_action = tx_action.clone();
                let player_clone = player.clone();
                thread::spawn(move || {
                    turn(player_clone, tx_action);
                });
            }

            let (action, target) = rx_action.recv().unwrap();
            if action == Action::Attack {
                let target = target.unwrap();
                if let Some(target) = opposing_map.get_mut(&target) {
                    target.health -= 10;
                    println!("Turn over")
                } else {
                    println!("That person doesn't exist!")
                }
            } else if action == Action::Heal {
                player.health += 5;
            }
            (name, player)
        })
        .collect();
    return (new_map, opposing_map);
}

fn turn(player: Player, tx: Sender<(Action, Option<String>)>) {
    let stream = player.stream.lock().unwrap();
    stream.set_nonblocking(false).unwrap();
    drop(stream);
    loop {
        let mut stream = player.stream.lock().unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        if &buffer[0..6] == b"attack" {
            let name = read_string(&mut stream);
            tx.send((Action::Attack, Some(name))).unwrap();
            break;
        } else if &buffer[0..4] == b"heal" {
            tx.send((Action::Heal, None)).unwrap();
            break;
        }
        drop(stream);
        thread::sleep(Duration::from_millis(30));
    }
}

fn single_connection(mut stream: TcpStream, tx: Sender<Player>, id: u8) {
    let team: Team;
    loop {
        stream.write(b"team? \n").unwrap();
        let mut buffer = [0; 1024];
        println!("About to read team");
        stream.read(&mut buffer).unwrap();
        if &buffer[0..5] == b"black" {
            team = Team::Black;
            break;
        } else if &buffer[0..3] == b"red" {
            team = Team::Red;
            break;
        }
        thread::sleep(Duration::from_millis(30));
    }

    stream.write(b"name? (Max 1028 bytes) \n").unwrap();
    let name = read_string(&mut stream);
    println!("name {} ", name);

    loop {
        stream.write(b"ready? \n").unwrap();
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let stream = stream.try_clone().unwrap();
        if &buffer[0..5] == b"ready" {
            println!("ready!");
            tx.send(Player {
                name: name.clone(),
                stream: Arc::new(Mutex::new(stream)),
                id,
                health: 100,
                ready: true,
                team,
            })
            .unwrap();
            break;
        } else {
            tx.send(Player {
                name: name.clone(),
                stream: Arc::new(Mutex::new(stream)),
                id,
                health: 100,
                ready: false,
                team,
            })
            .unwrap();
        }
        thread::sleep(Duration::from_millis(30));
    }
}

fn read_string(stream: &mut TcpStream) -> std::string::String {
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer).unwrap();
    str::from_utf8(&buffer[0..size]).unwrap().to_string()
}

fn update(game: &GameState) {
    let json = format!("{}\n", serde_json::to_string(game).unwrap());
    println!("{}", json);
    let json_bytes = json.as_bytes();
    let team_black = game.team_black.clone();
    let team_red = game.team_red.clone();
    for (_name, player) in team_black {
        let mut stream = player.stream.lock().unwrap();
        stream.write(json_bytes).unwrap();
    }

    for (_name, player) in team_red {
        let mut stream = player.stream.lock().unwrap();
        stream.write(json_bytes).unwrap();
    }
}
