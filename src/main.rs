pub mod string;
pub mod vecu8;

#[derive(rkyv::Archive, rkyv::Serialize)]
struct StringyThing {
    s: string::String<512>,
    vecu8: vecu8::VecU8<heapless::consts::U65>,
}

fn main() {
    let test_str = "1";
    let mut test_vec = vecu8::VecU8::new();
    test_vec.push(4).unwrap();
    test_vec.push(93).unwrap();
    test_vec.push(93).unwrap();
    let vecu8 = test_vec.clone();
    let my_s = StringyThing {
        s: string::String::from_str(test_str),
        vecu8,
    };

    let mut writer = rkyv::ser::serializers::BufferSerializer::new(rkyv::Aligned([0u8; 256]));

    // It works!
    println!("Archiving...");
    let pos = {
        use rkyv::ser::Serializer;
        writer
            .serialize_value(&my_s)
            .expect("failed to archive test")
    };
    println!("Finalizing...");
    let buf = writer.into_inner();
    println!("Deserializing...");
    let archived = unsafe { rkyv::archived_value::<StringyThing>(buf.as_ref(), pos) };
    // Let's make sure our data got written correctly
    assert_eq!(archived.s.as_str(), test_str);

    println!("Original string: {}", test_str);
    println!("Archived string: {}", archived.s.as_str());
    println!("Original vec: {:?}", test_vec.as_slice());
    println!("Archived vec: {:?}", archived.vecu8.as_slice());
    println!("Pos: {}", pos);
    print!("Memory buffer: ");
    for &x in buf.as_ref().iter() {
        print!(" {:02x}", x);
    }
    println!();
}
