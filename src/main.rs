extern crate websocket;

use std::str;
use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::header::WebSocketProtocol;

fn main() {
  let server = Server::bind("127.0.0.1:2794").unwrap();

  for connection in server {
    // Spawn a new thread for each connection.
    thread::spawn(move || {
      // Get the request
      let request = connection.unwrap().read_request().unwrap();
      // Keep the headers so we can check them
      let headers = request.headers.clone();

      // Validate the request
      request.validate().unwrap();

      // Form a response
      let mut response = request.accept();

      if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
        if protocols.contains(&("zuul-log-streamer".to_string())) {
          // We have a protocol we want to use
          response.headers.set(
            WebSocketProtocol(vec!["zuul-log-streamer".to_string()]));
        }
      }

      // Send the response
      let mut client = response.send().unwrap();

      let ip = client.get_mut_sender()
        .get_mut()
        .peer_addr()
        .unwrap();

      println!("Connection from {}", ip);

      let message: Message = Message::text("Hello".to_string());
      client.send_message(&message).unwrap();

      let (mut sender, mut receiver) = client.split();

      for message in receiver.incoming_messages() {
        let message: Message = message.unwrap();

        match message.opcode {
          Type::Close => {
            let message = Message::close();
            sender.send_message(&message).unwrap();
            println!("Client {} disconnected", ip);
            return;
          },
          Type::Ping => {
            let message = Message::pong(message.payload);
            sender.send_message(&message).unwrap();
          },
          _ => {
            println!("Msg {}", str::from_utf8(&message.payload).unwrap());
            sender.send_message(&message).unwrap();
          },
        }
      }
    });
  }
}
