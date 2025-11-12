use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub balance: i64,
}

#[derive(Debug)]
enum Command {
    AddUser(User, oneshot::Sender<Option<User>>),
    GetUserByEmail(String, oneshot::Sender<Option<User>>),
    GetUserById(String, oneshot::Sender<Option<User>>),
    GetBalance(String, oneshot::Sender<Result<i64, String>>),
    UpdateBalance(String, i64, oneshot::Sender<Result<(), String>>),
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

    pub async fn get_user_by_id(&self, id: String) -> Option<User> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetUserById(id, tx)).await;
        rx.await.ok().flatten()
    }

    pub async fn get_balance(&self, id: String) -> Result<i64, String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBalance(id, tx)).await;
        rx.await.unwrap_or_else(|_| Err("failed to get balance".into()))
    }

    pub async fn update_balance(&self, id: String, amount: i64) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::UpdateBalance(id, amount, tx)).await;
        rx.await.unwrap_or_else(|_| Err("failed to update balance".into()))
    }
}

pub fn spawn_user_actor() -> UserStore {
    let (tx, mut rx) = mpsc::channel::<Command>(1000);

    tokio::spawn(async move {
        let mut users: HashMap<String, User> = HashMap::new();

        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::AddUser(user, reply) => {
                    let id = user.id.clone();
                    users.insert(id.clone(), user.clone());
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
            }
        }
    });

    UserStore { tx }
}
