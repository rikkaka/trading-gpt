use anyhow::{anyhow, bail, ensure, Ok, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{self as openai_types, FunctionCall},
    Client,
};
use indoc::formatdoc;
use lazy_static::lazy_static;
use log::debug;
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

static SYSTEM_INIT: &str = "You are the AI assistant of a payment system. \
You need to assist the user based on the functions you are provided. \
Note that you have access to only four functions: signup, login, transfer, and logout. \
Please focus on the functions you are provided.\n";

lazy_static! {
    static ref MODEL_INIT: ModelArgs = ModelArgs::default()
        // .max_tokens(0u16)
        .model("gpt-3.5-turbo")
        .to_owned();
    static ref FUCTIONS_UNLOGIN: Vec<Function> = vec![
        FunctionArgs::default()
            .name("login")
            .description("Let the user login. User should provide username and password")
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
            .description("Sign up a new user. User should provide username and password. You CANNOT sign up if the user hasn't provided username and password")
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
            .description("Transfer money to another user. User should provide the receiver and the amount to transfer. Note the amount must between 1 and one's balance")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "to": {"type": "string"},
                    "amount": {"type": "integer"}
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
    functions: Vec<Function>,

    usermaynull: UserMayNull,
}

impl Bot {
    pub fn new(tx: Sender<String>) -> Bot {
        let mut bot = Bot {
            tx,
            client: Client::new(),
            system: Vec::new(),
            messages: Vec::new(),
            functions: Vec::new(),
            usermaynull: None,
        };
        bot.set_system().unwrap();
        bot.set_functions().unwrap();
        bot
    }

    pub async fn chat(&mut self, draft: &str) -> Result<()> {
        self.add_message(openai_types::Role::User, draft).unwrap();
        self.chat_call_loop().await?;
        Ok(())
    }

    fn add_message(&mut self, role: openai_types::Role, content: &str) -> Result<()> {
        self.messages
            .push(MessageArgs::default().role(role).content(content).build()?);
        Ok(())
    }

    fn add_function_msg(&mut self, name: &str, content: &str) -> Result<()> {
        self.messages.push(
            MessageArgs::default()
                .role(openai_types::Role::Function)
                .name(name)
                .content(content)
                .build()?,
        );
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
        debug!("Ready to build model.");
        let model = MODEL_INIT
            .to_owned()
            .messages(vec![self.system.to_owned(), self.messages.to_owned()].concat())
            .functions(self.functions.to_owned())
            .function_call("auto")
            .build()?;
        debug!("Model built: {:?}", model);
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

    async fn chat_call_loop(&mut self) -> Result<()> {
        loop {
            let response = self.chat_once().await?;
            let message = response.content;
            if let Some(msg) = message {
                self.add_message(response.role, &msg)?;
                self.tx.send(msg).await?
            }
            if let Some(function_call) = response.function_call {
                debug!("Function call: {:?}", function_call);
                let system_response = self
                    .perform(&function_call)
                    .unwrap_or_else(|e| format!("Error: {}", e));
                self.add_function_msg(&function_call.name, &system_response)?;
                debug!("System response: {}", system_response);
            } else {
                return Ok(());
            }
        }
    }

    fn perform(&mut self, function_call: &FunctionCall) -> Result<String> {
        let args: serde_json::Value = function_call.arguments.parse()?;
        match &function_call.name[..] {
            "signup" => {
                let username = args.get_or("username", "Missing username")?;
                let password = args.get_or("password", "Missing password")?;
                self.signup(username, password)?;
                Ok("Signup successfully".to_string())
            }

            "login" => {
                let username = args.get_or("username", "Missing username")?;
                let password = args.get_or("password", "Missing password")?;
                self.login(username, password)?;
                Ok("Login successfully".to_string())
            }

            "logout" => {
                self.logout()?;
                Ok("Logout successfully".to_string())
            }

            "transfer" => {
                let to = args.get_or("to", "Missing to")?;
                let amount = args.get_or("amount", "Missing amount")?;
                self.transfer(to, amount)?;
                Ok("Transfer successfully".to_string())
            }

            _ => bail!("Unknown function call: {}", function_call.name),
        }
    }

    fn signup(&mut self, username: &str, password: &str) -> Result<()> {
        let user = User::signup(username, password).or_else(|e| bail!("Signup failed: {}", e))?;
        self.usermaynull = Some(user);
        self.set_system().unwrap();
        self.set_functions().unwrap();
        Ok(())
    }

    fn login(&mut self, username: &str, password: &str) -> Result<()> {
        let user = User::login(username, password).or_else(|e| bail!("Login failed: {}", e))?;
        self.usermaynull = Some(user);
        self.set_system().unwrap();
        self.set_functions().unwrap();
        Ok(())
    }

    fn logout(&mut self) -> Result<()> {
        self.usermaynull = None;
        self.set_system().unwrap();
        self.set_functions().unwrap();
        Ok(())
    }

    fn transfer(&mut self, to: &str, amount: i32) -> Result<()> {
        let user = self
            .usermaynull
            .as_mut()
            .ok_or_else(|| anyhow!("User not logged in"))?;
        ensure!(user.balance >= amount, "Insufficient balance");
        ensure!(user.balance > 0, "Balance must be positive");
        user.transfer(to, amount).unwrap();
        self.set_system().unwrap();
        Ok(())
    }
}

trait GetOr<'a, T>
where T: 'a {
    fn get_or(&'a self, arg: &str, or: &str) -> Result<T>;
}

impl<'a> GetOr<'a, &'a str> for Value {
    fn get_or(&'a self, arg: &str, or: &str) -> Result<&'a str> {
        let res = self
            .get(arg)
            .ok_or(anyhow!(or.to_string()))?
            .as_str()
            .unwrap();
        Ok(res)
    }
}

impl GetOr<'_, i32> for Value {
    fn get_or(&self, arg: &str, or: &str) -> Result<i32> {
        let res = self
            .get(arg)
            .ok_or(anyhow!(or.to_string()))?
            .as_i64()
            .unwrap();
        Ok(res.try_into()?)
    }
}
