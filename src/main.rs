use std::io::prelude::*;
use std::io::{self, Read};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Team {
    Red,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Attack,
    Defend,
}

struct Player {
    id: u8,
    stream: TcpStream,
    ready: bool,
    team: Team,
    health: i8,
}
fn main() -> std::io::Result<()> {
    let mut team_black = Vec::<Player>::new();
    let mut team_red = Vec::<Player>::new();
    let listener = TcpListener::bind("127.0.0.1:1370")?;
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
                thread::spawn(move || {
                    single_connection(conn.0, tx_clone, id as u8);
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => panic!("encountered IO error: {}", e),
        }
        for receiver in rx.try_iter() {
            ready_check[receiver.id as usize] = receiver.ready;
            if receiver.team == Team::Black {
                team_black.push(receiver);
            } else {
                team_red.push(receiver);
            }
        }
        ready = !ready_check.contains(&false) && !ready_check.is_empty();
    }
    let black_length = team_black.len();
    let red_length = team_red.len();
    let (tx_action, rx_action) = mpsc::channel();
    let current_turn = Team::Black;
    loop {
        for player in &mut team_red {
            player.stream.write(b"Black turn, waiting...")?;
        }
        if current_turn == Team::Black {
            for player in team_black {
                let tx_clone = tx.clone();
                let tx_action = tx_action.clone();
                thread::spawn(move || {
                    turn(player, tx_clone, tx_action);
                });
            }
            for _ in 0..black_length {
                let (action, id) = rx_action.recv().unwrap();
                if action == Action::Attack {
                    let target = &mut team_red[id.unwrap() as usize];
                    target.health -= 10;
                    target.stream.write(
                        format!("You took 10 damage! Health remaining: {}", target.health)
                            .as_bytes(),
                    )?;
                }
            }
            team_black = Vec::new();
            for _ in 0..black_length {
                let response = rx.recv().unwrap();
                team_black.push(response);
            }
            for player in &mut team_black {
                player.stream.write(b"All done.")?;
            }
        } else {
            let teams = team_turn(team_red, &tx, &rx, team_black);
            team_red = teams.0;
            team_black = teams.1;
        }
    }

    Ok(())
}

fn team_turn(
    mut initial_vector: Vec<Player>,
    tx: &Sender<Player>,
    rx: &Receiver<Player>,
    mut opposing_vector: Vec<Player>,
) -> (Vec<Player>, Vec<Player>) {
    let length = initial_vector.len();
    let (tx_action, rx_action) = mpsc::channel();
    for player in &mut opposing_vector {
        player.stream.write(b"Enemy turn, waiting...").unwrap();
    }
    for player in initial_vector {
        if player.health > 0 {
            let tx_clone = tx.clone();
            let tx_action = tx_action.clone();
            thread::spawn(move || {
                turn(player, tx_clone, tx_action);
            });
        }
    }
    for _ in 0..length {
        let (action, id) = rx_action.recv().unwrap();
        if action == Action::Attack {
            let target = &mut opposing_vector[id.unwrap() as usize];
            target.health -= 10;
            target.stream.write(b"You took 10 damage!").unwrap();
        }
    }
    initial_vector = Vec::new();
    for _ in 0..length {
        let response = rx.recv().unwrap();
        initial_vector.push(response);
    }
    for player in &mut initial_vector {
        player.stream.write(b"All done.").unwrap();
    }
    return (initial_vector, opposing_vector);
}

fn turn(mut player: Player, player_tx: Sender<Player>, tx: Sender<(Action, Option<u8>)>) {
    player.stream.set_nonblocking(false).unwrap();
    player
        .stream
        .write(b"Attack [attack] or defend [defend]?")
        .unwrap();
    let mut buffer = [0; 1024];
    player.stream.read(&mut buffer).unwrap();
    if &buffer[0..6] == b"attack" {
        player.stream.write(b"Who? [id]").unwrap();
        let mut buffer = [0; 1];
        player.stream.read(&mut buffer).unwrap();
        let id = buffer[0] - 48;
        println!("Here {}", id);
        tx.send((Action::Attack, Some(id))).unwrap();
    } else if &buffer[0..6] == b"defend" {
    }
    thread::sleep(Duration::from_millis(30));
    player_tx.send(player).unwrap();
}

fn single_connection(mut stream: TcpStream, tx: Sender<Player>, id: u8) {
    let team: Team;
    loop {
        stream.write(b"team?").unwrap();
        let mut buffer = [0; 5];
        stream.read(&mut buffer).unwrap();
        if &buffer == b"black" {
            team = Team::Black;
            break;
        } else if &buffer[0..3] == b"red" {
            team = Team::Red;
            break;
        }
        thread::sleep(Duration::from_millis(30));
    }
    println!("Team: {:?}", team);
    loop {
        stream.write(b"ready?").unwrap();
        let mut buffer = [0; 5];
        stream.read(&mut buffer).unwrap();
        let stream = stream.try_clone().unwrap();
        if &buffer == b"ready" {
            println!("ready!");
            tx.send(Player {
                stream,
                id,
                health: 100,
                ready: true,
                team,
            })
            .unwrap();
            break;
        } else {
            tx.send(Player {
                stream,
                id,
                health: 100,
                ready: false,
                team,
            })
            .unwrap();
        }
        thread::sleep(Duration::from_millis(30));
    }
    stream.write(b"Waiting...").unwrap();
    loop {}
}
