extern crate byteorder;
extern crate mio;
extern crate rand;
extern crate slab;
extern crate ethcore_bigint as bigint;
extern crate memorydb;
extern crate patricia_trie;
extern crate env_logger;
extern crate rustc_serialize;
extern crate iron;
extern crate ntrumls;
#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;

#[macro_use] pub mod utils;
pub mod network;
pub mod model;
pub mod storage;

use std::{
    sync::{mpsc::channel, Condvar, Arc, Weak, Mutex, atomic::{AtomicBool, Ordering}},
    io::{self, Read},
    env,
    collections::VecDeque,
    thread,
    thread::Builder,
    time::Duration,
};

use mio::Poll;
use mio::net::TcpListener;

use network::node::*;
use network::replicator_pool::ReplicatorPool;
use model::config::{PORT, Configuration, ConfigurationSettings};
use model::config;
use network::paymoncoin::PaymonCoin;
use env_logger::LogBuilder;
use log::{LogRecord, LogLevelFilter};
use storage::Hive;
use network::api::API;

fn main() {
    use ntrumls::*;
    use rand::Rng;

    let format = |record: &LogRecord| {
        format!("[{} {:?}]: {}", record.level(), thread::current().id(), record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();
//    let mut sk_data = [0u8; 32 * 8];
//    rand::thread_rng().fill_bytes(&mut sk_data);
////    let (addr, sk, pk) = Hive::generate_address(&sk_data, 0);
//    use std::mem;
//    let fg_16 : [u16; 128] = unsafe { mem::transmute(sk_data) };
////    for n in fg_16.iter() {
////        print!("{}, ", n);
////    }
////    println!();
//    let mut mls = NTRUMLS::with_param_set(PQParamSetID::Security269Bit);
//
//    let (sk, pk) = mls.generate_keypair().unwrap();
//
//    let msg = "TEST MESSAGE";
////    let msg = [1u8; 16];
//    let sign = mls.sign(msg.as_bytes(), &sk, &pk).expect("fail");
//
//    println!("{:?}", sk);
//    println!("{:?}", pk);
//    println!("{:?}", sign);
//    println!("{}", mls.verify(msg.as_bytes(), &sign, &pk));
}

#[test]
fn test_threads() {
    let format = |record: &LogRecord| {
        format!("[{} {:?}]: {}", record.level(), thread::current().id(), record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(None, LogLevelFilter::Info);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();

    let mut jhs = VecDeque::new();

    let ports = [0, 70, 10002].iter();
    for port in ports {
        let port = *port;
        let mut neighbors = String::new();
        if port != 0 {
            let ports2 = [44832, 70, 10002].iter();
            let v: Vec<String> = ports2.filter(|p| **p != port).map(|p| format!("127.0.0.1:{}",
                                                                                p)).collect();
            neighbors = v.join(" ");
        }
        println!("{}", neighbors);

        let jh = Builder::new().name(format!("pmnc {}", port)).spawn(move || {
            let mut config = Configuration::new();
            if port != 0 {
                config.set_string(ConfigurationSettings::Neighbors, &neighbors);
                config.set_int(ConfigurationSettings::Port, port);
            }

            let pmnc = Arc::new(Mutex::new(PaymonCoin::new(config)));

            let node_arc = pmnc.lock().unwrap().run();

            let pmnc_clone = pmnc.clone();
//            let mut api_running = Arc::new(AtomicBool::from(true));
//            let api_running_clone = api_running.clone();
            let api_running = Arc::new((Mutex::new(true), Condvar::new()));
            let api_running_clone = api_running.clone();

            let api_jh = thread::spawn(move || {
                let mut api = API::new(pmnc_clone, (port + 10) as u16, api_running_clone);
                api.run();
                drop(api);
            });

            thread::sleep(Duration::from_secs(10000));

            {
    //            api_running.store(false, Ordering::SeqCst);
                let &(ref lock, ref cvar) = &*api_running;
                let mut is_running = lock.lock().unwrap();
                *is_running = false;
                cvar.notify_one();
            }

            api_jh.join();
            node_arc.lock().unwrap().shutdown();

//            thread::sleep(Duration::from_secs(9));

//            let mut hive = Arc::new(Mutex::new(Hive::new()));
//
//            // used for shutdown replicator pool
//            let (tx, rx) = channel::<()>();
//            let mut node = Arc::new(Mutex::new(Node::new(Arc::downgrade(&hive.clone()), &config, tx)));
//
//
//            let node_copy = node.clone();
//            let replicator_jh = thread::spawn(move || {
//                let mut replicator_pool = ReplicatorPool::new(&config, Arc::downgrade(&node_copy), rx);
//                replicator_pool.run();
//            });
//
//            {
//                let mut guard = node.lock().unwrap();
//                guard.init(replicator_jh);
//                guard.run().expect("Failed to run server");
//            }
//
//            use std::thread;
//            use std::time::Duration;
//            thread::sleep(Duration::from_secs(9));
//            node.lock().unwrap().shutdown();
        }).unwrap();

        jhs.push_back(jh);
    }

    while let Some(jh) = jhs.pop_front() {
        jh.join();
    }
}

#[test]
fn hive_test() {
    use model::{Transaction, TransactionObject};
    use model::transaction::{Hash, ADDRESS_NULL, HASH_SIZE};
    use storage::hive::{CFType};

    use self::rustc_serialize::hex::{ToHex, FromHex};
    use rand::Rng;

    let mut hive = Hive::new();
    hive.init();

    let h0 = Hash([1u8; HASH_SIZE]);
    let h1 = Hash([2u8; HASH_SIZE]);
    let h2 = Hash([3u8; HASH_SIZE]);

    assert!(hive.put_approvee(h1, h0));
    assert!(hive.put_approvee(h2, h0));

    let hashes = hive.storage_load_approvee(&h0).expect("failed to load hashes");
    println!("{:?}", hashes);

    let mut t0 = TransactionObject::new_random();
    hive.storage_put(CFType::Transaction, &t0.hash, &t0);
    let t1 = hive.storage_get_transaction(&t0.hash).expect("failed to load transaction from db");
    assert_eq!(t0, t1.object);

    let addr0 = ADDRESS_NULL;

    let random_sk = true;

    let mut data =
        "2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A\
        3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87\
        EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A\
        0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A774\
        84468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B021\
        4EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD".from_hex().expect("invalid sk");
//    let mut sk_data = [0u8; 32 * 8];
//    if random_sk {
//        rand::thread_rng().fill_bytes(&mut sk_data);
//    } else {
//        sk_data.copy_from_slice(&data[..(32 * 8)]);
//    }

//    let (addr, sk, pk) = Hive::generate_address(&sk_data, 0);
//    hive.storage_put(CFType::Address, &addr, &10000u32);
//    let balance = hive.storage_get_address(&addr).expect("storage get address error");

//    println!("sk={}", sk_data.to_hex().to_uppercase());
//    println!("address={:?}", addr);
//    println!("address={:?} balance={}", addr, balance);
}

#[test]
fn hive_transaction_test() {
    use model::{Transaction, TransactionObject};
    use model::transaction::ADDRESS_NULL;
    use storage::hive::{CFType};

    use self::rustc_serialize::hex::{ToHex, FromHex};
    use rand::Rng;

    let mut hive = Hive::new();
    hive.init();

    let mut t0 = TransactionObject::new_random();
    hive.storage_put(CFType::Transaction, &t0.hash, &t0);
    let t1 = hive.storage_get_transaction(&t0.hash).expect("failed to load transaction from db");
    assert_eq!(t0, t1.object);

    let addr0 = ADDRESS_NULL;

    let random_sk = true;

    let mut data =
        "2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A\
        3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87\
        EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A\
        0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A774\
        84468E87EC59ABDBD2FB5A00B0214EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD2FB5A00B021\
        4EDBDA0A0A004F8A3DBBCC76744523A8A77484468E87EC59ABDBD".from_hex().expect("invalid sk");
//    let mut sk_data = [0u8; 32 * 8];
////    if random_sk {
////        rand::thread_rng().fill_bytes(&mut sk_data);
////    } else {
////        sk_data.copy_from_slice(&data[..(32 * 8)]);
////    }

//    let addr = Hive::generate_address(&sk_data, 0);
//    hive.storage_put(CFType::Address, &addr, &10000u32);
//    let balance = hive.storage_get_address(&addr).expect("storage get address error");

//    println!("sk={}", sk_data.to_hex().to_uppercase());
//    println!("address={:?}", addr);
//    println!("address={:?} balance={}", addr, balance);
}