use crate::error::Result;

/// Wrapper around arboard clipboard for copying color strings.
#[allow(dead_code)]
pub struct Clipboard {
    inner: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let inner = arboard::Clipboard::new()?;
        Ok(Self { inner })
    }

    pub fn copy_text(&mut self, text: &str) -> Result<()> {
        self.inner.set_text(text)?;
        Ok(())
    }
}
