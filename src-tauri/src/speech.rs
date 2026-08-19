use crate::error::{AppError, AppResult};
use std::sync::{mpsc, Mutex, OnceLock};

#[cfg(windows)]
use windows::core::{BSTR, GUID};
#[cfg(windows)]
use windows::Win32::Media::Speech::{
    ISpeechVoice, SVSFPurgeBeforeSpeak, SVSFlagsAsync, SpVoice, SpeechVoiceSpeakFlags,
};
#[cfg(windows)]
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};

#[cfg(windows)]
type RequestResult = mpsc::Sender<AppResult<()>>;

#[cfg(windows)]
enum SpeechRequest {
    Speak {
        text: String,
        rate: f64,
        reply: RequestResult,
    },
    Stop {
        reply: RequestResult,
    },
}

#[cfg(windows)]
static SPEECH_SENDER: OnceLock<Mutex<Option<mpsc::Sender<SpeechRequest>>>> = OnceLock::new();

pub fn speak_text(text: String, language: String, rate: f64) -> AppResult<()> {
    if text.trim().is_empty() {
        return Err(AppError::InvalidData("speech text is empty".into()));
    }
    if language != "en-US" {
        return Err(AppError::InvalidData(
            "only en-US offline speech is supported".into(),
        ));
    }
    if !rate.is_finite() || !(-10.0..=10.0).contains(&rate) {
        return Err(AppError::InvalidData(
            "speech rate must be between -10 and 10".into(),
        ));
    }
    #[cfg(windows)]
    {
        let sender = speech_sender()?;
        let (reply, result) = mpsc::channel();
        sender
            .send(SpeechRequest::Speak { text, rate, reply })
            .map_err(|_| AppError::Resource("speech worker stopped".into()))?;
        result
            .recv()
            .map_err(|_| AppError::Resource("speech worker stopped".into()))?
    }
    #[cfg(not(windows))]
    {
        let _ = (text, language, rate);
        Err(AppError::Resource(
            "Windows offline speech is unavailable on this platform".into(),
        ))
    }
}

pub fn stop_speaking() -> AppResult<()> {
    #[cfg(windows)]
    {
        let Some(lock) = SPEECH_SENDER.get() else {
            return Ok(());
        };
        let sender = lock
            .lock()
            .map_err(|_| AppError::Resource("speech worker lock is poisoned".into()))?
            .clone();
        let Some(sender) = sender else {
            return Ok(());
        };
        let (reply, result) = mpsc::channel();
        sender
            .send(SpeechRequest::Stop { reply })
            .map_err(|_| AppError::Resource("speech worker stopped".into()))?;
        result
            .recv()
            .map_err(|_| AppError::Resource("speech worker stopped".into()))?
    }
    #[cfg(not(windows))]
    Ok(())
}

#[cfg(windows)]
fn speech_sender() -> AppResult<mpsc::Sender<SpeechRequest>> {
    let lock = SPEECH_SENDER.get_or_init(|| Mutex::new(None));
    let mut guard = lock
        .lock()
        .map_err(|_| AppError::Resource("speech worker lock is poisoned".into()))?;
    if let Some(sender) = guard.as_ref() {
        return Ok(sender.clone());
    }
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("vocapip-speech".into())
        .spawn(move || speech_worker(receiver))
        .map_err(|error| AppError::Resource(format!("unable to start speech worker: {error}")))?;
    *guard = Some(sender.clone());
    Ok(sender)
}

#[cfg(windows)]
fn speech_worker(receiver: mpsc::Receiver<SpeechRequest>) {
    let voice = match create_english_voice() {
        Ok(voice) => voice,
        Err(error) => {
            let message = error.to_string();
            for request in receiver {
                let reply = match request {
                    SpeechRequest::Speak { reply, .. } | SpeechRequest::Stop { reply } => reply,
                };
                let _ = reply.send(Err(AppError::Resource(message.clone())));
            }
            return;
        }
    };
    for request in receiver {
        match request {
            SpeechRequest::Speak { text, rate, reply } => {
                let result = (|| {
                    unsafe { voice.SetRate(rate.round() as i32) }.map_err(|error| {
                        AppError::Resource(format!("unable to set speech rate: {error}"))
                    })?;
                    let flags = SpeechVoiceSpeakFlags(SVSFlagsAsync.0 | SVSFPurgeBeforeSpeak.0);
                    unsafe { voice.Speak(&BSTR::from(text), flags) }
                        .map(|_| ())
                        .map_err(|error| {
                            AppError::Resource(format!("unable to speak text: {error}"))
                        })
                })();
                let _ = reply.send(result);
            }
            SpeechRequest::Stop { reply } => {
                let result = unsafe { voice.Skip(&BSTR::from("Sentence"), i32::MAX) }
                    .map(|_| ())
                    .map_err(|error| AppError::Resource(format!("unable to stop speech: {error}")));
                let _ = reply.send(result);
            }
        }
    }
}

#[cfg(windows)]
fn create_english_voice() -> AppResult<ISpeechVoice> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|error| {
                AppError::Resource(format!("unable to initialize Windows speech: {error}"))
            })?;
        let clsid: GUID = SpVoice;
        let voice: ISpeechVoice = CoCreateInstance(&clsid, None, CLSCTX_ALL).map_err(|error| {
            AppError::Resource(format!("Windows speech is unavailable: {error}"))
        })?;
        let voices = voice
            .GetVoices(&BSTR::from("Language=409"), &BSTR::new())
            .map_err(|error| {
                AppError::Resource(format!("unable to enumerate English voices: {error}"))
            })?;
        let count = voices.Count().map_err(|error| {
            AppError::Resource(format!("unable to inspect English voices: {error}"))
        })?;
        if count <= 0 {
            return Err(AppError::Resource(
                "no English offline voice is installed".into(),
            ));
        }
        let token = voices.Item(0).map_err(|error| {
            AppError::Resource(format!("unable to select English voice: {error}"))
        })?;
        voice.putref_Voice(&token).map_err(|error| {
            AppError::Resource(format!("unable to activate English voice: {error}"))
        })?;
        Ok(voice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_speech_contract() {
        assert!(speak_text("hello".into(), "zh-CN".into(), 1.0).is_err());
        assert!(speak_text("".into(), "en-US".into(), 1.0).is_err());
        assert!(speak_text("hello".into(), "en-US".into(), 99.0).is_err());
    }
}
