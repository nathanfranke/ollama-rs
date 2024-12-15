use std::collections::HashMap;

use ollama_rs::{
    generation::chat::{
        request::ChatMessageRequest,
        tools::{
            Function, FunctionDetails, FunctionParameter, ObjectParameter, RawTool, StringParameter,
        },
        ChatMessage, MessageRole, ToolCall,
    },
    Ollama,
};

use serde::Serialize;
use tokio::io::AsyncWriteExt as _;
use tokio_stream::StreamExt as _;

const MODEL_NAME: &str = "mistral";
const HISTORY_ID: &str = "1234";

async fn chat(
    ollama: &mut Ollama,
    request: ChatMessageRequest,
    tools: Vec<RawTool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = ollama
        .send_chat_messages_with_history_stream(request, HISTORY_ID)
        .await?;

    let mut stdout = tokio::io::stdout();
    while let Some(res) = stream.next().await {
        let response = res.unwrap().message.unwrap();

        stdout.write(response.content.as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();

        if let Some(tool_calls) = response.tool_calls {
            let messages = tool_calls
                .into_iter()
                .map(|tool_call| {
                    ChatMessage::new(
                        MessageRole::Tool,
                        match tool_call {
                            ToolCall::Function(function) => match function.name.as_str() {
                                "get_current_weather" => {
                                    #[derive(Serialize)]
                                    struct Response {
                                        location: String,
                                        format: String,
                                        date: String,
                                        high_temperature: f64,
                                        low_temperature: f64,
                                        humidity: usize,
                                        precipitation_probability: usize,
                                    }

                                    let location = function.arguments.get("location").unwrap();
                                    let response = serde_json::to_string(&Response {
                                        location: location.clone(),
                                        format: "celsius".to_string(),
                                        date: "2024-12-14".to_string(),
                                        high_temperature: 7.0,
                                        low_temperature: -1.0,
                                        humidity: 70,
                                        precipitation_probability: 90,
                                    })
                                    .unwrap();
                                    println!("[DEBUG] get_current_weather: {}", response);

                                    response
                                }
                                name => panic!("invalid command: {}", name),
                            },
                        },
                    )
                })
                .collect::<Vec<_>>();

            let request =
                ChatMessageRequest::new(MODEL_NAME.to_string(), messages).tools(tools.clone());
            Box::pin(chat(ollama, request, tools.clone())).await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ollama = Ollama::default();

    ollama.clear_messages_for_id(HISTORY_ID);

    let weather = RawTool::Function(Function {
        function: FunctionDetails {
            name: "get_current_weather".to_string(),
            description: "Get the current weather for a location.".to_string(),
            parameters: FunctionParameter::Object(ObjectParameter {
                properties: {
                    let mut map = HashMap::new();
                    map.insert(
                        "location".to_string(),
                        FunctionParameter::String(StringParameter {
                            description: Some(
                                "The location to get the weather for, e.g. San Francisco, CA"
                                    .to_string(),
                            ),
                        }),
                    );
                    map
                },
                required: vec!["location".to_string()],
            }),
        },
    });
    println!(
        "[DEBUG] schema: {}",
        serde_json::to_string(&weather).unwrap()
    );
    let tools = vec![weather];

    let message = ChatMessage::new(
        MessageRole::User,
        "How is weather today in Paris?".to_string(),
    );
    let request =
        ChatMessageRequest::new(MODEL_NAME.to_string(), vec![message]).tools(tools.clone());

    chat(&mut ollama, request, tools.clone()).await?;
    println!();

    Ok(())
}
