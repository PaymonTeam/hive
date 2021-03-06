extern crate futures;

use std::io::Error;

use crate::utils::config::{Configuration, ConfigurationSettings};
use std::sync::{Arc, Weak, Mutex};
use mio::tcp::TcpStream;
use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use serde_pm::{SerializedBuffer, to_buffer, Identifiable};
use std::net::{SocketAddr, IpAddr};
//use network::replicator::*;
use crate::network::replicator_new::*;
use std::fmt::{Debug, Formatter};
use std::fmt;
use futures::Stream;

pub struct Neighbor {
    pub addr: SocketAddr,
//    pub replicator_source: Option<Weak<Mutex<ReplicatorSource>>>,
//    pub replicator_sink: Option<Weak<Mutex<ReplicatorSource>>>,
    pub sink: Option<ReplicatorSink>,
    pub source: Option<ReplicatorSource>,
    pub connecting: bool,
}

impl Debug for Neighbor {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Host: {:?}", self.addr)
    }
}

impl Neighbor {
    pub fn from_connection(conn: &Arc<Mutex<TcpStream>>) -> Self {
        let conn = conn.clone();
        let addr = conn.lock().unwrap().peer_addr().expect("invalid address");

        Neighbor {
            addr,
            source: None,
            sink: None,
            connecting: false,
        }
    }

    pub fn from_address(addr: SocketAddr) -> Self {
        Neighbor {
            addr,
            source: None,
            sink: None,
            connecting: false,
        }
    }

    pub fn from_replicator_source(replicator: /*Weak<Mutex<*/ReplicatorSource/*>>*/, addr: SocketAddr) -> Self {
        Neighbor {
            addr,
            source: Some(replicator),
            sink: None,
            connecting: false,
        }
    }

    pub fn from_replicator_sink(replicator: /*Weak<Mutex<*/ReplicatorSink/*>>*/, addr: SocketAddr)
        -> Self {
        Neighbor {
            addr,
            source: None,
            sink: Some(replicator),
            connecting: false,
        }
    }

    pub fn get_sockaddr(&self) -> SocketAddr {
        self.addr.clone()
    }

    pub fn send_packets<S, T>(&mut self, _stream: S) where S: Stream<Item=T, Error=()>, T: Serialize + Identifiable {
//        stream.map(|p| {
//
//        });
    }

    pub fn prepare_data<T>(packet: Box<T>) -> Vec<u8> where T: Serialize + Identifiable {
        use serde_pm::{SerializedBuffer, to_boxed_buffer};
        use self::futures::{Sink, Future};
        let message_id = 0;
//        let sb = to_boxed_buffer(&packet).unwrap();

        let sb = to_boxed_buffer(&packet).unwrap();

        let message_length = sb.capacity(); //calculate_object_size(&packet);
        let size = match message_length % 4 == 0 {
            true => 8 + 4 + message_length as usize,
            false => {
                let additional = 4 - (message_length % 4) as usize;
                8 + 4 + message_length as usize + additional
            }
        };

        let mut buff = SerializedBuffer::new_with_size(size);
        buff.set_position(0);
        buff.write_i64(message_id);
        buff.write_i32(message_length as i32);
        buff.write_bytes(&sb);
//            packet.serialize_to_stream(&mut buff);

        buff.rewind();

        let mut buffer_len = 0;
        let mut packet_length = (buff.limit() / 4) as i32;

        if packet_length < 0x7f {
            buffer_len += 1;
        } else {
            buffer_len += 4;
        }

        let mut buffer = SerializedBuffer::new_with_size(buffer_len);
        if packet_length < 0x7f {
            buffer.write_byte(packet_length as u8);
        } else {
            packet_length = (packet_length << 8) + 0x7f;
            buffer.write_i32(packet_length);
        }

        buffer.rewind();
        buff.rewind();

        let mut vec = buffer.as_ref().to_vec();
        vec.extend_from_slice(buff.as_ref());
        vec
    }

    pub fn send_packet<T>(&mut self, packet: Box<T>) where T: Serialize + Identifiable {
        if let Some(ref mut o) = self.sink {
            let data = Self::prepare_data(packet);
            // FIXME: (maybe)
            o.tx.send(data);
        }
//            o.tx.clone()
//                .send((&buffer).to_vec()).wait().unwrap()
//                .send((&buff).to_vec()).wait().unwrap()
//                .flush().wait().unwrap();

//            if let Some(arc) = o.upgrade() {
//                if let Ok(mut replicator) = arc.lock() {
//                    o.send_packet(packet, 0);
//                }
//            }
    }
}