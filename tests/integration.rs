use std::process::*;
use ws::{ connect, Message, Sender };
use std::time::Duration;
use std::thread;

fn send(c: &Sender, req: &str)
{
    println!(">>> {}", req);
    c.send(req).unwrap();
}

#[test]
fn it_works()
{
    println!("Starting the server...");

    let mut server = Command::new("cargo")
        .arg("run")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start server.");
    

    thread::spawn(move ||
    {
        server.wait().unwrap();
        panic!("Server exited unexpectedly.");
    });

    println!("Server started.");

    thread::sleep(Duration::from_millis(400));
    
    println!("Connecting to the server...");


    
    connect("ws://127.0.0.1:1234", |c|
    {
        println!("Connected to the server.");

        thread::spawn(move ||
        {
            loop
            {
                send(&c, "hello");

                thread::sleep(Duration::from_millis(500));

                send(&c, "{\"request\": \"Ping\", \"data\": {},\
                       \"reply_to\":\
                \"84a0c091-5ba8-47db-9d2f-9b4aad197366\"}");

                thread::sleep(Duration::from_millis(500));
                
                send(&c, "{\"request\": \"GetServerInfo\",\
                        \"data\": { \"field\": \"Uptime\" },\
                       \"reply_to\":\
                \"84a0c091-5ba8-47db-9d2f-9b4aad197367\"}");

                thread::sleep(Duration::from_millis(500));

                send(&c, "{\"request\": \"GetServerInfo\",\
                        \"data\": { \"field\": \"ConnectedClients\" },\
                       \"reply_to\":\
                \"84a0c091-5ba8-47db-9d2f-9b4aad197367\"}");

                thread::sleep(Duration::from_millis(500));

                send(&c, "{\"request\": \"GetServerInfo\",\
                        \"data\": { \"field\": \"abc\" },\
                       \"reply_to\":\
                \"84a0c091-5ba8-47db-9d2f-9b4aad197367\"}");

                thread::sleep(Duration::from_millis(500));

                send(&c, "{\"request\": \"qwer\",\
                        \"data\": { \"field\": \"abc\" },\
                       \"reply_to\":\
                \"84a0c091-5ba8-47db-9d2f-9b4aad197367\"}");

                thread::sleep(Duration::from_millis(500));

                send(&c, "{\"request\": \"Ping\",\
                        \"data\": {},\
                       \"reply_to\":\
                \"84a0x091-5ba8-47db-9d2f-9b4aad197367\"}");

                thread::sleep(Duration::from_millis(500));
            }
        });

        move |msg: Message|
        {
            println!("<<< {}", msg);
            Ok(())
        }
    }).unwrap();

    loop {}
}
