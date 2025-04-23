use std::sync::atomic::{AtomicBool, Ordering};

use axum::response::sse::Event;
use eventsource_stream::EventStreamError;
use futures::{Stream, StreamExt, pin_mut};
use serde_json::Value;
use transform_stream::{AsyncTryStream, Yielder};

use crate::error::ClewdrError;

#[derive(Debug)]
pub struct ClewdrTransformer {
    in_thinking: AtomicBool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StreamEventData {
    choices: Vec<StreamEventDelta>,
}

impl StreamEventData {
    fn new(content: String) -> Self {
        Self {
            choices: vec![StreamEventDelta {
                delta: EventContent { content },
            }],
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct NonStreamEventData {
    choices: Vec<NonStreamEventMessage>,
}

impl NonStreamEventData {
    pub fn new(content: String) -> Self {
        Self {
            choices: vec![NonStreamEventMessage {
                message: EventContent { content },
            }],
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct StreamEventDelta {
    delta: EventContent,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct NonStreamEventMessage {
    message: EventContent,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct EventContent {
    content: String,
}

impl ClewdrTransformer {
    pub fn new() -> Self {
        Self {
            in_thinking: AtomicBool::new(false),
        }
    }

    fn build(&self, selection: &str) -> Event {
        let event = Event::default();
        let data = StreamEventData::new(selection.to_string());
        event.json_data(data).unwrap()
    }

    async fn parse_buf(&mut self, buf: &str, y: &mut Yielder<Result<Event, ClewdrError>>) {
        if buf.is_empty() {
            return;
        }
        let Ok(parsed) = serde_json::from_str::<Value>(buf) else {
            return;
        };
        if let Some("thinking") = parsed["content_block"]["type"].as_str() {
            self.in_thinking.store(true, Ordering::SeqCst);
            let event = self.build("<thinking>");
            y.yield_ok(event).await;
            return;
        }
        if self.in_thinking.load(Ordering::SeqCst) {
            if let Some(thinking) = parsed["delta"]["thinking"].as_str() {
                let event = self.build(thinking);
                y.yield_ok(event).await;
                return;
            }
        }

        let Some(completion) = parsed
            .get("completion")
            .or(parsed.pointer("/delta/text"))
            .or(parsed.pointer("/choices/0/delta/content"))
            .and_then(|c| c.as_str())
        else {
            return;
        };
        if self.in_thinking.load(Ordering::SeqCst) {
            self.in_thinking.store(false, Ordering::SeqCst);
            let event = self.build("</thinking>");
            y.yield_ok(event).await;
        }
        let event = self.build(completion);
        y.yield_ok(event).await;
    }

    async fn transform(
        &mut self,
        event: eventsource_stream::Event,
        y: &mut Yielder<Result<Event, ClewdrError>>,
    ) {
        let data = event.data;
        self.parse_buf(&data, y).await;
    }

    async fn flush(&mut self, y: &mut Yielder<Result<Event, ClewdrError>>) {
        // Flush logic
        let event = Event::default();
        y.yield_ok(event.data("[DONE]")).await;
    }

    pub fn transform_stream<S>(
        mut self,
        input: S,
    ) -> AsyncTryStream<
        Event,
        ClewdrError,
        impl std::future::Future<Output = Result<(), ClewdrError>> + Send,
    >
    where
        S: Stream<Item = Result<eventsource_stream::Event, EventStreamError<rquest::Error>>>
            + Send
            + 'static,
    {
        AsyncTryStream::new(move |mut y| async move {
            pin_mut!(input);

            while let Some(chunk) = input.next().await {
                match chunk {
                    Ok(event) => {
                        self.transform(event, &mut y).await;
                    }
                    Err(e) => {
                        y.yield_err(e.into()).await;
                    }
                }
            }
            self.flush(&mut y).await;
            Ok(())
        })
    }
}
