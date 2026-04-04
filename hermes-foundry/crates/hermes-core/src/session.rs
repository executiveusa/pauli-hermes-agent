/// Session — Conversation history with in-memory store
/// In production, swap the Vec-based store for SQLite via rusqlite.

use chrono::Utc;
use hermes_types::{Conversation, Message, Role};

pub struct Session {
    pub conversation: Conversation,
    pub messages:     Vec<Message>,
}

impl Session {
    pub fn new(model: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            conversation: Conversation {
                id:            id.clone(),
                title:         "New conversation".to_string(),
                model:         model.to_string(),
                message_count: 0,
                created_at:    Utc::now(),
                updated_at:    Utc::now(),
            },
            messages: Vec::new(),
        }
    }

    pub fn add_user_message(&mut self, content: String) {
        let msg = Message {
            id:              uuid::Uuid::new_v4().to_string(),
            role:            Role::User,
            content,
            conversation_id: self.conversation.id.clone(),
            model:           None,
            created_at:      Utc::now(),
            tool_calls:      vec![],
        };
        if self.messages.len() == 0 {
            // Set title from first message
            self.conversation.title = msg.content.chars().take(60).collect();
        }
        self.messages.push(msg);
        self.conversation.message_count += 1;
        self.conversation.updated_at = Utc::now();
    }

    pub fn add_assistant_message(&mut self, content: String, model: &str) {
        let msg = Message {
            id:              uuid::Uuid::new_v4().to_string(),
            role:            Role::Assistant,
            content,
            conversation_id: self.conversation.id.clone(),
            model:           Some(model.to_string()),
            created_at:      Utc::now(),
            tool_calls:      vec![],
        };
        self.messages.push(msg);
        self.conversation.message_count += 1;
        self.conversation.updated_at = Utc::now();
    }

    pub fn as_openai_messages(&self) -> Vec<serde_json::Value> {
        self.messages.iter().map(|m| {
            serde_json::json!({
                "role": match m.role {
                    Role::User      => "user",
                    Role::Assistant => "assistant",
                    Role::System    => "system",
                    Role::Tool      => "tool",
                },
                "content": m.content,
            })
        }).collect()
    }
}
