use anyhow::{anyhow, bail, ensure, Result, Ok};
use async_openai::{
    config::OpenAIConfig,
    types::{self as openai_types, FunctionCall},
    Client,
};
use diesel::expression::is_aggregate::No;
use indoc::formatdoc;
use lazy_static::lazy_static;
use serde_json::{json, Value};
use tokio::sync::mpsc::Sender;

use super::types::User;

type Response = openai_types::ChatCompletionResponseMessage;
type Model = openai_types::CreateChatCompletionRequest;
type Message = openai_types::ChatCompletionRequestMessage;
type Function = openai_types::ChatCompletionFunctions;
type ModelArgs = openai_types::CreateChatCompletionRequestArgs;
type MessageArgs = openai_types::ChatCompletionRequestMessageArgs;
type FunctionArgs = openai_types::ChatCompletionFunctionsArgs;

static SYSTEM_INIT: &str = "You are an AI assistant of an intelligent trading system. You need to assist the user in performing a series of businesses based on the functions you are provided.\n";

lazy_static! {
    static ref MODEL_INIT: ModelArgs = ModelArgs::default()
        .max_tokens(4096u16)
        .model("gpt-3.5-turbo")
        .to_owned();
    static ref FUCTIONS_UNLOGIN: Vec<Function> = vec![
        FunctionArgs::default()
            .name("login")
            .description("Let the user login")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "username": {"type": "string"},
                    "password": {"type": "string"}
                },
                "required": ["username", "password"],
            }))
            .build()
            .unwrap(),
        FunctionArgs::default()
            .name("signup")
            .description("Sign up a new user")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "username": {"type": "string"},
                    "password": {"type": "string"}
                },
                "required": ["username", "password"],
            }))
            .build()
            .unwrap()
    ];
    static ref FUCTIONS_LOGIN: Vec<Function> = vec![
        FunctionArgs::default()
            .name("transfer")
            .description("Transfer money to another user")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "to": {"type": "string"},
                    "amount": {"type": "int32"}
                },
            }))
            .build()
            .unwrap(),
        FunctionArgs::default()
            .name("logout")
            .description("Let the user logout")
            .parameters(json!({
                "type": "object",
                "properties": {},
            }))
            .build()
            .unwrap()
    ];
}

type UserMayNull = Option<User>;

pub struct Bot {
    tx: Sender<String>,

    client: Client<OpenAIConfig>,
    system: Vec<Message>,
    messages: Vec<Message>,
    messages_tmp: Vec<Message>,
    functions: Vec<Function>,

    usermaynull: UserMayNull,
}

impl Bot {
    pub fn new(tx: Sender<String>) -> Bot {
        let functions = FUCTIONS_UNLOGIN.to_owned();
        Bot {
            tx,
            client: Client::new(),
            system: Vec::new(),
            messages: Vec::new(),
            messages_tmp: Vec::new(),
            functions,
            usermaynull: None,
        }
    }

    pub async fn chat(&mut self, draft: &str) -> Result<()> {
        // self.add_message(openai_types::Role::User, draft).unwrap();
        // unimplemented!()
        self.tx.send("1".into()).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.tx.send("2".into()).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        self.tx.send("3".into()).await?;

        Ok(())
    }

    fn add_message(&mut self, role: openai_types::Role, content: &str) -> Result<()> {
        self.messages
            .push(MessageArgs::default().role(role).content(content).build()?);
        Ok(())
    }

    fn set_system(&mut self) -> Result<()> {
        let mut system = SYSTEM_INIT.to_owned();
        match &self.usermaynull {
            Some(user) => {
                system.push_str(&formatdoc!(
                    "User info:
                        username: {}
                        balance: {}
                    .",
                    user.username,
                    user.balance
                ));
            }
            None => {
                system.push_str("User hasn't logged in.");
            }
        }
        self.system.clear();
        self.system.push(
            MessageArgs::default()
                .role(openai_types::Role::System)
                .content(system)
                .build()?,
        );
        Ok(())
    }

    fn set_functions(&mut self) -> Result<()> {
        self.functions.clear();
        match &self.usermaynull {
            Some(_) => {
                self.functions.extend(FUCTIONS_LOGIN.to_owned());
            }
            None => {
                self.functions.extend(FUCTIONS_UNLOGIN.to_owned());
            }
        }
        Ok(())
    }

    fn build_model(&self) -> Result<Model> {
        let model = MODEL_INIT
            .to_owned()
            .messages(vec![self.system.to_owned(), self.messages_tmp.to_owned()].concat())
            .functions(self.functions.to_owned())
            .function_call("auto")
            .build()?;
        Ok(model)
    }

    async fn chat_once(&self) -> Result<Response> {
        let model = self.build_model()?;
        let response = self
            .client
            .chat()
            .create(model)
            .await?
            .choices
            .get(0)
            .unwrap()
            .message
            .to_owned();
        Ok(response)
    }

    async fn chat_call_loop(&mut self) -> Result<String> {
        loop {
            let response = self.chat_once().await?;
            self.add_message(response.role, &response.content.unwrap())?;
            if let Some(function_call) = response.function_call {
                let system_response = self.perform(function_call)?;
                self.add_message(openai_types::Role::Function, &system_response)?;
            } else {
                unimplemented!()
            }
        }
    }

    fn perform(&mut self, function_call: FunctionCall) -> Result<String> {
        let args: serde_json::Value = function_call.arguments.parse()?;
        match &function_call.name[..] {
            "signup" => {
                let username = args.get_or("username", "Missing username")?;
                let password = args.get_or("password", "Missing password")?;
                self.signup(username, password);
                Ok("Signup successfully".to_string())
            }

            "login" => {
                let username = args.get_or("username", "Missing username")?;
                let password = args.get_or("password", "Missing password")?;
                self.login(username, password);
                Ok("Login successfully".to_string())
            }

            "logout" => {
                self.logout();
                Ok("Logout successfully".to_string())
            }

            "transfer" => {
                let to = args.get_or("to", "Missing to")?;
                let amount = args.get_or("amount", "Missing amount")?;
                let amount = amount
                    .parse::<i32>()
                    .or_else(|_| bail!("Amout must be an int32"))?;
                ensure!(amount > 0, "Amount must be positive");
                self.transfer(to, amount);
                Ok("Transfer successfully".to_string())
            }

            _ => bail!("Unknown function call: {}", function_call.name),
        }
    }

    fn signup(&mut self, username: &str, password: &str) {
        User::signup(username, password).unwrap();
    }

    fn login(&mut self, username: &str, password: &str) {
        let user = User::login(username, password).unwrap();
        self.usermaynull = Some(user);
        self.set_system().unwrap();
        self.set_functions().unwrap();
    }

    fn logout(&mut self) {
        self.usermaynull = None;
        self.set_system().unwrap();
        self.set_functions().unwrap();
    }

    fn transfer(&mut self, to: &str, amount: i32) {
        let user = self.usermaynull.as_mut().unwrap();
        user.transfer(to, amount).unwrap();
        self.set_system().unwrap();
    }
}

trait GetOr {
    fn get_or(&self, arg: &str, or: &str) -> Result<&str>;
}

impl GetOr for Value {
    fn get_or(&self, arg: &str, or: &str) -> Result<&str> {
        let res = self
            .get(arg)
            .ok_or(anyhow!(or.to_string()))?
            .as_str()
            .unwrap();
        Ok(res)
    }
}
