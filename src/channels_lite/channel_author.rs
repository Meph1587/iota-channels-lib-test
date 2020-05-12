#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use iota::Client;
use iota_streams::app_channels::{
    api::tangle::{ Address, Author, DefaultTW, Message}
    , message
};
use iota_streams::app::transport::Transport;
use iota_streams::app::transport::tangle::client::SendTrytesOptions;
use iota_streams::protobuf3::types::Trytes;
use iota_streams::core::tbits::Tbits;
use std::string::ToString;
use std::str::FromStr;
use failure::Fallible;


use crate::messaging::async_messaging;

pub struct Channel{
    author: Author,
    client: iota::Client<'static>,
    send_options: SendTrytesOptions,
    channel_address:String,
    announcement_link: Address,
    keyload_link: Address,
}

impl Channel {

    pub fn new(seed: &str, node_ulr: &'static str) -> Channel{

        let author = Author::new(seed, 3, true);

        let channel_address = author.channel_address().to_string();
        let mut send_opt = SendTrytesOptions::default();

        send_opt.min_weight_magnitude = 9;
        send_opt.local_pow = false;

        Self {
            author:author,
            client: iota::Client::new(node_ulr),
            send_options: send_opt,
            channel_address:channel_address,
            announcement_link: Address::default(),
            keyload_link: Address::default(),
        }
    }

    pub async fn open(&mut self)-> Result<(String, String), &str>{
       
        let announcement_message = self.author.announce().unwrap();
        async_messaging::send_message(&mut self.client, &announcement_message).await.unwrap();
        println!("Announced a new channel");

        let announcement_address: String = announcement_message.link.appinst.to_string();
        let announcement_tag: String = announcement_message.link.msgid.to_string();

        self.announcement_link = Address::from_str(&announcement_address, &announcement_tag).unwrap();

        Ok((self.channel_address.clone(), announcement_tag))
    }

    pub async fn add_subscriber(&mut self, subscribe_tag: String) -> Result<String,&str>{

        let subscribe_link = Address::from_str(&self.channel_address, &subscribe_tag).unwrap();

        let message_list = async_messaging::recv_messages(&mut self.client, &subscribe_link).await.unwrap();
    
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

                    if header.check_content_type(message::subscribe::TYPE) {
                        match self.author.unwrap_subscribe(header.clone()) {
                            Ok(_) => {
                                println!("Subscribtion successfull"); 
                                break;
                            },
                            Err(e) => println!("Subscribe Packet Error: {}", e),
                        }
                        continue;
                    }
                }
            }
        }

        self.keyload_link = {
            let msg = self.author.share_keyload_for_everyone(&self.announcement_link).unwrap();
            async_messaging::send_message(&mut self.client, &msg).await.unwrap();
            msg.link
        };

        Ok(self.keyload_link.msgid.to_string())

    }

    pub async fn write_signed(&mut self, public_payload: &str, private_payload: &str)-> Result<String, &str>{

        let public_payload = Trytes(Tbits::from_str(&public_payload).unwrap());
        let private_payload = Trytes(Tbits::from_str(&private_payload).unwrap());

        let signed_packet_link = {
            let msg = self.author.sign_packet(&self.announcement_link, &public_payload, &private_payload).unwrap();
            async_messaging::send_message(&mut self.client, &msg).await.unwrap();
            println!("Sent signed packet");
            msg.link.clone()
        };

        Ok(signed_packet_link.msgid.to_string())

    }

    pub async fn write_tagged(&mut self, public_payload: &str, private_payload: &str)-> Result<String, &str>{

        let public_payload = Trytes(Tbits::from_str(&public_payload).unwrap());
        let private_payload = Trytes(Tbits::from_str(&private_payload).unwrap());

        let tagged_packet_link = {
            let msg = self.author.tag_packet(&self.keyload_link, &public_payload, &private_payload).unwrap();
            async_messaging::send_message(&mut self.client, &msg).await.unwrap();
            println!("Sent tagged packet");
            msg.link.clone()
        };
        Ok(tagged_packet_link.msgid.to_string())

    }

    pub async fn remove_subscriber(&mut self, unsubscribe_tag: String) -> Fallible<()>{

        let unsubscribe_link = Address::from_str(&self.channel_address, &unsubscribe_tag).unwrap();

        let message_list = async_messaging::recv_messages(&mut self.client, &unsubscribe_link).await.unwrap();
    
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

                    if header.check_content_type(message::unsubscribe::TYPE) {
                        match self.author.unwrap_unsubscribe(header.clone()) {
                            Ok(_) => {
                                println!("Unsubscribtion successfull");
                                break;
                            },
                            Err(e) => println!("Unsubscribe Packet Error: {}", e),
                        }
                        continue;
                    }
                }
            }
        }
        Ok(())
    }

}