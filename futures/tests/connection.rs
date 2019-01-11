// Long and nested future chains can quickly result in large generic types.
#![type_length_limit="2097152"]

#[macro_use] extern crate log;
extern crate lapin_futures as lapin;
extern crate failure;
extern crate futures;
extern crate tokio;
extern crate env_logger;

use failure::Error;
use futures::Stream;
use futures::future::Future;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;

use lapin::types::FieldTable;
use lapin::client::ConnectionOptions;
use lapin::channel::{BasicConsumeOptions,BasicPublishOptions,BasicQosOptions,BasicProperties,QueueDeclareOptions,QueueDeleteOptions,QueuePurgeOptions};

#[test]
fn connection() {
  let _ = env_logger::try_init();

  let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "127.0.0.1:5672".to_string()).parse().unwrap();

  Runtime::new().unwrap().block_on_all(
    TcpStream::connect(&addr).map_err(Error::from).and_then(|stream| {
      lapin::client::Client::connect(stream, ConnectionOptions::default()).map_err(Error::from)
    }).and_then(|(client, _)| {

      client.create_channel().and_then(|channel| {
        let id = channel.id;
        info!("created channel with id: {}", id);

        channel.queue_declare("hello", QueueDeclareOptions::default(), FieldTable::new()).and_then(move |_| {
          info!("channel {} declared queue {}", id, "hello");

          channel.queue_purge("hello", QueuePurgeOptions::default()).and_then(move |_| {
            channel.basic_publish("", "hello", b"hello from tokio".to_vec(), BasicPublishOptions::default(), BasicProperties::default())
          })
        })
      }).and_then(move |_| {
        client.create_channel()
      }).and_then(|channel| {
        let id = channel.id;
        info!("created channel with id: {}", id);

        let ch1 = channel.clone();
        let ch2 = channel.clone();
        channel.basic_qos(BasicQosOptions { prefetch_count: 16, ..Default::default() }).and_then(move |_| {
          info!("channel QoS specified");
          channel.queue_declare("hello", QueueDeclareOptions::default(), FieldTable::new()).map(move |queue| (channel, queue))
        }).and_then(move |(channel, queue)| {
          info!("channel {} declared queue {}", id, "hello");

          channel.basic_consume(&queue, "my_consumer", BasicConsumeOptions::default(), FieldTable::new())
        }).and_then(move |stream| {
          info!("got consumer stream");

          stream.into_future().map_err(|(err, _)| err).and_then(move |(message, _)| {
            let msg = message.unwrap();
            info!("got message: {:?}", msg);
            assert_eq!(msg.data, b"hello from tokio");
            ch1.basic_ack(msg.delivery_tag, false)
          }).and_then(move |_| {
            ch2.queue_delete("hello", QueueDeleteOptions::default())
          })
        })
      }).map_err(Error::from)
    })
  ).expect("runtime failure");
}
