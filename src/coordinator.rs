use anyhow::Result;
use base64::prelude::*;
use log::{debug, info};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch, Mutex as TokioMutex};
use tokio::time::{sleep, Duration};

use crate::cancellation::GhostwriterCancellation;
use crate::config::Config;
use crate::embedded_assets::load_config;
use crate::keyboard::Keyboard;
use crate::llm_engine::{LLMEngine, ModelExecutionStatus};
use crate::memory::{MemoryUpdater, MEMORY_DIR};
use crate::messages;
use crate::screenshot::Screenshot;
use crate::segmenter::ImageAnalyzer;
use crate::simulation::SimulationConfig;
use crate::touch::Touch;

/// Events that can trigger AI processing
#[derive(Debug, Clone)]
pub enum TriggerEvent {
    /// User touched the trigger corner
    UserTouch,
    /// Trigger via web API (for testing/simulation)
    WebTrigger,
}

/// Progress states during AI processing
/// Uses ModelExecutionStatus for LLM operations, plus additional states for the full workflow
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressState {
    /// No processing happening
    Idle,
    /// Waiting for user trigger
    WaitingForTrigger,
    /// Taking screenshot
    TakingScreenshot,
    /// LLM execution state
    LlmState(ModelExecutionStatus),
    /// A short, self-erasing message for the person at the tablet (see messages.rs)
    Message(&'static str),
    /// Processing completed successfully
    Done,
}

/// Message from coordinator to processing task
#[derive(Debug)]
pub struct ProcessingRequest {
    /// The trigger event that started this
    pub trigger: TriggerEvent,
}

/// Communication channels for the coordinator
pub struct CoordinatorChannels {
    /// Send trigger events to coordinator
    pub trigger_tx: mpsc::Sender<TriggerEvent>,
    /// Receive trigger events in coordinator
    pub trigger_rx: mpsc::Receiver<TriggerEvent>,

    /// Broadcast progress state updates
    pub progress_tx: watch::Sender<ProgressState>,
    /// Receive progress state updates
    pub progress_rx: watch::Receiver<ProgressState>,
}

impl CoordinatorChannels {
    pub fn new() -> Self {
        let (trigger_tx, trigger_rx) = mpsc::channel(10);
        let (progress_tx, progress_rx) = watch::channel(ProgressState::Idle);

        Self {
            trigger_tx,
            trigger_rx,
            progress_tx,
            progress_rx,
        }
    }
}

impl Default for CoordinatorChannels {
    fn default() -> Self {
        Self::new()
    }
}

/// Task that waits for triggers and notifies the coordinator
pub async fn trigger_task(
    touch: Arc<tokio::sync::RwLock<Touch>>,
    trigger_tx: mpsc::Sender<TriggerEvent>,
    cancellation: Arc<GhostwriterCancellation>,
    no_trigger: bool,
) -> Result<()> {
    info!("Trigger task starting");

    loop {
        debug!("Trigger loop looping");

        if no_trigger {
            debug!("No-trigger mode: auto-triggering");
            if trigger_tx.send(TriggerEvent::UserTouch).await.is_err() {
                info!("Trigger receiver dropped, exiting trigger task");
                break;
            }
            // In no-trigger mode, wait a bit before next auto-trigger or check for cancellation
            tokio::select! {
                _ = sleep(Duration::from_millis(100)) => {
                    if cancellation.should_cancel_main() {
                        info!("Trigger task: cancelled in no-trigger mode");
                        break;
                    }
                }
                _ = async {
                    while !cancellation.should_cancel_main() {
                        sleep(Duration::from_millis(10)).await;
                    }
                } => {
                    info!("Trigger task: cancelled in no-trigger mode");
                    break;
                }
            }
            continue;
        }

        info!("Trigger task: waiting for touch trigger...");

        debug!("Trigger task: about to acquire touch write lock");
        let mut touch_guard = touch.write().await;
        debug!("Trigger task: acquired touch write lock, calling wait_for_trigger");

        match touch_guard.wait_for_trigger(&cancellation).await {
            Ok(()) => {
                debug!("Trigger task: wait_for_trigger returned Ok, touch detected");
                info!("Trigger task: touch detected");

                // Drop the lock before sending the event so processing_task can acquire it
                drop(touch_guard);
                debug!("Trigger task: dropped touch write lock");

                if trigger_tx.send(TriggerEvent::UserTouch).await.is_err() {
                    info!("Trigger receiver dropped, exiting trigger task");
                    break;
                }
                debug!("Trigger task: sent trigger event, continuing loop");
            }
            Err(e) => {
                debug!("Trigger task: wait_for_trigger returned Err: {}", e);
                if e.to_string().contains("cancelled") {
                    info!("Trigger task: cancelled (likely config change)");
                    return Ok(()); // Clean exit for restart
                } else {
                    info!("Trigger task: error waiting for trigger: {}", e);
                    return Err(e);
                }
            }
        }
    }

    debug!("Escaped from trigger task loop");

    Ok(())
}

/// Task that monitors for cancel touch during processing
pub async fn cancel_monitor_task(touch: Arc<tokio::sync::RwLock<Touch>>, cancellation: Arc<GhostwriterCancellation>) -> Result<()> {
    info!("Cancel monitor task: starting");

    // Wait for any touch to cancel
    match touch.write().await.wait_for_trigger(&cancellation).await {
        Ok(()) => {
            info!("Cancel monitor task: touch detected, cancelling processing");
            cancellation.cancel_execution();
            Ok(())
        }
        Err(e) => {
            if e.to_string().contains("cancelled") {
                info!("Cancel monitor task: processing completed before touch");
                Ok(())
            } else {
                info!("Cancel monitor task: error: {}", e);
                Err(e)
            }
        }
    }
}

/// Task that displays progress updates on the keyboard
pub async fn progress_task(
    keyboard: Arc<Mutex<Keyboard>>,
    mut progress_rx: watch::Receiver<ProgressState>,
    cancellation: Arc<GhostwriterCancellation>,
) -> Result<()> {
    info!("Progress task starting");

    let mut current_state = ProgressState::Idle;
    let mut dots: u32 = 0;
    // The execution token is replaced every run; the main token is what ends this task.
    let cancel_token = cancellation.main_token();

    loop {
        tokio::select! {
            // Check for cancellation
            _ = cancel_token.cancelled() => {
                info!("Progress task cancelled");
                // Clear any progress display
                if let Ok(mut kb) = keyboard.lock() {
                    let _ = kb.progress_end();
                }
                return Ok(());
            }

            // Watch for progress updates
            result = progress_rx.changed() => {
                if result.is_err() {
                    info!("Progress sender dropped, exiting progress task");
                    break;
                }

                let new_state = progress_rx.borrow().clone();
                if new_state != current_state {
                    current_state = new_state.clone();

                    match &current_state {
                        ProgressState::Idle => {
                            info!("Progress: Idle");
                            if let Ok(mut kb) = keyboard.lock() {
                                let _ = kb.progress_end();
                            }
                        }
                        ProgressState::WaitingForTrigger => {
                            info!("Progress: Waiting for trigger");
                        }
                        ProgressState::TakingScreenshot => {
                            info!("Progress: Taking screenshot...");
                        }
                        ProgressState::LlmState(ModelExecutionStatus::BuildingContext) => {
                            info!("Progress: Building context...");
                            dots = 0;
                            if let Ok(mut kb) = keyboard.lock() {
                                let _ = kb.progress(messages::WORKING);
                            }
                        }
                        ProgressState::LlmState(ModelExecutionStatus::LlmProcessing) => {
                            info!("Progress: Thinking...");
                        }
                        ProgressState::LlmState(ModelExecutionStatus::ProcessingResponse) => {
                            info!("Progress: Processing response...");
                        }
                        ProgressState::LlmState(ModelExecutionStatus::CallingTools) => {
                            info!("Progress: Executing tools...");
                            if let Ok(mut kb) = keyboard.lock() {
                                let _ = kb.progress_end();
                            }
                        }
                        ProgressState::LlmState(ModelExecutionStatus::Done) => {
                            debug!("Progress: LLM Done");
                        }
                        ProgressState::LlmState(ModelExecutionStatus::Error(msg)) => {
                            debug!("Progress: Error - {}", msg);
                        }
                        ProgressState::Message(message) => {
                            info!("Progress: on-page message: {}", message);
                            if let Ok(mut kb) = keyboard.lock() {
                                let _ = kb.progress_end();
                                let _ = kb.progress(message);
                            }
                        }
                        ProgressState::Done => {
                            debug!("Progress: Done");
                        }
                    }
                }
            }

            // Add dots while waiting for the model, up to a limit
            _ = sleep(Duration::from_millis(messages::DOT_INTERVAL_MS)) => {
                if matches!(current_state, ProgressState::LlmState(ModelExecutionStatus::LlmProcessing)) && dots < messages::MAX_DOTS {
                    if let Ok(mut kb) = keyboard.lock() {
                        let _ = kb.progress(messages::DOT);
                        dots += 1;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Task that processes a trigger: screenshot -> LLM -> tool execution.
///
/// Whatever happens inside, the page is left clean: the working indicator is
/// erased, and a failure shows a short message for a moment before that is
/// erased too.
pub async fn processing_task(
    config: Config,
    engine: Arc<TokioMutex<Box<dyn LLMEngine>>>,
    progress_tx: watch::Sender<ProgressState>,
    cancellation: Arc<GhostwriterCancellation>,
    tap_touch: Arc<TokioMutex<Touch>>,
    last_reply: Arc<Mutex<Option<String>>>,
    memory: Option<Arc<MemoryUpdater>>,
) -> Result<()> {
    let result = processing_inner(&config, engine, &progress_tx, &cancellation, tap_touch, last_reply, memory).await;

    if let Err(e) = &result {
        let error_msg = e.to_string();
        info!("Processing task: error: {}", error_msg);
        // A cancelled run (config change or shutdown) is not a failure to report on the page.
        if !error_msg.contains("cancelled") && !error_msg.contains("canceled") {
            let _ = progress_tx.send(ProgressState::Message(messages::pick(&error_msg)));
            sleep(Duration::from_millis(messages::MESSAGE_HOLD_MS)).await;
        }
    }

    // Always leave the page clean, whichever path was taken.
    let _ = progress_tx.send(ProgressState::Idle);
    result
}

/// Script that sets the tablet clock from the network (installed by deploy/install.sh).
const FIX_CLOCK_SCRIPT: &str = "/home/root/ghostwriter/fix-clock.sh";
/// Unix time for 2025-01-01: anything earlier means the clock was never set
/// after boot (the rM2's clock battery may be dead), and TLS would reject
/// every certificate as "not yet valid".
const PLAUSIBLE_CLOCK_SECS: u64 = 1_735_689_600;

/// Make sure the system clock is plausible before talking to the API.
fn ensure_clock_is_plausible() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now >= PLAUSIBLE_CLOCK_SECS {
        return;
    }
    if !std::path::Path::new(FIX_CLOCK_SCRIPT).exists() {
        log::warn!(
            "System clock is at {} (before 2025) and {} is missing; API calls may fail",
            now,
            FIX_CLOCK_SCRIPT
        );
        return;
    }
    info!("System clock looks wrong ({}); running {}", now, FIX_CLOCK_SCRIPT);
    match std::process::Command::new("sh").arg(FIX_CLOCK_SCRIPT).output() {
        Ok(output) => info!("fix-clock: status {} {}", output.status, String::from_utf8_lossy(&output.stdout).trim()),
        Err(e) => log::warn!("fix-clock could not run: {}", e),
    }
}

/// Load a prompt file and check it has the one field the run needs.
fn load_prompt(name: &str) -> Result<serde_json::Value> {
    let raw = load_config(name)?;
    let json = serde_json::from_str::<serde_json::Value>(&raw)?;
    if json["prompt"].as_str().is_none() {
        anyhow::bail!("'{}' has no 'prompt' string", name);
    }
    Ok(json)
}

async fn processing_inner(
    config: &Config,
    engine: Arc<TokioMutex<Box<dyn LLMEngine>>>,
    progress_tx: &watch::Sender<ProgressState>,
    cancellation: &GhostwriterCancellation,
    tap_touch: Arc<TokioMutex<Touch>>,
    last_reply: Arc<Mutex<Option<String>>>,
    memory: Option<Arc<MemoryUpdater>>,
) -> Result<()> {
    info!("Processing task: starting");
    if let Ok(mut reply) = last_reply.lock() {
        *reply = None;
    }

    // Update progress: taking screenshot
    info!("Setting ProgressState::TakingScreenshot");
    let _ = progress_tx.send(ProgressState::TakingScreenshot);
    tokio::time::sleep(Duration::from_millis(10)).await; // Give progress_task time

    // Take screenshot
    let screenshot_path = config.save_screenshot.clone();
    let base64_image = if let Some(input_png) = &config.input_png {
        BASE64_STANDARD.encode(std::fs::read(input_png)?)
    } else {
        let mut screenshot = if config.is_test_mode() {
            let simulation_config = SimulationConfig::from_config(config);
            Screenshot::new_simulated(simulation_config)?
        } else {
            Screenshot::new()?
        };
        screenshot.take_screenshot()?;
        if let Some(save_screenshot) = &config.save_screenshot {
            info!("Saving screenshot to {}", save_screenshot);
            screenshot.save_image(save_screenshot)?;
        }
        screenshot.base64()?
    };

    if config.no_submit {
        info!("Skipping LLM submission (no_submit mode)");
        let _ = progress_tx.send(ProgressState::Done);
        return Ok(());
    }

    // A tap while the on-screen keyboard is open is a key press (the space bar
    // sits in the bottom-centre zone), not a request: ignore it quietly.
    if let Ok(png) = BASE64_STANDARD.decode(&base64_image) {
        if Screenshot::keyboard_looks_open(&png) {
            info!("On-screen keyboard looks open; ignoring this tap");
            let _ = progress_tx.send(ProgressState::Done);
            return Ok(());
        }
    }

    // Decide where the reply goes (before showing "Thinking"). This uses the
    // writer-only handle: the shared Touch is held by the trigger listener.
    if config.reply_on_new_page {
        // Turn to the next page (a new one at the end of the notebook) so the
        // typed reply never lands on top of the handwriting.
        info!("Turning to the next page for the reply");
        if let Err(e) = tap_touch.lock().await.swipe_to_next_page().await {
            info!("Failed to swipe to the next page: {}", e);
        }
        // Give the e-ink page turn time to finish before typing.
        sleep(Duration::from_millis(1500)).await;
    } else if let Err(e) = tap_touch.lock().await.tap_middle_bottom().await {
        // Same-page mode: tap bottom-middle so the text cursor moves below the notes.
        info!("Failed to tap middle bottom: {}", e);
    }

    // Update progress: building context
    let _ = progress_tx.send(ProgressState::LlmState(ModelExecutionStatus::BuildingContext));
    tokio::time::sleep(Duration::from_millis(10)).await; // Give progress_task time

    // Apply segmentation if requested
    let segmentation_description = if config.apply_segmentation {
        let image_path = config
            .input_png
            .as_ref()
            .or(screenshot_path.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Segmentation requires either input_png or save_screenshot"))?;

        info!("Applying segmentation to {}", image_path);
        let analyzer = ImageAnalyzer::new(0.001, 10); // min_region_size=0.1%, max_regions=10
        match analyzer.analyze_image(image_path) {
            Ok(result) => {
                let description = analyzer.generate_description(&result);
                info!("Segmentation found {} regions", result.regions.len());
                Some(description)
            }
            Err(e) => {
                info!("Segmentation failed: {}, continuing without it", e);
                None
            }
        }
    } else {
        None
    };

    // Load the prompt, falling back to the built-in default if the configured one is missing or broken
    let prompt_json = match load_prompt(&config.prompt) {
        Ok(json) => json,
        Err(e) => {
            log::error!(
                "Could not load prompt '{}': {}. Using the built-in {}",
                config.prompt,
                e,
                crate::config::DEFAULT_PROMPT
            );
            load_prompt(crate::config::DEFAULT_PROMPT)?
        }
    };
    let mut prompt = prompt_json["prompt"].as_str().unwrap_or("").to_string();

    // Add what the coach has learned about her so far
    if memory.is_some() {
        let learned = MemoryUpdater::load(std::path::Path::new(MEMORY_DIR));
        if !learned.is_empty() {
            prompt.push_str(&MemoryUpdater::prompt_section(&learned));
        }
    }

    // Add segmentation to prompt if available
    if let Some(seg_desc) = segmentation_description {
        prompt.push_str("\n\nImage Analysis:\n");
        prompt.push_str(&seg_desc);
    }

    // A wrong clock makes every TLS certificate look invalid; fix it before the request.
    tokio::task::block_in_place(ensure_clock_is_plausible);

    // Prepare engine
    let mut engine_guard = engine.lock().await;
    engine_guard.clear_content();
    engine_guard.add_image_content(&base64_image);
    engine_guard.add_text_content(&prompt);

    // Create status callback that wraps model execution status in LlmState
    let progress_tx_clone = progress_tx.clone();
    let status_callback = Some(Box::new(move |status: ModelExecutionStatus| {
        let _ = progress_tx_clone.send(ProgressState::LlmState(status));
    }) as Box<dyn FnMut(ModelExecutionStatus) + Send>);

    // Execute LLM; errors are reported on the page by processing_task
    info!("Processing task: calling LLM");
    engine_guard.execute(cancellation, status_callback).await?;

    // Learn from this page in the background so she never waits for it
    if let Some(memory) = memory {
        let reply = last_reply.lock().ok().and_then(|r| r.clone()).unwrap_or_default();
        if MemoryUpdater::should_skip(&reply) {
            info!("memory: nothing to learn from this reply");
        } else {
            let page = base64_image.clone();
            tokio::spawn(async move {
                if let Err(e) = memory.update(&page, &reply).await {
                    log::warn!("memory: update failed: {}", e);
                }
            });
        }
    }

    // Write model output if configured
    if let Some(model_output_file) = &config.model_output_file {
        info!("Would write model output to {}", model_output_file);
        // Note: The actual model output would need to be captured from the engine
        // This is a placeholder - the LLMEngine trait would need to expose the raw response
    }

    let _ = progress_tx.send(ProgressState::Done);
    info!("Processing task: completed successfully");
    Ok(())
}
