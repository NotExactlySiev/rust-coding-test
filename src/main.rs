use std::time::{ Instant };
use std::sync::{ Mutex };
use uuid::Uuid;
use json::{ JsonValue, parse, object };
use ws::{ listen, Sender, Handler, Message, Handshake, CloseCode };

static mut START_TIME: Option<Instant> = None;
static mut CLIENTS: Option<Mutex<u32>> = None;

fn uptime() -> u64
{
    unsafe { START_TIME.unwrap().elapsed().as_secs() }
}

fn clients() -> u32
{
    unsafe
    {
        let count: u32 = *(CLIENTS.as_ref().unwrap().lock().unwrap());
        //CLIENTS.unlock();
        count
    }
}


fn handle_request(req: Request, data: &JsonValue)
    -> Result<(&'static str,JsonValue), ServiceError>
{
    match req
    {
        Request::Ping => Ok(("Pong",object!{})),

        Request::GetServerInfo =>
            if data.has_key("field")
            {
                match data["field"].as_str()
                {
                    Some("Uptime") =>
                        // use const here
                        Ok(("GetServerInfo",
                            object!{ value: uptime() })),

                    Some("ConnectedClients") =>
                        Ok(("GetServerInfo",
                            object!{ value: clients() })),

                    _ => Err(ServiceError::InvalidFieldName)
                }
            }
            else
            {
                Err(ServiceError::MissingParameter)
            }
    }
}

fn parse_and_handle(raw: String)
    -> (Result<(&'static str,JsonValue), ServiceError>, Uuid)
{
    if let Ok(parsed) = parse(&raw)
    {
        if let Some(uuid_raw) = parsed["reply_to"].as_str()
        {
            if let Ok(uuid) = Uuid::parse_str(uuid_raw)
            {
                if let Some(req_raw) = parsed["request"].as_str()
                {
                    match Request::from_str(req_raw)
                    {
                        Ok(req) => 
                        {
                            let data = &parsed["data"];
                            match handle_request(req,data)
                            {
                                Ok(response) => (Ok(response),uuid),
                                Err(e) => (Err(e),uuid)
                            }
                        },

                        Err(e) => (Err(e),uuid)
                    }
                }
                else
                {
                    (Err(ServiceError::MissingRequest),uuid)
                }
            }
            else
            {
                (Err(ServiceError::InvalidUuid),Uuid::nil())
            }
        }
        else
        {
            (Err(ServiceError::MissingUuid),Uuid::nil())
        }
    }
    else
    {
        (Err(ServiceError::InvalidJson),Uuid::nil())
    }
}

#[derive(Debug)]
enum ServiceError
{
    InvalidJson,        // Message cannot be parsed as JSON
    InvalidRequest,     // Desired request does not exist
    InvalidUuid,
    InvalidFieldName,   // Such server info field does not exist 
    MissingUuid,
    MissingRequest,
    MissingParameter,
}

#[derive(Debug)]
enum Request
{
    Ping,
    GetServerInfo,
}

impl Request
{
    pub fn from_str(text: &str) -> Result<Request, ServiceError>
    {
        match text
        {
            "Ping" => Ok(Request::Ping),
            "GetServerInfo" => Ok(Request::GetServerInfo),
            _ => Err(ServiceError::InvalidRequest),
        }
    }
}

pub struct TestHandler(Sender);

impl Handler for TestHandler
{
    fn on_open(&mut self, _: Handshake) -> ws::Result<()>
    {
        println!("A client has connected.");
        unsafe
        {
            *CLIENTS.as_ref().unwrap().lock().unwrap() += 1;
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str)
    {
        println!("Client connection closed ({:?}): {}", code, reason);
        unsafe
        {
            *CLIENTS.as_ref().unwrap().lock().unwrap() -= 1;
        }
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()>
    {
        let msg = format!("{}", msg);

        let result = parse_and_handle(msg);

        let (response, data): (&'static str, JsonValue) = match result.0
        {
            Ok(resp) => resp,

            Err(e) =>
                ("ServiceError",
                 object!{ code: format!("{:?}", e) })
        };

        let response = object! {
            response: JsonValue::from(response),
            data: data,
            reply_to: result.1.hyphenated().to_string(),
        };

        let response = Message::text(response.dump());

        self.0.send(response)
    }
}

pub fn main()
{
    unsafe 
    { 
        START_TIME = Some(Instant::now());
        CLIENTS = Some(Mutex::new(0));
    };

    listen("127.0.0.1:1234", |out| TestHandler(out))
    .expect("Couldn't start server.");
}
