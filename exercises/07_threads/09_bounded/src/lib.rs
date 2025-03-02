// TODO: Convert the implementation to use bounded channels.
use crate::data::{Ticket, TicketDraft};
use crate::store::{TicketId, TicketStore};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SyncSender};

pub mod data;
pub mod store;

#[derive(Clone)]
pub struct TicketStoreClient {
    sender: SyncSender<Command>,
}

impl TicketStoreClient {
    pub fn insert(&self, draft: TicketDraft) -> Result<TicketId, mpsc::SendError<Command>> {
        let (response_sender, response_receiver) = mpsc::channel();
        let command = Command::Insert {
            draft,
            response_channel: response_sender,
        };
        self.sender.send(command.clone())?;
        response_receiver.recv().map_err(|_| mpsc::SendError(command))
    }

    pub fn get(&self, id: TicketId) -> Result<Option<Ticket>, mpsc::SendError<Command>> {
        let (response_sender, response_receiver) = mpsc::channel();
        let command = Command::Get {
            id,
            response_channel: response_sender,
        };
        self.sender.send(command.clone())?;
        response_receiver.recv().map_err(|_| mpsc::SendError(command))
    }
}

pub fn launch(capacity: usize) -> TicketStoreClient {
    let (sender, receiver) = std::sync::mpsc::sync_channel(capacity);
    std::thread::spawn(move || server(receiver));
    TicketStoreClient { sender }
}

#[derive(Clone)]
pub enum Command {
    Insert {
        draft: TicketDraft,
        response_channel: Sender<TicketId>,
    },
    Get {
        id: TicketId,
        response_channel: Sender<Option<Ticket>>,
    },
}

pub fn server(receiver: Receiver<Command>) {
    let mut store = TicketStore::new();
    loop {
        match receiver.recv() {
            Ok(Command::Insert {
                draft,
                response_channel,
            }) => {
                let id = store.add_ticket(draft);
                let _ = response_channel.send(id);
            }
            Ok(Command::Get {
                id,
                response_channel,
            }) => {
                let ticket = store.get(id);
                let _ = response_channel.send(ticket.cloned());
            }
            Err(_) => {
                // There are no more senders, so we can safely break
                // and shut down the server.
                break;
            }
        }
    }
}
