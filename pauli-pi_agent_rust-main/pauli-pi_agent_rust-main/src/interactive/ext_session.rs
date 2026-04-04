use super::conversation::extension_model_from_entry;
use super::*;

#[derive(Clone)]
pub(super) struct InteractiveExtensionHostActions {
    pub(super) session: Arc<Mutex<Session>>,
    pub(super) agent: Arc<Mutex<Agent>>,
    pub(super) event_tx: mpsc::Sender<PiMsg>,
    pub(super) extension_streaming: Arc<AtomicBool>,
    pub(super) user_queue: Arc<StdMutex<InteractiveMessageQueue>>,
    pub(super) injected_queue: Arc<StdMutex<InjectedMessageQueue>>,
}

impl InteractiveExtensionHostActions {
    const fn should_trigger_turn(
        deliver_as: Option<ExtensionDeliverAs>,
        trigger_turn: bool,
    ) -> bool {
        trigger_turn && !matches!(deliver_as, Some(ExtensionDeliverAs::NextTurn))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn queue_custom_message(
        &self,
        deliver_as: Option<ExtensionDeliverAs>,
        message: ModelMessage,
    ) -> crate::error::Result<()> {
        let deliver_as = deliver_as.unwrap_or(ExtensionDeliverAs::Steer);
        let kind = match deliver_as {
            ExtensionDeliverAs::FollowUp => QueuedMessageKind::FollowUp,
            ExtensionDeliverAs::Steer | ExtensionDeliverAs::NextTurn => QueuedMessageKind::Steering,
        };
        let Ok(mut queue) = self.injected_queue.lock() else {
            return Ok(());
        };
        match kind {
            QueuedMessageKind::Steering => queue.push_steering(message),
            QueuedMessageKind::FollowUp => queue.push_follow_up(message),
        }
        Ok(())
    }

    async fn append_to_session(&self, message: ModelMessage) -> crate::error::Result<()> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let mut session_guard = self
            .session
            .lock(&cx)
            .await
            .map_err(|e| crate::error::Error::session(e.to_string()))?;
        session_guard.append_model_message(message);
        Ok(())
    }
}

#[async_trait]
impl ExtensionHostActions for InteractiveExtensionHostActions {
    async fn send_message(&self, message: ExtensionSendMessage) -> crate::error::Result<()> {
        let custom_message = ModelMessage::Custom(CustomMessage {
            content: message.content,
            custom_type: message.custom_type,
            display: message.display,
            details: message.details,
            timestamp: Utc::now().timestamp_millis(),
        });
        let cx = Cx::current().unwrap_or_else(Cx::for_request);

        let is_streaming = self.extension_streaming.load(Ordering::SeqCst);
        if is_streaming {
            // Queue into the agent loop; session persistence happens when the message is delivered.
            self.queue_custom_message(message.deliver_as, custom_message.clone())?;
            if let ModelMessage::Custom(custom) = &custom_message {
                if custom.display {
                    let _ = enqueue_pi_event(
                        &self.event_tx,
                        &cx,
                        PiMsg::SystemNote(custom.content.clone()),
                    )
                    .await;
                }
            }
            return Ok(());
        }

        // Agent is idle: persist immediately and update in-memory history so it affects the next run.
        self.append_to_session(custom_message.clone()).await?;

        if let Ok(mut agent_guard) = self.agent.lock(&cx).await {
            agent_guard.add_message(custom_message.clone());
        }

        if let ModelMessage::Custom(custom) = &custom_message {
            if custom.display {
                let _ = enqueue_pi_event(
                    &self.event_tx,
                    &cx,
                    PiMsg::SystemNote(custom.content.clone()),
                )
                .await;
            }
        }

        if Self::should_trigger_turn(message.deliver_as, message.trigger_turn) {
            let _ = enqueue_pi_event(
                &self.event_tx,
                &cx,
                PiMsg::EnqueuePendingInput(PendingInput::Continue),
            )
            .await;
        }

        Ok(())
    }

    async fn send_user_message(
        &self,
        message: ExtensionSendUserMessage,
    ) -> crate::error::Result<()> {
        let is_streaming = self.extension_streaming.load(Ordering::SeqCst);
        if is_streaming {
            let deliver_as = message.deliver_as.unwrap_or(ExtensionDeliverAs::Steer);
            let Ok(mut queue) = self.user_queue.lock() else {
                return Ok(());
            };
            match deliver_as {
                ExtensionDeliverAs::FollowUp => queue.push_follow_up(message.text),
                ExtensionDeliverAs::Steer | ExtensionDeliverAs::NextTurn => {
                    queue.push_steering(message.text);
                }
            }
            return Ok(());
        }

        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let _ = enqueue_pi_event(
            &self.event_tx,
            &cx,
            PiMsg::EnqueuePendingInput(PendingInput::Text(message.text)),
        )
        .await;
        Ok(())
    }
}

pub(super) struct InteractiveExtensionSession {
    pub(super) session: Arc<Mutex<Session>>,
    pub(super) model_entry: Arc<StdMutex<ModelEntry>>,
    pub(super) is_streaming: Arc<AtomicBool>,
    pub(super) is_compacting: Arc<AtomicBool>,
    pub(super) config: Config,
    pub(super) save_enabled: bool,
}

#[async_trait]
impl ExtensionSession for InteractiveExtensionSession {
    async fn get_state(&self) -> Value {
        let model = {
            let guard = self
                .model_entry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            extension_model_from_entry(&guard)
        };

        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let (
            session_file,
            session_id,
            session_name,
            message_count,
            thinking_level,
            durability_mode,
            autosave_pending_mutations,
            autosave_max_pending_mutations,
            autosave_flush_failed_count,
            autosave_backpressure,
            persistence_status,
        ) = self.session.lock(&cx).await.map_or_else(
            |_| {
                (
                    None,
                    String::new(),
                    None,
                    0,
                    "off".to_string(),
                    "balanced".to_string(),
                    0usize,
                    0usize,
                    0u64,
                    false,
                    "unknown".to_string(),
                )
            },
            |guard| {
                let message_count = guard
                    .entries_for_current_path()
                    .iter()
                    .filter(|entry| matches!(entry, SessionEntry::Message(_)))
                    .count();
                let session_name = guard.get_name();
                let thinking_level = guard
                    .header
                    .thinking_level
                    .clone()
                    .unwrap_or_else(|| "off".to_string());
                let autosave_metrics = guard.autosave_metrics();
                let durability_mode = guard.autosave_durability_mode().as_str().to_string();
                let autosave_backpressure = autosave_metrics.max_pending_mutations > 0
                    && autosave_metrics.pending_mutations >= autosave_metrics.max_pending_mutations;
                let persistence_status = if autosave_metrics.flush_failed > 0 {
                    "degraded"
                } else if autosave_backpressure {
                    "backpressure"
                } else if autosave_metrics.pending_mutations > 0 {
                    "draining"
                } else {
                    "healthy"
                }
                .to_string();
                (
                    guard.path.as_ref().map(|p| p.display().to_string()),
                    guard.header.id.clone(),
                    session_name,
                    message_count,
                    thinking_level,
                    durability_mode,
                    autosave_metrics.pending_mutations,
                    autosave_metrics.max_pending_mutations,
                    autosave_metrics.flush_failed,
                    autosave_backpressure,
                    persistence_status,
                )
            },
        );

        json!({
            "model": model,
            "thinkingLevel": thinking_level,
            "isStreaming": self.is_streaming.load(Ordering::SeqCst),
            "isCompacting": self.is_compacting.load(Ordering::SeqCst),
            "steeringMode": self.config.steering_queue_mode().as_str(),
            "followUpMode": self.config.follow_up_queue_mode().as_str(),
            "sessionFile": session_file,
            "sessionId": session_id,
            "sessionName": session_name,
            "autoCompactionEnabled": self.config.compaction_enabled(),
            "messageCount": message_count,
            "pendingMessageCount": autosave_pending_mutations,
            "durabilityMode": durability_mode,
            "autosavePendingMutations": autosave_pending_mutations,
            "autosaveMaxPendingMutations": autosave_max_pending_mutations,
            "autosaveFlushFailedCount": autosave_flush_failed_count,
            "autosaveBackpressure": autosave_backpressure,
            "persistenceStatus": persistence_status,
        })
    }

    async fn get_messages(&self) -> Vec<SessionMessage> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let Ok(guard) = self.session.lock(&cx).await else {
            return Vec::new();
        };
        guard
            .entries_for_current_path()
            .iter()
            .filter_map(|entry| match entry {
                SessionEntry::Message(msg) => match msg.message {
                    SessionMessage::User { .. }
                    | SessionMessage::Assistant { .. }
                    | SessionMessage::ToolResult { .. }
                    | SessionMessage::BashExecution { .. }
                    | SessionMessage::Custom { .. } => Some(msg.message.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    async fn get_entries(&self) -> Vec<Value> {
        // Spec §3.1: return ALL session entries (entire session file), append order.
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let Ok(guard) = self.session.lock(&cx).await else {
            return Vec::new();
        };
        guard
            .entries
            .iter()
            .filter_map(|entry| serde_json::to_value(entry).ok())
            .collect()
    }

    async fn get_branch(&self) -> Vec<Value> {
        // Spec §3.2: return current path from root to leaf.
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let Ok(guard) = self.session.lock(&cx).await else {
            return Vec::new();
        };
        guard
            .entries_for_current_path()
            .iter()
            .filter_map(|entry| serde_json::to_value(*entry).ok())
            .collect()
    }

    async fn set_name(&self, name: String) -> crate::error::Result<()> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let mut guard =
            self.session.lock(&cx).await.map_err(|err| {
                crate::error::Error::session(format!("session lock failed: {err}"))
            })?;
        guard.set_name(&name);
        if self.save_enabled {
            guard.save().await?;
        }
        Ok(())
    }

    async fn append_message(&self, message: SessionMessage) -> crate::error::Result<()> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let mut guard =
            self.session.lock(&cx).await.map_err(|err| {
                crate::error::Error::session(format!("session lock failed: {err}"))
            })?;
        guard.append_message(message);
        if self.save_enabled {
            guard.save().await?;
        }
        Ok(())
    }

    async fn append_custom_entry(
        &self,
        custom_type: String,
        data: Option<Value>,
    ) -> crate::error::Result<()> {
        if custom_type.trim().is_empty() {
            return Err(crate::error::Error::validation(
                "customType must not be empty",
            ));
        }
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let mut guard =
            self.session.lock(&cx).await.map_err(|err| {
                crate::error::Error::session(format!("session lock failed: {err}"))
            })?;
        guard.append_custom_entry(custom_type, data);
        if self.save_enabled {
            guard.save().await?;
        }
        Ok(())
    }

    async fn set_model(&self, provider: String, model_id: String) -> crate::error::Result<()> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let mut guard =
            self.session.lock(&cx).await.map_err(|err| {
                crate::error::Error::session(format!("session lock failed: {err}"))
            })?;
        let changed = guard.header.provider.as_deref() != Some(provider.as_str())
            || guard.header.model_id.as_deref() != Some(model_id.as_str());
        if changed {
            guard.append_model_change(provider.clone(), model_id.clone());
        }
        guard.set_model_header(Some(provider), Some(model_id), None);
        if self.save_enabled {
            guard.save().await?;
        }
        Ok(())
    }

    async fn get_model(&self) -> (Option<String>, Option<String>) {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let Ok(guard) = self.session.lock(&cx).await else {
            return (None, None);
        };
        (guard.header.provider.clone(), guard.header.model_id.clone())
    }

    async fn set_thinking_level(&self, level: String) -> crate::error::Result<()> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let effective_level = match level.parse::<crate::model::ThinkingLevel>() {
            Ok(parsed) => self
                .model_entry
                .lock()
                .map(|entry| entry.clamp_thinking_level(parsed).to_string())
                .unwrap_or(level),
            Err(_) => level,
        };
        let mut guard =
            self.session.lock(&cx).await.map_err(|err| {
                crate::error::Error::session(format!("session lock failed: {err}"))
            })?;
        let changed = guard.header.thinking_level.as_deref() != Some(effective_level.as_str());
        guard.set_model_header(None, None, Some(effective_level.clone()));
        if changed {
            guard.append_thinking_level_change(effective_level);
        }
        if changed && self.save_enabled {
            guard.save().await?;
        }
        Ok(())
    }

    async fn get_thinking_level(&self) -> Option<String> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let Ok(guard) = self.session.lock(&cx).await else {
            return None;
        };
        guard.header.thinking_level.clone()
    }

    async fn set_label(
        &self,
        target_id: String,
        label: Option<String>,
    ) -> crate::error::Result<()> {
        let cx = Cx::current().unwrap_or_else(Cx::for_request);
        let mut guard =
            self.session.lock(&cx).await.map_err(|err| {
                crate::error::Error::session(format!("session lock failed: {err}"))
            })?;
        if guard.add_label(&target_id, label).is_none() {
            return Err(crate::error::Error::validation(format!(
                "target entry '{target_id}' not found in session"
            )));
        }
        if self.save_enabled {
            guard.save().await?;
        }
        Ok(())
    }
}

pub fn format_extension_ui_prompt(request: &ExtensionUiRequest) -> String {
    let title = request
        .payload
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Extension");
    let message = request
        .payload
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("");

    // Show provenance: which extension is making this request.
    let provenance = request
        .extension_id
        .as_deref()
        .or_else(|| request.payload.get("extension_id").and_then(Value::as_str))
        .unwrap_or("unknown");

    match request.method.as_str() {
        "confirm" => {
            format!("[{provenance}] confirm: {title}\n{message}\n\nEnter yes/no, or 'cancel'.")
        }
        "select" => {
            let options = request
                .payload
                .get("options")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();

            let mut out = String::new();
            let _ = writeln!(&mut out, "[{provenance}] select: {title}");
            if !message.trim().is_empty() {
                let _ = writeln!(&mut out, "{message}");
            }
            for (idx, opt) in options.iter().enumerate() {
                let label = opt
                    .get("label")
                    .and_then(Value::as_str)
                    .or_else(|| opt.get("value").and_then(Value::as_str))
                    .or_else(|| opt.as_str())
                    .unwrap_or("");
                let _ = writeln!(&mut out, "  {}) {label}", idx + 1);
            }
            out.push_str("\nEnter a number, label, or 'cancel'.");
            out
        }
        "input" => format!("[{provenance}] input: {title}\n{message}"),
        "editor" => format!("[{provenance}] editor: {title}\n{message}"),
        _ => format!("[{provenance}] {title} {message}"),
    }
}

pub fn parse_extension_ui_response(
    request: &ExtensionUiRequest,
    input: &str,
) -> Result<ExtensionUiResponse, String> {
    let trimmed = input.trim();

    if trimmed.eq_ignore_ascii_case("cancel") || trimmed.eq_ignore_ascii_case("c") {
        return Ok(ExtensionUiResponse {
            id: request.id.clone(),
            value: None,
            cancelled: true,
        });
    }

    match request.method.as_str() {
        "confirm" => {
            let value = match trimmed.to_lowercase().as_str() {
                "y" | "yes" | "true" | "1" => true,
                "n" | "no" | "false" | "0" => false,
                _ => {
                    return Err("Invalid confirmation. Enter yes/no, or 'cancel'.".to_string());
                }
            };
            Ok(ExtensionUiResponse {
                id: request.id.clone(),
                value: Some(Value::Bool(value)),
                cancelled: false,
            })
        }
        "select" => {
            let options = request
                .payload
                .get("options")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    "Invalid selection. Enter a number, label, or 'cancel'.".to_string()
                })?;

            if let Ok(index) = trimmed.parse::<usize>() {
                if index > 0 && index <= options.len() {
                    let chosen = &options[index - 1];
                    let value = chosen
                        .get("value")
                        .cloned()
                        .or_else(|| chosen.get("label").cloned())
                        .or_else(|| chosen.as_str().map(|s| Value::String(s.to_string())));
                    return Ok(ExtensionUiResponse {
                        id: request.id.clone(),
                        value,
                        cancelled: false,
                    });
                }
            }

            let lowered = trimmed.to_lowercase();
            for option in options {
                if let Some(value_str) = option.as_str() {
                    if value_str.to_lowercase() == lowered {
                        return Ok(ExtensionUiResponse {
                            id: request.id.clone(),
                            value: Some(Value::String(value_str.to_string())),
                            cancelled: false,
                        });
                    }
                }

                let label = option.get("label").and_then(Value::as_str).unwrap_or("");
                if !label.is_empty() && label.to_lowercase() == lowered {
                    let value = option.get("value").cloned().or_else(|| {
                        option
                            .get("label")
                            .and_then(Value::as_str)
                            .map(|s| Value::String(s.to_string()))
                    });
                    return Ok(ExtensionUiResponse {
                        id: request.id.clone(),
                        value,
                        cancelled: false,
                    });
                }

                if let Some(value_str) = option.get("value").and_then(Value::as_str) {
                    if value_str.to_lowercase() == lowered {
                        return Ok(ExtensionUiResponse {
                            id: request.id.clone(),
                            value: Some(Value::String(value_str.to_string())),
                            cancelled: false,
                        });
                    }
                }
            }

            Err("Invalid selection. Enter a number, label, or 'cancel'.".to_string())
        }
        _ => Ok(ExtensionUiResponse {
            id: request.id.clone(),
            value: Some(Value::String(input.to_string())),
            cancelled: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::agent::{Agent, AgentConfig};
    use crate::config::Config;
    use crate::model::StreamEvent;
    use crate::models::ModelEntry;
    use crate::provider::{Context, InputType, Model, ModelCost, Provider, StreamOptions};
    use crate::session::{Session, SessionMessage};
    use crate::tools::ToolRegistry;
    use asupersync::runtime::RuntimeBuilder;
    use async_trait::async_trait;
    use futures::stream;
    use serde_json::json;
    use std::collections::HashMap;
    use std::path::Path;
    use std::pin::Pin;
    use std::time::Duration;

    type TestStream =
        Pin<Box<dyn futures::Stream<Item = crate::error::Result<StreamEvent>> + Send>>;
    type HostActionsHarness = (
        InteractiveExtensionHostActions,
        mpsc::Receiver<PiMsg>,
        Arc<Mutex<Session>>,
        Arc<Mutex<Agent>>,
    );

    struct NoopProvider;

    #[async_trait]
    impl Provider for NoopProvider {
        fn name(&self) -> &'static str {
            "noop"
        }

        fn api(&self) -> &'static str {
            "noop"
        }

        fn model_id(&self) -> &'static str {
            "noop-model"
        }

        async fn stream(
            &self,
            _context: &Context<'_>,
            _options: &StreamOptions,
        ) -> crate::error::Result<TestStream> {
            Ok(Box::pin(stream::empty()))
        }
    }

    fn build_host_actions() -> HostActionsHarness {
        build_host_actions_with_capacity(8)
    }

    fn build_host_actions_with_capacity(capacity: usize) -> HostActionsHarness {
        let session = Arc::new(Mutex::new(Session::in_memory()));
        let provider: Arc<dyn Provider> = Arc::new(NoopProvider);
        let agent = Arc::new(Mutex::new(Agent::new(
            provider,
            ToolRegistry::new(&[], Path::new("."), None),
            AgentConfig::default(),
        )));
        let (event_tx, event_rx) = mpsc::channel(capacity);
        (
            InteractiveExtensionHostActions {
                session: Arc::clone(&session),
                agent: Arc::clone(&agent),
                event_tx,
                extension_streaming: Arc::new(AtomicBool::new(false)),
                user_queue: Arc::new(StdMutex::new(InteractiveMessageQueue::new(
                    QueueMode::OneAtATime,
                    QueueMode::OneAtATime,
                ))),
                injected_queue: Arc::new(StdMutex::new(InjectedMessageQueue::new(
                    QueueMode::OneAtATime,
                    QueueMode::OneAtATime,
                ))),
            },
            event_rx,
            session,
            agent,
        )
    }

    fn dummy_model_entry() -> ModelEntry {
        ModelEntry {
            model: Model {
                id: "noop-model".to_string(),
                name: "Noop Model".to_string(),
                api: "noop".to_string(),
                provider: "noop".to_string(),
                base_url: "https://example.invalid".to_string(),
                reasoning: false,
                input: vec![InputType::Text],
                cost: ModelCost {
                    input: 0.0,
                    output: 0.0,
                    cache_read: 0.0,
                    cache_write: 0.0,
                },
                context_window: 8192,
                max_tokens: 1024,
                headers: HashMap::new(),
            },
            api_key: None,
            headers: HashMap::new(),
            auth_header: true,
            compat: None,
            oauth_config: None,
        }
    }

    #[test]
    fn interactive_extension_session_get_messages_includes_custom_messages() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let session = Arc::new(Mutex::new(Session::in_memory()));
            let cx = Cx::for_request();
            {
                let mut guard = session.lock(&cx).await.expect("lock session");
                guard.append_message(SessionMessage::Custom {
                    custom_type: "note".to_string(),
                    content: "hello".to_string(),
                    display: true,
                    details: Some(json!({ "from": "test" })),
                    timestamp: Some(1),
                });
            }

            let ext_session = InteractiveExtensionSession {
                session,
                model_entry: Arc::new(StdMutex::new(dummy_model_entry())),
                is_streaming: Arc::new(AtomicBool::new(false)),
                is_compacting: Arc::new(AtomicBool::new(false)),
                config: Config::default(),
                save_enabled: false,
            };

            let messages = ext_session.get_messages().await;
            assert!(
                messages.iter().any(|message| {
                    matches!(
                        message,
                        SessionMessage::Custom {
                            custom_type,
                            content,
                            display,
                            details,
                            ..
                        } if custom_type == "note"
                            && content == "hello"
                            && *display
                            && details.as_ref().and_then(|value| value.get("from").and_then(Value::as_str))
                                == Some("test")
                    )
                }),
                "expected custom message in interactive extension session messages, got {messages:?}"
            );
        });
    }

    #[test]
    fn interactive_extension_session_set_name_inherits_cancelled_context_when_locked() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let session = Arc::new(Mutex::new(Session::in_memory()));
            let ext_session = InteractiveExtensionSession {
                session: Arc::clone(&session),
                model_entry: Arc::new(StdMutex::new(dummy_model_entry())),
                is_streaming: Arc::new(AtomicBool::new(false)),
                is_compacting: Arc::new(AtomicBool::new(false)),
                config: Config::default(),
                save_enabled: false,
            };

            let hold_cx = Cx::for_request();
            let held_guard = session.lock(&hold_cx).await.expect("lock session");

            let ambient_cx = Cx::for_testing();
            ambient_cx.set_cancel_requested(true);
            let _current = Cx::set_current(Some(ambient_cx));
            let inner = asupersync::time::timeout(
                asupersync::time::wall_now(),
                Duration::from_millis(100),
                ext_session.set_name("cancelled-name".to_string()),
            )
            .await;
            let outcome = inner.expect("cancelled helper should finish before timeout");
            let err = outcome.expect_err("lock acquisition should honor inherited cancellation");
            assert!(
                err.to_string().contains("session lock failed"),
                "unexpected error: {err}"
            );

            drop(held_guard);

            let cx = Cx::for_request();
            let guard = session.lock(&cx).await.expect("lock session");
            assert_eq!(guard.get_name(), None);
        });
    }

    #[test]
    fn idle_send_message_trigger_turn_enqueues_continue() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let (actions, event_rx, session, agent) = build_host_actions();

            actions
                .send_message(ExtensionSendMessage {
                    extension_id: Some("ext".to_string()),
                    custom_type: "note".to_string(),
                    content: "continue-now".to_string(),
                    display: false,
                    details: None,
                    deliver_as: Some(ExtensionDeliverAs::Steer),
                    trigger_turn: true,
                })
                .await
                .expect("send_message");

            let queued = event_rx.try_recv().expect("continue should be queued");
            assert!(matches!(
                queued,
                PiMsg::EnqueuePendingInput(PendingInput::Continue)
            ));

            let cx = Cx::for_request();
            let session_guard = session.lock(&cx).await.expect("lock session");
            assert!(
                session_guard
                    .to_messages_for_current_path()
                    .iter()
                    .any(|msg| {
                        matches!(
                            msg,
                            ModelMessage::Custom(CustomMessage { custom_type, content, .. })
                                if custom_type == "note" && content == "continue-now"
                        )
                    })
            );
            drop(session_guard);

            let agent_guard = agent.lock(&cx).await.expect("lock agent");
            assert!(agent_guard.messages().iter().any(|msg| {
                matches!(
                    msg,
                    ModelMessage::Custom(CustomMessage { custom_type, content, .. })
                        if custom_type == "note" && content == "continue-now"
                )
            }));
        });
    }

    #[test]
    fn idle_send_message_next_turn_ignores_trigger_turn() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let (actions, event_rx, _session, _agent) = build_host_actions();

            actions
                .send_message(ExtensionSendMessage {
                    extension_id: Some("ext".to_string()),
                    custom_type: "note".to_string(),
                    content: "defer".to_string(),
                    display: false,
                    details: None,
                    deliver_as: Some(ExtensionDeliverAs::NextTurn),
                    trigger_turn: true,
                })
                .await
                .expect("send_message");

            assert!(
                event_rx.try_recv().is_err(),
                "nextTurn should stay deferred even when triggerTurn is set"
            );
        });
    }

    #[test]
    fn streaming_send_message_preserves_display_note_under_backpressure() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let (actions, event_rx, _session, _agent) = build_host_actions_with_capacity(1);
            actions.extension_streaming.store(true, Ordering::SeqCst);
            actions
                .event_tx
                .try_send(PiMsg::System("busy".to_string()))
                .expect("fill bounded event channel");

            let send_message = actions.send_message(ExtensionSendMessage {
                extension_id: Some("ext".to_string()),
                custom_type: "note".to_string(),
                content: "visible".to_string(),
                display: true,
                details: None,
                deliver_as: Some(ExtensionDeliverAs::Steer),
                trigger_turn: false,
            });
            let recv_cx = Cx::for_request();
            let recv_messages = async {
                let first = event_rx.recv(&recv_cx).await.expect("busy message");
                let second = event_rx.recv(&recv_cx).await.expect("display note");
                (first, second)
            };

            let (result, (first, second)) = futures::join!(send_message, recv_messages);

            result.expect("send_message");
            assert!(matches!(first, PiMsg::System(text) if text == "busy"));
            assert!(matches!(second, PiMsg::SystemNote(text) if text == "visible"));
        });
    }

    #[test]
    fn idle_send_message_preserves_display_and_continue_under_backpressure() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let (actions, event_rx, _session, _agent) = build_host_actions_with_capacity(1);
            actions
                .event_tx
                .try_send(PiMsg::System("busy".to_string()))
                .expect("fill bounded event channel");

            let send_message = actions.send_message(ExtensionSendMessage {
                extension_id: Some("ext".to_string()),
                custom_type: "note".to_string(),
                content: "continue-now".to_string(),
                display: true,
                details: None,
                deliver_as: Some(ExtensionDeliverAs::Steer),
                trigger_turn: true,
            });
            let recv_cx = Cx::for_request();
            let recv_messages = async {
                let first = event_rx.recv(&recv_cx).await.expect("busy message");
                let second = event_rx.recv(&recv_cx).await.expect("display note");
                let third = event_rx.recv(&recv_cx).await.expect("continue enqueue");
                (first, second, third)
            };

            let (result, (first, second, third)) = futures::join!(send_message, recv_messages);

            result.expect("send_message");
            assert!(matches!(first, PiMsg::System(text) if text == "busy"));
            assert!(matches!(second, PiMsg::SystemNote(text) if text == "continue-now"));
            assert!(matches!(
                third,
                PiMsg::EnqueuePendingInput(PendingInput::Continue)
            ));
        });
    }

    #[test]
    fn idle_send_user_message_preserves_text_under_backpressure() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let (actions, event_rx, _session, _agent) = build_host_actions_with_capacity(1);
            actions
                .event_tx
                .try_send(PiMsg::System("busy".to_string()))
                .expect("fill bounded event channel");

            let send_message = actions.send_user_message(ExtensionSendUserMessage {
                extension_id: Some("ext".to_string()),
                text: "hello from extension".to_string(),
                deliver_as: None,
            });
            let recv_cx = Cx::for_request();
            let recv_messages = async {
                let first = event_rx.recv(&recv_cx).await.expect("busy message");
                let second = event_rx.recv(&recv_cx).await.expect("user input enqueue");
                (first, second)
            };

            let (result, (first, second)) = futures::join!(send_message, recv_messages);

            result.expect("send_user_message");
            assert!(matches!(first, PiMsg::System(text) if text == "busy"));
            assert!(matches!(
                second,
                PiMsg::EnqueuePendingInput(PendingInput::Text(text))
                    if text == "hello from extension"
            ));
        });
    }

    #[test]
    fn set_thinking_level_clamps_and_dedupes_for_non_reasoning_models() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let mut entry = dummy_model_entry();
            entry.model.reasoning = false;
            let session = Arc::new(Mutex::new(Session::in_memory()));
            let ext_session = InteractiveExtensionSession {
                session: Arc::clone(&session),
                model_entry: Arc::new(StdMutex::new(entry)),
                is_streaming: Arc::new(AtomicBool::new(false)),
                is_compacting: Arc::new(AtomicBool::new(false)),
                config: Config::default(),
                save_enabled: false,
            };

            ext_session
                .set_thinking_level("high".to_string())
                .await
                .expect("first thinking update");
            ext_session
                .set_thinking_level("high".to_string())
                .await
                .expect("second thinking update");

            let cx = Cx::for_request();
            let guard = session.lock(&cx).await.expect("lock session");
            assert_eq!(guard.header.thinking_level.as_deref(), Some("off"));
            let thinking_changes = guard
                .entries_for_current_path()
                .iter()
                .filter(|entry| {
                    matches!(entry, crate::session::SessionEntry::ThinkingLevelChange(_))
                })
                .count();
            assert_eq!(thinking_changes, 1);
        });
    }

    #[test]
    fn set_model_avoids_duplicate_history_for_same_target() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let session = Arc::new(Mutex::new(Session::in_memory()));
            let ext_session = InteractiveExtensionSession {
                session: Arc::clone(&session),
                model_entry: Arc::new(StdMutex::new(dummy_model_entry())),
                is_streaming: Arc::new(AtomicBool::new(false)),
                is_compacting: Arc::new(AtomicBool::new(false)),
                config: Config::default(),
                save_enabled: false,
            };

            ext_session
                .set_model("anthropic".to_string(), "claude-sonnet-4-5".to_string())
                .await
                .expect("first model update");
            ext_session
                .set_model("anthropic".to_string(), "claude-sonnet-4-5".to_string())
                .await
                .expect("second model update");

            let cx = Cx::for_request();
            let guard = session.lock(&cx).await.expect("lock session");
            let model_changes = guard
                .entries_for_current_path()
                .iter()
                .filter(|entry| matches!(entry, crate::session::SessionEntry::ModelChange(_)))
                .count();
            assert_eq!(model_changes, 1);
        });
    }

    #[test]
    fn get_state_reports_configured_queue_modes() {
        let runtime = RuntimeBuilder::current_thread()
            .build()
            .expect("runtime build");

        runtime.block_on(async {
            let session = Arc::new(Mutex::new(Session::in_memory()));
            let ext_session = InteractiveExtensionSession {
                session,
                model_entry: Arc::new(StdMutex::new(dummy_model_entry())),
                is_streaming: Arc::new(AtomicBool::new(false)),
                is_compacting: Arc::new(AtomicBool::new(false)),
                config: Config {
                    steering_mode: Some("all".to_string()),
                    follow_up_mode: Some("one-at-a-time".to_string()),
                    ..Config::default()
                },
                save_enabled: false,
            };

            let state = ext_session.get_state().await;
            assert_eq!(state["steeringMode"], "all");
            assert_eq!(state["followUpMode"], "one-at-a-time");
        });
    }
}
