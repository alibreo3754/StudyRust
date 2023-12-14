use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

use bytes::Bytes;

#[derive(Debug)]
pub enum Command {
  Get {
    key: String,
    resp: Responder<Option<Bytes>>,
  },
  Set {
    key: String,
    val: Bytes,
    resp: Responder<()>,
  }
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
  let (tx, mut rx) = mpsc::channel(32);
  let manager = tokio::spawn(async move {
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    while let Some(cmd) = rx.recv().await {
      use Command::*;
      match cmd {
        Get { key, resp } => {
          let res = client.get(&key).await;
          let _ = resp.send(res);
        }
        Set { key, val , resp } => {
          let res = client.set(&key, val).await;
          let _ = resp.send(res);
        }
      }

    }
  });

  let tx2 = tx.clone();


  let tx3 = tx.clone();
  let t1 = tokio::spawn(async move {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::Get { 
      key: "foo".to_string(),
      resp: resp_tx,
    };

    tx.send(cmd).await.unwrap();

    let res = resp_rx.await;
    println!("GOT = {:?}", res);
  });

  let t2 = tokio::spawn(async move {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::Set {
      key: "foo".to_string(),
      val: "bar".into(),
      resp: resp_tx,
    };

    tx2.send(cmd).await.unwrap();

    let res = resp_rx.await;
    println!("Got = {:?}", res);
  });

  manager.await.unwrap();
  t1.await.unwrap();
  t2.await.unwrap();
}

