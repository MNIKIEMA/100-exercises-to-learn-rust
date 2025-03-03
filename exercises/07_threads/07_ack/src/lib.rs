use std::sync::mpsc::{Receiver, Sender};
use crate::store::TicketStore;

pub mod data;
pub mod store;

use data::{TicketDraft, Ticket};
use store::{TicketId};

// Refer to the tests to understand the expected schema.
pub enum Command {
    Insert { draft: TicketDraft, response_sender: Sender<TicketId>},
    Get { id: TicketId, response_sender: Sender<Option<Ticket>>}
}

pub fn launch() -> Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || server(receiver));
    sender
}

// TODO: handle incoming commands as expected.
pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {draft, response_sender}) => {
                let ticket_id = store.add_ticket(draft);
                if let Err(_) = response_sender.send(ticket_id) {
                    println!("Failed to send ticket ID.");
                }
            }
            Ok(Command::Get {
                id, response_sender
            }) => {
                let ticket: Option<&data::Ticket> = store.get(id);
                if let Err(_) = response_sender.send(ticket.cloned()) {
                    println!("Failed to send ticket.");
                }
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break
            },
        }
    }
}
