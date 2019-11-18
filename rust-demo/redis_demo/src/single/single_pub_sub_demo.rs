use std::thread;
use redis::{Commands, ControlFlow, PubSubCommands};
use std::time::Duration;

pub fn single_pub_sub_main() {
    let a = scoped_pubsub();
//    test();
//    println!("{:?}", a);
}


pub fn single_pub_sub() -> redis::RedisResult<()> {
    println!("sub pub");


    thread::spawn(|| {
        loop {
            pub_msg();
        }
    });


    let client = redis::Client::open("redis://:123456@192.168.101.13:16379")?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();
    let msg = pubsub.get_message()?;
    let payload: String = msg.get_payload()?;
    println!("channel '{}': {}", msg.get_channel_name(), payload);
    loop {}
    Ok(())
}

pub fn pub_msg() -> redis::RedisResult<()> {
    println!("sub");
    let client = redis::Client::open("redis://:123456@192.168.101.13:16379")?;
    let mut con = client.get_connection()?;
//    let mut pubsub = con.as_pubsub();

//    pubsub.subscribe("channel_1")?;

//    con.publish("chhh",23);

    Ok(())
}


pub fn sub_msg() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://:123456@192.168.101.13:16379")?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();
    println!("pub");
    pubsub.subscribe("channel_2")?;
    let msg = pubsub.get_message()?;
    let payload: String = msg.get_payload()?;
    println!("channel '{}': {}", msg.get_channel_name(), payload);

    Ok(())
}

fn scoped_pubsub() -> redis::RedisResult<()> {
    let client = redis::Client::open("redis://:123456@192.168.101.13:16379").unwrap();
    let mut con = client.get_connection().unwrap();


    let mut pubsub_con = client.get_connection().unwrap();

    let thread = thread::spawn(move || {
        let mut count = 0;
        pubsub_con
            .subscribe(&["foo", "bar"], |msg| {
                count += 1;
                match msg.get_channel_name() {
                    "foo" => {
                        let value: i32 = msg.get_payload().unwrap();
                        println!("ggggggggggggggg{:?},{:?}", msg.get_channel_name(), value);
                        assert_eq!(msg.get_channel(), Ok("foo".to_string()));
                        assert_eq!(msg.get_payload(), Ok(42));
                        ControlFlow::Continue
                    }
                    "bar" => {
                        let value: i32 = msg.get_payload().unwrap();
                        println!("ggggggggggggggg{:?},{:?}", msg.get_channel_name(), value);
                        assert_eq!(msg.get_channel(), Ok("bar".to_string()));
                        assert_eq!(msg.get_payload(), Ok(23));
                        ControlFlow::Continue
                    }
                    _ => ControlFlow::Break(()),
                }
            })
            .unwrap();

        pubsub_con
    });

    // Can't use a barrier in this case since there's no opportunity to run code
    // between channel subscription and blocking for messages.
    thread::sleep(Duration::from_millis(100));

    loop {
        redis::cmd("PUBLISH").arg("foo").arg(42).execute(&mut con);

        thread::sleep(Duration::from_millis(1000));

        con.publish("bar", 23)?;
//        assert_eq!(con.publish("bar", 23), Ok(1));
    }


    // Wait for thread
    let mut pubsub_con = thread.join().ok().expect("pubsub thread terminates ok");

    // Connection should be usable again for non-pubsub commands
    let _: redis::Value = pubsub_con.set("foo", "bar").unwrap();
    let foo: String = pubsub_con.get("foo").unwrap();
    assert_eq!(&foo[..], "bar");
    Ok(())
}

