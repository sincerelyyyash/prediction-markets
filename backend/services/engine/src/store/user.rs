use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;

use crate::types::user_types::User;

#[derive(Debug)]
enum Command {
    AddUser(User, oneshot::Sender<Option<User>>),
    GetUserByEmail(String, oneshot::Sender<Option<User>>),
    GetUserById(u64, oneshot::Sender<Option<User>>),
    GetBalance(u64, oneshot::Sender<Result<i64, String>>),
    UpdateBalance(u64, i64, oneshot::Sender<Result<(), String>>),
    GetPosition(u64, u64, oneshot::Sender<Result<u64, String>>),
    UpdatePosition(u64, u64, i64, oneshot::Sender<Result<(), String>>),
    CheckPositionSufficient(u64, u64, u64, oneshot::Sender<Result<bool, String>>),
}

#[derive(Clone)]
pub struct UserStore {
    tx: mpsc::Sender<Command>,
}

impl UserStore {
    pub async fn add_user(&self, user: User) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::AddUser(user, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_user_by_email(&self, email: String) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserByEmail(email, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_user_by_id(&self, id: u64) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserById(id, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_balance(&self, id: u64) -> Result<i64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBalance(id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("failed to get balance".into()))
    } 

    pub async fn update_balance(&self, id: u64, amount: i64) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::UpdateBalance(id, amount, tx)).await;
        rx.await.unwrap_or_else(|_| Err("failed to update balance".into()))
    }

    pub async fn get_position(&self, user_id: u64, market_id: u64)-> Result<u64, String>{
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPosition(user_id, market_id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to get positions".into()))
    }

    pub async fn update_position(&self, user_id: u64, market_id: u64, amount: i64)-> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::UpdatePosition(user_id, market_id, amount, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to update position".into()))
    }

    pub async fn check_position_sufficient(&self, user_id: u64, market_id: u64, required_qty: u64) -> Result<bool, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CheckPositionSufficient(user_id, market_id, required_qty, tx)).await;
        rx.await.unwrap_or_else(|_| Err("Failed to check position".into()))
    }
}

pub fn spawn_user_actor() -> UserStore {
    let (tx, mut rx) = mpsc::channel::<Command>(1000);

    tokio::spawn(async move {
        let mut users: HashMap<u64, User> = HashMap::new();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::AddUser(user, reply) => {
                    let id = user.id;
                    users.insert(id, user.clone());
                    let _ = reply.send(Some(user));
                }
                Command::GetUserById(id, reply) => {
                    let user = users.get(&id).cloned();
                    let _ = reply.send(user);
                }
                Command::GetUserByEmail(email, reply) => {
                    let user = users.values().find(|u| u.email == email).cloned();
                    let _ = reply.send(user);
                }
                Command::GetBalance(id, reply) => {
                    let res = users
                        .get(&id)
                        .map(|u| Ok(u.balance))
                        .unwrap_or_else(|| Err("User not found".into()));
                    let _ = reply.send(res);
                }
                Command::UpdateBalance(id, amount, reply) => {
                    if let Some(u) = users.get_mut(&id) {
                        u.balance += amount;
                        let _ = reply.send(Ok(()));
                    } else {
                        let _ = reply.send(Err("User not found".into()));
                    }
                }
                Command::GetPosition(user_id, market_id ,reply ) => {
                    let position = users
                    .get(&user_id)
                    .and_then(|u| u.positions.get(&market_id))
                    .copied()
                    .unwrap_or(0);

                    let _ = reply.send(Ok(position));
                }
                Command::UpdatePosition(user_id,market_id ,amount ,reply )=>{
                    if let Some(user) = users.get_mut(&user_id) {
                        let current = user.positions.entry(market_id).or_insert(0);

                        if amount < 0 && (*current as i64) < -amount {
                            let _ = reply.send(Err("Insufficient position".into()));
                        } else {
                            *current = ((*current as i64) + amount) as u64;
                            if *current == 0 {
                                user.positions.remove(&market_id);
                            }
                        }
                      } else {
                        let _ = reply.send(Err("User not found".into()));
                      }
                }
                Command::CheckPositionSufficient(user_id,market_id , required_qty ,reply )=> {
                    let position = users
                    .get(&user_id)
                    .and_then(|u| u.positions.get(&market_id))
                    .copied()
                    .unwrap_or(0);
                
                    let _ = reply.send(Ok(position >= required_qty));
                }
            }
        }
    });

    UserStore { tx }
}
