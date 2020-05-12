use failure::{Fallible};
use iota_streams::app_channels::{
    api::tangle::{Address, Message},
};
use crate::transport::AsyncTransport;

pub async fn recv_messages<T>(transport: &mut T, addr: &Address) -> Fallible<Vec<Message>>
where
    T: AsyncTransport,
    <T>::RecvOptions: Copy + Default + Send,
{
    transport
        .recv_messages_with_options(addr, T::RecvOptions::default())
        .await
}

pub async fn recv_message<T>(transport: &mut T, addr: &Address) -> Fallible<Option<Message>>
where
    T: AsyncTransport + Send,
    <T>::RecvOptions: Copy + Default + Send,
{
    transport.recv_message(addr).await
}

pub async fn send_message<T>(transport: &mut T, message: &Message) -> Fallible<()>
where
    T: AsyncTransport + Send,
    <T>::SendOptions: Copy + Default + Send,
{
    transport.send_message(message).await?;
    Ok(())
}

