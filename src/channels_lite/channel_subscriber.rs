#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use iota::Client;
use iota_streams::app_channels::{
    api::tangle::{ Address,Subscriber, DefaultTW, Message, Preparsed}
    , message
};
use iota_streams::app::transport::Transport;
use iota_streams::app::transport::tangle::client::SendTrytesOptions;
use failure::{Fallible, ensure};


use crate::messaging::async_messaging;

pub struct Channel{
    subscriber: Subscriber,
    is_connected: bool,
    client: iota::Client<'static>,
    announcement_link: Address,
    subscription_link: Address,
    channel_address: String,
}

impl Channel {

    pub fn new(seed: &str, node_ulr: &'static str, channel_address: String, announcement_tag: String) -> Channel{

        let subscriber = Subscriber::new(seed, true);

        Self {
            subscriber: subscriber,
            is_connected:false,
            client: iota::Client::new(node_ulr),
            announcement_link: Address::from_str(&channel_address, &announcement_tag).unwrap(),
            subscription_link: Address::default(),
            channel_address: channel_address,
        }
    }

    pub async fn connect(&mut self) -> Result<String,&str>{

        println!("Receiving announcement messages");
     
        let message_list = async_messaging::recv_messages(&mut self.client, &self.announcement_link).await.unwrap();

        let mut found_valid_msg = false;
        for tx in message_list.iter(){
            let header_opt = match tx.parse_header(){
                Ok(val) => Some(val),
                Err(e) => {
                    println!("Parsing Error Header: {}", e);
                    None
                }
            };
            match header_opt {
                None => println!("Invalid message"),
                Some(header) => {
                    if header.check_content_type(message::announce::TYPE){
                        self.subscriber.unwrap_announcement(header.clone()).unwrap();
                        println!("Found and verified {} message", header.content_type());
                        found_valid_msg = true;
                        break;
                    }
                }
            }
        }
        
        if found_valid_msg {
            println!("Subscribing to channel");
        
            let subscribe_link = {
                let msg = self.subscriber.subscribe(&self.announcement_link).unwrap();
                async_messaging::send_message(&mut self.client, &msg).await.unwrap();
                println!("Subscribed to the channel");
                msg.link.clone()
            };

            self.subscription_link =  subscribe_link;
            self.is_connected = true;
        }else{
            println!("No valid announce message found");
        }
        Ok(self.subscription_link.msgid.to_string())
    }

    pub async fn disconnect(&mut self) -> Result<String,&str>{

        let unsubscribe_link = {
            let msg = self.subscriber.unsubscribe(&self.subscription_link).unwrap();
            async_messaging::send_message(&mut self.client, &msg).await.unwrap();
            msg.link.msgid
        };
        println!("Unsubscribed from channel");
        
        Ok(unsubscribe_link.to_string())
    }

    pub async fn read_signed(&mut self) -> Result<Vec<(String,String)>,&str>{

        println!("Receiving signed messages");

        let mut response:Vec<(String,String)> = Vec::new();
    
        if self.is_connected {

            let message_list = async_messaging::recv_messages(&mut self.client, &self.subscription_link).await.unwrap();
        
            for tx in message_list.iter() {
                let header_opt = match tx.parse_header(){
                    Ok(val) => Some(val),
                    Err(e) => {
                        println!("Parsing Error Header: {}", e);
                        None
                    }
                };
                match header_opt {
                    None => println!("Invalid message"),
                    Some(header) => {

                        if header.check_content_type(message::signed_packet::TYPE) {
                            match self.subscriber.unwrap_signed_packet(header.clone()) {
                                Ok((unwrapped_public, unwrapped_masked)) => {
                                    response.push((unwrapped_public.to_string(),unwrapped_masked.to_string()));
                                }
                                Err(e) => println!("Signed Packet Error: {}", e),
                            }
                            continue;
                        }
                    }
                }
            }

        }else {
            println!("Channel not connected");
        }

        Ok(response)
    }


    pub async fn read_tagged(&mut self) -> Result<Vec<(String,String)>,&str>{
        
        println!("Receiving tagged messages");

        let mut response:Vec<(String,String)> = Vec::new();
    
        if self.is_connected {

            let message_list = async_messaging::recv_messages(&mut self.client, &self.subscription_link).await.unwrap();

            for tx in message_list.iter() {
                let header_opt = match tx.parse_header(){
                    Ok(val) => Some(val),
                    Err(e) => {
                        println!("Parsing Error Header: {}", e);
                        None
                    }
                };
                match header_opt {
                    None => println!("Invalid message"),
                    Some(header) => {

                        if header.check_content_type(message::tagged_packet::TYPE) {
                            match self.subscriber.unwrap_tagged_packet(header.clone()) {
                                Ok((unwrapped_public, unwrapped_masked)) => {
                                    response.push((unwrapped_public.to_string(),unwrapped_masked.to_string()));
                                }
                                Err(e) => println!("Tagged Packet Error: {}", e),
                            }
                            continue;
                        }
                    }
                }
            }

        }else {
            println!("Channel not connected");
        }

        Ok(response)
    }

    pub async fn update_keyload(&mut self, keyload_tag:String) -> Fallible<()>{

        let keyload_link = Address::from_str(&self.channel_address, &keyload_tag).unwrap();

        if self.is_connected {

            let message_list = async_messaging::recv_messages(&mut self.client, &keyload_link).await.unwrap();

            for tx in message_list.iter() {
                let header_opt = match tx.parse_header(){
                    Ok(val) => Some(val),
                    Err(e) => {
                        println!("Parsing Error Header: {}", e);
                        None
                    }
                };
                match header_opt {
                    None => println!("Invalid message"),
                    Some(header) => {

                        if header.check_content_type(message::keyload::TYPE) {
                            match self.subscriber.unwrap_keyload(header.clone()) {
                                Ok(_) => {
                                    println!("Updated keyload!");
                                    break;
                                }
                                Err(e) => println!("Tagged Packet Error: {}", e),
                            }
                            continue;
                        }
                    }
                }
            }

        }else {
            println!("Channel not connected");
        }

        Ok(())
    }
}