use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use actix_web::web::Data;
use reqwest::{Client, Response};
use serde::{Deserialize};
use serde_json::{Value, json};
use std::collections::HashMap;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = Data::new(State {users: std::sync::Mutex::new(HashMap::new())});

    // env_logger::init_from_env(env_logger::Env::default().default_filter_or("trace"));
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(send_message)
            .service(get_next_word)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}




struct Conversation {
    prompt: Value,
    res: Option<Response>,
    prompted: bool,
}
#[derive(Debug, Deserialize)]
struct StreamChunkMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct StreamChunk {
    message: Option<StreamChunkMessage>,
    done: Option<bool>,
}


impl Conversation {
    async fn send(&mut self, message: String) {
        let client = Client::new();

        // Insert system prompt once
        if !self.prompted {
            if let Some(Value::Array(messages)) = self.prompt.get_mut("messages") {
                messages.insert(0, json!({
                    "role": "system",
                    "content": "You are the opposing debater. Argue strongly against the user's position. Do not agree unless the user proves their point convincingly. Keep it brief. If the user's first message doesn't declare a topic, you can choose any topic you want. It can be silly, but it must be a devisive topic where each side can be argued. Avoid being too dismissive. Evaluate the user's counterargument and recognize if it has any merit. Maintain a baseline level of respect for the user and don't cross the line until they do."
                }));

            }
            self.prompted = true;
        }

        // Add user message
        if let Some(Value::Array(messages)) = self.prompt.get_mut("messages") {
            messages.push(json!({
                "role": "user",
                "content": message,
            }));
        }

        // Build properly formatted Ollama chat request
        let req_body = json!({
            "model": "gemma3:4b",        // Make sure this matches `ollama list`
            "stream": true,
            "messages": self.prompt["messages"]
        });

        println!("Sending JSON to Ollama: {}", req_body);

        let res = client
            .post("http://localhost:11434/api/chat")
            .json(&req_body)
            .send()
            .await
            .expect("Failed to send request to AI server");

        self.res = Some(res);

    }



    async fn next_word(&mut self) -> Option<String> {
        if let Some(res) = &mut self.res {
            if let Some(chunk) = res.chunk().await.unwrap() {

                let text = String::from_utf8(chunk.to_vec()).unwrap();
                let parsed: StreamChunk = match serde_json::from_str(&text) {
                    Ok(p) => p,
                    Err(_) => return None, // Skip malformed chunks
                };

                // Stop if stream ended
                if parsed.done.unwrap_or(false) {
                    return None;
                }

                if let Some(msg) = parsed.message {
                    // Append assistant message to retained conversation
                    if let Value::Object(map) = &mut self.prompt {
                        if let Some(Value::Array(t)) = map.get_mut("messages") {
                            t.push(json!({
                                "role": msg.role,
                                "content": msg.content.clone(),
                            }));
                        }
                    }

                    return Some(msg.content);
                }
            }
        }

        None
    }


}

struct State {
    users: std::sync::Mutex<HashMap<String, Conversation>>,
}

#[get("/{id}/{msg}")]
async fn send_message(path: web::Path<(String, String)>, state: web::Data<State>) -> impl Responder {
    let pin = path.into_inner();

    let id = pin.0;
    println!("{id}");
    let msg = pin.1;

    let mut users = state.users.lock().unwrap();

    if let None = users.get(&id) {
        (*users).insert(id.clone(), Conversation {
            prompt: json!({
                "messages": []
            }),
            res: None,
            prompted: false,
        });
    }



    let user = users.get_mut(&id).unwrap();



    user.send(msg).await;


    HttpResponse::Ok().body("Message Sent".to_string())
}

#[get("/{id}")]
async fn get_next_word(path: web::Path<String>, state: web::Data<State>) -> impl Responder {
    let id = path.into_inner();
    let mut users = state.users.lock().unwrap();

    let user = if let Some(t) = users.get_mut(&id) { t } else { println!("WHY"); return HttpResponse::NoContent().body(""); };

    if let Some(msg) = user.next_word().await {
        HttpResponse::Ok().body(msg)
    } else {
        HttpResponse::NoContent().body("")
    }
}