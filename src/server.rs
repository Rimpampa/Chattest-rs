use crate::*;
use std::io::ErrorKind;
use std::net::TcpListener;
use std::sync::{Arc, RwLock};
use std::thread;

pub fn chat(win: &Window, name: String) -> bool {
    // Get the name of the room from the user
    win.printw("  What's the name of this room?\n  [press ESC to return to menu]\n > ");
    let room = Arc::new(match get_string(&win) {
        (string, false) => string,
        (_, true) => return false,
    });

    let arc_name = Arc::new(name);

    // Print out the information of the room
    win.mvprintw(
        0,
        0,
        format!("  Room name: {}\n  Admin: {}\n", room, *arc_name),
    );
    // Remove the previous text
    win.clrtobot();
    win.refresh();

    // Bind the listener to the port 7357
    let listener = Arc::new(TcpListener::bind("0.0.0.0:7357").unwrap());

    // Vector that stores all the incoming messages
    let messages = Arc::new(RwLock::new(Vec::new()));

    // Vector of the connected clients and their names
    let clients = Arc::new(RwLock::new(Vec::new()));

    accept_thread(&listener, &messages, &clients, &room, &arc_name);
    clients_thread(&messages, &clients);

    win.nodelay(true);
    win.mvprintw(LAST, 0, " > ");

    let mut last = messages.read().unwrap().len();
    let mut index = 0;

    let mut string = String::new();
    let mut cursor = 0;
    loop {
        let rmsgs = messages.read().unwrap();
        if rmsgs.len() > last {
            if index + 1 == last || last == 0 {
                index = rmsgs.len() - 1;

                win.mvprintw(3, 0, format!("{}", rmsgs[index]));
                win.clrtobot();
                win.mvprintw(LAST, 0, " > ");
                win.printw(&string);
                win.mv(LAST, 3 + cursor as i32);
            }
            last = rmsgs.len();
            println!(
                "New messages (last): {} (index: {}, len: {})",
                rmsgs.last().unwrap(),
                index,
                last
            );
            win.refresh();
        }
        if let Some(input) = try_get_string(&win, &mut string, &mut cursor) {
            match input {
                Input::Character('\n') if string.len() > 1 => {
                    let mut mut_clients = clients.write().unwrap();
                    for i in 0..mut_clients.len() {
                        println!("Sending message {} to {}", string, mut_clients[i].1);
                        match mut_clients[i]
                            .0
                            .write(chattest::Code::MessageTo(string.clone()))
                        {
                            Ok(()) => println!("Ok!"),
                            Err(error) => println!("Err! {:?}", error),
                        }
                    }
                    std::mem::drop(rmsgs);
                    let mut wmsgs = messages.write().unwrap();
                    wmsgs.push(format!("  {}", string));

                    string.clear();
                    cursor = 0;
                    win.mv(HEIGHT - 3, 3);
                    win.clrtobot();
                }
                Input::KeyUp if index > 0 => {
                    println!("UP!");
                    index -= 1;
                    win.mvprintw(3, 0, &rmsgs[index]);
                    win.clrtobot();
                    win.mvprintw(LAST, 0, " > ");
                    win.printw(&string);
                    win.mv(LAST, 3 + cursor as i32);
                }
                Input::KeyDown if index + 1 < last => {
                    println!("DOWN!");
                    index += 1;
                    win.mvprintw(3, 0, &rmsgs[index]);
                    win.clrtobot();
                    win.mvprintw(LAST, 0, " > ");
                    win.printw(&string);
                    win.mv(LAST, 3 + cursor as i32);
                }
                _ => (),
            }
            win.refresh();
        }
    }
}

fn find_string(vec: &[(chattest::NonBlockingStream, String)], val: &str) -> bool {
    for (_, string) in vec.iter() {
        if string == val {
            return true;
        }
    }
    false
}

fn accept_thread(
    listener: &Arc<TcpListener>,
    messages: &Arc<RwLock<Vec<String>>>,
    clients: &Arc<RwLock<Vec<(chattest::NonBlockingStream, String)>>>,
    room: &Arc<String>,
    arc_name: &Arc<String>,
) {
    let messages = Arc::clone(messages);
    let listener = Arc::clone(listener);
    let clients = Arc::clone(clients);
    let room = Arc::clone(room);
    let arc_name = Arc::clone(arc_name);
    thread::spawn(move || loop {
        // Wait for a client to connect
        match listener.accept() {
            // When the client connectes:
            Ok((stream, addr)) => {
                // Make the stream a chattest BlockingStream
                let mut stream = chattest::BlockingStream::new(stream);
                loop {
                    // Get the client name:
                    match stream.read() {
                        // If he sends a chattest message match the code
                        Ok(code) => match code {
                            // If he sends his name:
                            chattest::Code::Name(name) => {
                                // Check if there is noone else with that name
                                if name != **arc_name
                                    && !find_string(&clients.read().unwrap(), &name)
                                {
                                    // Tell the client the name of the room
                                    stream
                                        .write(chattest::Code::Welcome(
                                            (*room).clone(),
                                            (*arc_name).clone(),
                                        ))
                                        .unwrap();
                                    println!("Connected: {}({})", name, addr);
                                    // Comunicate the new connection:
                                    messages
                                        .write()
                                        .unwrap()
                                        .push(format!("  User connected:\n  {}({})", name, addr));
                                    // Lock the vector of clients
                                    let mut mut_clients = clients.write().unwrap();
                                    // Comunicating the event to the other clients
                                    for j in 0..mut_clients.len() {
                                        println!(
                                            "Sending User {} connected! to {}",
                                            name, mut_clients[j].1
                                        );
                                        match mut_clients[j].0.write(chattest::Code::MessageTo(
                                            format!("User {} connected!", name),
                                        )) {
                                            Ok(()) => println!("Ok!"),
                                            Err(error) => println!("Err! {:?}", error),
                                        }
                                    }
                                    // Push the new client in the list
                                    mut_clients.push((stream.non_blocking(), name));
                                    break;
                                }
                                println!("Already here: {}", name);
                                // Else tell him to use another name
                                stream.write(chattest::Code::AlreadyHere).unwrap();
                            }
                            _ => println!("Code not expected: {:?}", code),
                        },
                        Err(error) => println!("Read error: {}", error),
                    }
                }
            }
            Err(error) => println!("Accept error: {}", error),
        }
    });
}

fn clients_thread(
    messages: &Arc<RwLock<Vec<String>>>,
    clients: &Arc<RwLock<Vec<(chattest::NonBlockingStream, String)>>>,
) {
    let messages = Arc::clone(messages);
    let clients = Arc::clone(clients);
    thread::spawn(move || loop {
        // Lock the clients vector
        let mut mut_clients = clients.write().unwrap();

        // For every client:
        for i in 0..mut_clients.len() {
            // Get the name of the client
            let name = mut_clients[i].1.clone();
            // Try to get his message
            match mut_clients[i].0.try_read() {
                // If his message arrived match the code:
                Ok(Some(code)) => match code {
                    // If it's a text message
                    chattest::Code::MessageTo(text) => {
                        println!("Recived {} from {}", text, name);
                        // Print the message
                        messages
                            .write()
                            .unwrap()
                            .push(format!("  {}> {}", name, text));
                        // Send the message to the other clients
                        for j in 0..mut_clients.len() {
                            // Exclude the current client
                            if j != i {
                                println!("Sending {} to {}", text, mut_clients[j].1);
                                match mut_clients[j]
                                    .0
                                    .write(chattest::Code::MessageFrom(name.clone(), text.clone()))
                                {
                                    Ok(()) => println!("Ok!"),
                                    Err(error) => println!("Err! {:?}", error),
                                }
                            }
                        }
                    }
                    _ => println!("Code not expected from {}: {:?}", name, code),
                },
                // If the message is not complete, go to the next client
                Ok(None) => (),
                // If there was an error:
                Err(error) => match error.kind() {
                    // If the client discnnected:
                    ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
                        // Comunicating the event to the other clients
                        for j in 0..mut_clients.len() {
                            // Exclude the current client
                            if j != i {
                                println!(
                                    "Sending User {} disconnected! to {}",
                                    name, mut_clients[j].1
                                );
                                match mut_clients[j].0.write(chattest::Code::MessageTo(format!(
                                    "User {} disconnected!",
                                    name
                                ))) {
                                    Ok(()) => println!("Ok!"),
                                    Err(error) => println!("Err! {:?}", error),
                                }
                            }
                        }
                        // Remove it from the list
                        mut_clients.remove(i);
                        println!("Disconnected {}", name);
                        // Comunicate the event
                        messages
                            .write()
                            .unwrap()
                            .push(format!("  User {} disconnected!", name));
                        break;
                    }
                    _ => println!("Error with client {}: {}({:?})", name, error, error.kind()),
                },
            }
        }
    });
}
