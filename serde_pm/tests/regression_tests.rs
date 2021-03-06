extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_pm as serde_pm_other_name;    // Tests `serde_pm_derive`
#[macro_use]
extern crate serde_pm_derive;
#[macro_use]
extern crate log;
extern crate env_logger;

use serde::ser::{self};
use serde::de::{self, Deserializer, DeserializeSeed, Error as DeError};
use serde_pm_other_name::{Boxed, PMSized, from_stream, to_buffer, to_buffer_with_padding, to_boxed_buffer};
use serde_pm_other_name::serializable::*;
use serde_pm_other_name::identifiable::Identifiable;

/// Doesn't work perfectly now
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, PMIdentifiable, PMSized)]
#[pm_identifiable(id = "0xbbbbbbbb")]
enum Algebraic {
//    #[pm_identifiable(id = "0xbbbbbbbb")]
    A,
//    #[pm_identifiable(id = "0xbbbbbbbb")]
    B(u32),
//    #[pm_identifiable(id = "0xcccccccc")]
    C(i8, String)
}

impl Default for Algebraic {
    fn default() -> Self {
        Algebraic::A
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, PMIdentifiable)]
#[pm_identifiable(id = "0xaeaeaeae")]
struct PackWithBorrowedData<'a> {
    a_b: &'a [u8],
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, PMIdentifiable, PMSized, Default)]
#[pm_identifiable(id = "0xacacacac")]
struct Pack {
    v: i32,
    s: String,
    b: u8,
    vec: Vec<bool>,
    b_a: [u8; 5],
    variant: Algebraic,
    variant_box: Boxed<Algebraic>,
//    variant: Boxed<Algebraic>,
}

impl Pack {
    pub fn new() -> Self {
        Pack::default()
    }
}

fn init_log() {
    use env_logger::LogBuilder;
    use log::{LogRecord, LogLevelFilter};
    use std::env;

    let format = |record: &LogRecord| {
        format!("[{}]: {}", record.level(), record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format)
        .filter(None, LogLevelFilter::Info)
        .filter(Some("futures"), LogLevelFilter::Error)
        .filter(Some("tokio"), LogLevelFilter::Error)
        .filter(Some("tokio-io"), LogLevelFilter::Error)
        .filter(Some("hyper"), LogLevelFilter::Error)
        .filter(Some("iron"), LogLevelFilter::Error);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init().unwrap();
}

//#[test]
//fn serde_serialized_buffer_with_borrowed_data() {
//    init_log();
//
//    let mut pack = PackWithBorrowedData {
//        a_b: &[1, 2],
//    };
//
//    let mut b0 = to_buffer(&pack).expect("failed to serialize data");
//    debug!("b0={:?}", b0.as_ref());
//    let mut b1 = SerializedBuffer::from_slice(&[2, 0, 0, 0, 1, 2]);
//
//    assert_eq!(b0.as_ref(), b1.as_ref());
//    let pack2 = from_owned_stream::<PackWithBorrowedData>(&mut b0).expect("failed to deserealize data");
//    assert_eq!(pack, pack2);
//    debug!("{:?}", pack2);
//}

#[test]
fn serde_serialized_buffer() {
    init_log();

    let mut pack = Pack::new();
    pack.v = 3;
    pack.s = "hello".into();
    pack.b = 42;
    pack.vec.append(&mut vec![true, false]);
    pack.b_a = [1u8, 2, 3, 4, 5];
    pack.variant = Algebraic::C(4, "kek".into());
    pack.variant_box = Boxed::new(Algebraic::B(1));

    let mut b0 = to_buffer(&pack).expect("failed to serialize data");
    debug!("b0={:?}", b0.as_ref());
    let mut b1 = SerializedBuffer::from_slice(&[3, 0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 42, 2, 0, 0, 0, 179, 100, 74, 110, 195, 41, 93, 63, 1, 2, 3, 4, 5, 2, 4, 3, 107, 101, 107, 187, 187, 187, 187, 1, 1, 0, 0, 0]);

    assert_eq!(b0.as_ref(), b1.as_ref());
    let pack2 = from_stream::<Pack>(&mut b0).expect("failed to deserealize data");
    assert_eq!(pack, pack2);
    debug!("{:?}", pack2);
}

#[test]
fn serde_serialized_buffer_with_type_id() {
    init_log();
    assert_eq!(Pack::all_type_ids().len(), 1);

    let mut pack = Pack::new();
    pack.v = 3;
    pack.s = "hello".into();
    pack.b = 42;
    pack.vec.append(&mut vec![true, false]);
    pack.b_a = [1u8, 2, 3, 4, 5];
    pack.variant = Algebraic::C(4, "kek".into());
    pack.variant_box = Boxed::new(Algebraic::B(1));

    let mut b0 = to_boxed_buffer(&pack).expect("failed to serialize data");
    debug!("b0={:?}", b0.as_ref());
    let mut b1 = SerializedBuffer::from_slice(&[172, 172, 172, 172, 3, 0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 42, 2, 0, 0, 0, 179, 100, 74, 110, 195, 41, 93, 63, 1, 2, 3, 4, 5, 2, 4, 3, 107, 101, 107, 187, 187, 187, 187, 1, 1, 0, 0, 0]);

    assert_eq!(b0.as_ref(), b1.as_ref());

    let type_id = b0.read_u32().expect("failed to read type_id");
    let pack_type_id = Pack::all_type_ids()[0];
    if type_id == pack_type_id {
        let pack2 = from_stream::<Pack>(&mut b0).expect("failed to deserealize data");
        assert_eq!(pack, pack2);
        debug!("{:?}", pack2);
    } else {
        panic!("unknown type_id");
    }
}

#[test]
fn serde_serialized_buffer_padded() {
    let enum_ids = [ "", ];
    let mut pack = Pack::new();
    pack.v = 3;
    pack.s = "hello".into();
    pack.b = 42;
    pack.vec.append(&mut vec![true, false]);
    pack.b_a = [1u8, 2, 3, 4, 5];
    pack.variant_box = Boxed::new(Algebraic::B(1));

    let mut b0 = to_buffer_with_padding(&pack).expect("failed to serialize data");
    debug!("b0={:?}", b0.as_ref());
    let mut b1 = SerializedBuffer::from_slice(&[3, 0, 0, 0, 5, 104, 101, 108, 108, 111, 0, 0, 42, 2, 0, 0, 0, 179, 100, 74, 110, 195, 41, 93, 63, 1, 2, 3, 4, 5, 0, 187, 187, 187, 187, 1, 1, 0, 0, 0]);

    assert_eq!(b0.as_ref(), b1.as_ref());

    let pack2 = from_stream::<Pack>(&mut b0).expect("failed to deserealize data");
    assert_eq!(pack, pack2);
    debug!("{:?}", pack2);
}

#[test]
fn floating_point() {
    #[derive(Debug, Serialize, Deserialize, PMSized, Default)]
    struct FD(f32, f64);
    let fd = FD(std::f32::consts::PI, std::f64::consts::SQRT_2);

    let b0 = to_buffer(&fd).expect("failed to serialize data");
    debug!("b0={:?}", b0.as_ref());
    let mut b1 = SerializedBuffer::from_slice(&[219, 15, 73, 64, 205, 59, 127, 102, 158, 160, 246, 63]);

    assert_eq!(b0.as_ref(), b1.as_ref());
    let pack2 = from_stream::<FD>(&mut b1).expect("failed to deserealize data");
    debug!("{:?}", pack2);
}