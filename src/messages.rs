//! On-page status messages.
//!
//! Everything typed through these constants goes through `Keyboard::progress`
//! and is erased again by `Keyboard::progress_end`, so only the model's reply
//! ever stays on the page. Keep every message plain ASCII, under 80 characters,
//! and free of error codes: the reader is not the person who installed this.

/// Typed right after the screenshot while the request is in flight.
pub const WORKING: &str = "Thinking";
/// Appended every `DOT_INTERVAL_MS` while waiting, up to `MAX_DOTS` times.
pub const DOT: &str = ".";
pub const DOT_INTERVAL_MS: u64 = 1000;
pub const MAX_DOTS: u32 = 60;
/// How long a failure message stays on the page before it is erased.
pub const MESSAGE_HOLD_MS: u64 = 10_000;

pub const NO_INTERNET: &str = "No internet connection. Check Wi-Fi, then tap the icon again.";
pub const SERVICE_BUSY: &str = "The AI service is busy right now. Please tap again in a minute.";
pub const NOT_SET_UP: &str = "The assistant is not set up yet (no API key). Ask the person who installed it.";
pub const SOMETHING_WRONG: &str = "Something went wrong. Please tap again in a moment.";

/// Map an internal error string to the short message shown on the page.
pub fn pick(error: &str) -> &'static str {
    let e = error.to_ascii_lowercase();
    if e.contains("no api key") || e.contains("api 401") || e.contains("api 403") {
        NOT_SET_UP
    } else if e.contains("api 429") || e.contains("api 5") || e.contains("overloaded") {
        SERVICE_BUSY
    } else if e.contains("connect") || e.contains("dns") || e.contains("timed out") || e.contains("timeout") || e.contains("error sending request") {
        NO_INTERNET
    } else {
        SOMETHING_WRONG
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_right_message() {
        assert_eq!(pick("API 401: {\"error\": {\"code\": \"invalid_api_key\"}}"), NOT_SET_UP);
        assert_eq!(pick("No API key configured: set OPENAI_API_KEY"), NOT_SET_UP);
        assert_eq!(pick("API 503: overloaded"), SERVICE_BUSY);
        assert_eq!(pick("API 429: rate limit"), SERVICE_BUSY);
        assert_eq!(pick("error sending request for url (https://api.openai.com/v1/chat/completions)"), NO_INTERNET);
        assert_eq!(pick("operation timed out"), NO_INTERNET);
        assert_eq!(pick("Model returned neither text nor a tool call"), SOMETHING_WRONG);
    }

    #[test]
    fn messages_are_plain_ascii_and_short() {
        for message in [WORKING, NO_INTERNET, SERVICE_BUSY, NOT_SET_UP, SOMETHING_WRONG] {
            assert!(message.is_ascii(), "{message}");
            assert!(message.len() < 80, "{message}");
        }
    }
}
